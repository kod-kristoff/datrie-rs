use std::io::{self, SeekFrom};

use ::libc;
use byteorder::{BigEndian, WriteBytesExt};
use core::mem::size_of;

use crate::{fileutils::ReadExt, trie::TrieCharString, DatrieError, DatrieResult};

use super::TrieCharStr;

#[cfg(test)]
mod tests;

pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type TrieChar = u8;
pub type TrieIndex = i32;
pub type TrieData = i32;
#[derive(Clone, Debug, PartialEq)]
pub struct Tail {
    tails2: Vec<TailBlock>,
    pub first_free: TrieIndex,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TailBlock {
    pub next_free: TrieIndex,
    pub data: TrieData,
    pub suffix: TrieCharString,
}

const TAIL_START_BLOCKNO: usize = 1;

impl Default for TailBlock {
    fn default() -> Self {
        TailBlock {
            next_free: -1,
            data: -1,
            suffix: Default::default(),
        }
    }
}
impl TailBlock {
    fn reset(&mut self) {
        self.data = -1;
        self.suffix = Default::default();
    }
}

impl Default for Tail {
    fn default() -> Self {
        Self::new()
    }
}

impl Tail {
    pub fn new() -> Tail {
        Tail {
            first_free: 0,
            tails2: Vec::new(),
        }
    }
}

impl Tail {
    pub fn fread_safe<R: ReadExt + io::Seek>(reader: &mut R) -> DatrieResult<Tail> {
        let save_pos = reader.stream_position()?;
        Tail::do_fread_safe(reader).map_err(|err| {
            if let Err(io_err) = reader.seek(SeekFrom::Start(save_pos)) {
                return io_err.into();
            }
            err
        })
    }
    fn do_fread_safe<R: ReadExt>(reader: &mut R) -> DatrieResult<Tail> {
        let current_block: u64;
        let mut sig: u32 = 0;
        reader.read_uint32(&mut sig)?;
        if sig != Self::SIGNATURE {
            return Err(DatrieError::new(
                crate::ErrorKind::InvalidFileSignature,
                format!("tail: unexpected signature '{}'", sig),
            ));
        }
        let mut first_free = 0;
        let mut num_tails = 0;
        reader.read_int32(&mut first_free)?;
        reader.read_int32(&mut num_tails)?;
        if num_tails as libc::c_ulong
            > (18446744073709551615 as libc::c_ulong)
                .wrapping_div(::core::mem::size_of::<TailBlock>() as libc::c_ulong)
        {
            return Err(DatrieError::new(
                crate::ErrorKind::Bug,
                "failed to read tail: num_tails too large".into(),
            ));
        }
        let mut tails2 = Vec::with_capacity(num_tails as usize);
        let mut i = 0 as libc::c_int;
        loop {
            if i >= num_tails {
                current_block = 15904375183555213903;
                break;
            }
            let mut length: i16 = 0;
            let mut next_free = 0;
            let mut data = 0;
            // reader.read_int32(&mut next_free)?;
            if reader.read_int32(&mut next_free).is_err()
                || reader.read_int32(&mut data).is_err()
                || reader.read_int16(&mut length).is_err()
            {
                dbg!(next_free);
                current_block = 1386273818809128762;
                break;
            }
            let mut suffix_data = vec![0; length as usize];
            if length as libc::c_int > 0 as libc::c_int
                && reader.read_exact(&mut suffix_data).is_err()
            {
                current_block = 1386273818809128762;
                break;
            }
            tails2.push(TailBlock {
                next_free,
                data,
                suffix: TrieCharString::new(suffix_data).unwrap(),
            });
            i += 1;
        }
        if let 15904375183555213903 = current_block {
            return Ok(Tail { first_free, tails2 });
        }
        Err(DatrieError::new(
            crate::ErrorKind::Bug,
            "failed to read tail".into(),
        ))
    }
}

impl Tail {
    pub fn num_tails(&self) -> usize {
        self.tails2.len()
    }

    pub fn get_serialized_size(&self) -> usize {
        let static_count = ::core::mem::size_of::<i32>() + 2 * ::core::mem::size_of::<TrieIndex>();
        let mut dynamic_count: usize = 0;
        if self.num_tails() > 0 {
            dynamic_count += (size_of::<TrieIndex>() + size_of::<TrieData>() + size_of::<i16>())
                * self.num_tails();
            for i in 0..(self.num_tails()) {
                dynamic_count += self.tails2[i].suffix.as_bytes().len();
            }
        }
        static_count + dynamic_count
    }

    /// Magic number signature for the Tail binary format (0xdffcdffc)
    /// Introduced in the initial binary serialization format
    const SIGNATURE: u32 = 0xdffcdffc;
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> DatrieResult<usize> {
        writer.write_u32::<BigEndian>(Self::SIGNATURE)?;
        writer.write_i32::<BigEndian>(self.first_free)?;
        writer.write_i32::<BigEndian>(self.num_tails() as i32)?;
        let mut written = 12;
        for i in 0..self.num_tails() {
            let next_free = self.tails2[i].next_free;
            let data = self.tails2[i].data;
            writer.write_i32::<BigEndian>(next_free)?;
            writer.write_i32::<BigEndian>(data)?;
            let length = self.tails2[i].suffix.as_bytes().len();
            writer.write_i16::<BigEndian>(length as i16)?;
            written += 10;
            writer.write_all(self.tails2[i].suffix.as_bytes())?;
            written += length;
        }
        Ok(written)
    }
}

impl Tail {
    pub fn get_suffix(&self, index: TrieIndex) -> Option<&TrieCharStr> {
        self.tails2
            .get(index as usize - TAIL_START_BLOCKNO)
            .map(|s| s.suffix.as_trie_str())
    }
    pub fn take_suffix(&mut self, index: TrieIndex) -> Option<TrieCharString> {
        let index = index as usize - TAIL_START_BLOCKNO;
        if index < self.num_tails() {
            Some(std::mem::replace(
                &mut self.tails2[index].suffix,
                TrieCharString::default(),
            ))
        } else {
            None
        }
    }

    pub fn set_suffix(&mut self, index: TrieIndex, suffix: TrieCharString) -> bool {
        let index = index as usize - TAIL_START_BLOCKNO;
        if index < self.num_tails() {
            // if !suffix.() {
            self.tails2[index].suffix = suffix;
            // } else {
            //     self.tails2[index].suffix = TrieCharString::default();
            // }
            return true;
        }
        false
    }
}
impl Tail {
    pub fn add_suffix(&mut self, suffix: TrieCharString) -> TrieIndex {
        let new_block = self.alloc_block();
        if new_block == 0 {
            return 0;
        }
        self.set_suffix(new_block, suffix);
        new_block
    }
    fn alloc_block(&mut self) -> TrieIndex {
        let block: TrieIndex;
        if self.first_free != 0 {
            block = self.first_free;
            self.first_free = self.tails2[block as usize].next_free;
        } else {
            block = self.num_tails() as TrieIndex;
            self.tails2.push(TailBlock::default());
        }
        block + 1
    }
}
impl Tail {
    unsafe fn free_block(&mut self, mut block: TrieIndex) {
        block -= 1 as libc::c_int;
        if block >= self.num_tails() as TrieIndex {
            return;
        }
        self.tails2[block as usize].reset();
        let mut j = 0 as libc::c_int;
        let mut i = self.first_free;
        while i != 0 as libc::c_int && i < block {
            j = i;
            i = self.tails2[i as usize].next_free;
        }
        self.tails2[block as usize].next_free = i;
        if 0 as libc::c_int != j {
            self.tails2[j as usize].next_free = block;
        } else {
            self.first_free = block;
        };
    }
}
impl Tail {
    pub fn get_data(&self, mut index: TrieIndex) -> Option<TrieData> {
        index -= 1 as libc::c_int;
        if ((index as usize) < self.num_tails()) as libc::c_int as libc::c_long != 0 {
            Some(self.tails2[index as usize].data)
        } else {
            None
        }
    }

    pub fn set_data(&mut self, index: TrieIndex, data: TrieData) -> bool {
        let index = index as usize - TAIL_START_BLOCKNO;
        if index < self.num_tails() {
            self.tails2[index].data = data;
            return true;
        }
        false
    }
}
impl Tail {
    pub unsafe fn delete(&mut self, index: TrieIndex) {
        self.free_block(index);
    }

    fn walk_str(&self, index: TrieIndex, suffix_idx: &mut usize, s: &[TrieChar]) -> usize {
        println!("index={index}");
        let mut i = 0;
        let mut j = *suffix_idx;
        if let Some(suffix) = self.get_suffix(index) {
            println!("suffix={suffix:?}, str={s:?}");
            let suffix_bytes = suffix.to_bytes();
            if j >= suffix_bytes.len() {
                return i;
            }

            while i < s.len() {
                println!("i={i}, j={j}");
                if s.get(i) != suffix_bytes.get(j) {
                    break;
                }
                i += 1;
                if j == suffix_bytes.len() {
                    break;
                }
                j += 1;
            }
        }
        *suffix_idx = j;
        println!("i={i}, suffix_idx={j}");
        i
    }
    pub fn walk_char(&self, s: TrieIndex, suffix_idx: &mut usize, c: TrieChar) -> bool {
        if let Some(suffix) = self.get_suffix(s) {
            let suffix_bytes = suffix.to_bytes();
            if c == b'\0' && { *suffix_idx } == suffix_bytes.len() {
                return true;
            }
            if let Some(suffix_char) = suffix_bytes.get(*suffix_idx) {
                if *suffix_char == c {
                    *suffix_idx += 1;
                    return true;
                }
            }
        }
        false
    }
}
