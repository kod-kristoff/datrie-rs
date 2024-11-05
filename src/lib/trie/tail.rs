use std::io::{self, SeekFrom, Write};

use ::libc;
use byteorder::{BigEndian, WriteBytesExt};
use core::mem::size_of;

use crate::{
    fileutils::ReadExt, trie::TrieCharString, trie_string::trie_char_strdup, DatrieError,
    DatrieResult,
};

pub type FILE = libc::FILE;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type TrieChar = u8;
pub type TrieIndex = i32;
pub type TrieData = i32;
#[derive(Clone, Debug)]
pub struct Tail {
    tails2: Vec<TailBlock>,
    pub first_free: TrieIndex,
}
#[derive(Clone, Debug)]
pub struct TailBlock {
    pub next_free: TrieIndex,
    pub data: TrieData,
    pub suffix: TrieCharString,
}

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
        self.suffix.clear();
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

#[no_mangle]
pub unsafe extern "C" fn tail_fwrite(t: *const Tail, file: *mut FILE) -> libc::c_int {
    let tail = &*t;
    let size = tail.get_serialized_size();
    let mut buf = vec![0; size];
    if tail.serialize_to_slice(&mut buf).is_ok() {
        if libc::fwrite(
            buf.as_ptr() as *const libc::c_void,
            size_of::<u8>(),
            size,
            file,
        ) == size
        {
            0
        } else {
            -1
        }
    } else {
        -1
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
                dynamic_count += self.tails2[i].suffix.len();
            }
        }
        static_count + dynamic_count
    }

    const SIGNATURE: u32 = 0xdffcdffc;
    pub fn serialize(&self, writer: &mut dyn std::io::Write) -> DatrieResult<()> {
        writer.write_u32::<BigEndian>(Self::SIGNATURE)?;
        writer.write_i32::<BigEndian>(self.first_free)?;
        writer.write_i32::<BigEndian>(self.num_tails() as i32)?;
        for i in 0..self.num_tails() {
            let next_free = self.tails2[i].next_free;
            let data = self.tails2[i].data;
            writer.write_i32::<BigEndian>(next_free)?;
            writer.write_i32::<BigEndian>(data)?;
            let length = self.tails2[i].suffix.len();
            writer.write_i16::<BigEndian>(length as i16)?;
            writer.write_all(self.tails2[i].suffix.as_bytes())?;
        }
        Ok(())
    }
    pub fn serialize_to_slice(&self, mut buf: &mut [u8]) -> DatrieResult<usize> {
        buf.write_u32::<BigEndian>(Self::SIGNATURE)?;
        buf.write_i32::<BigEndian>(self.first_free)?;
        buf.write_i32::<BigEndian>(self.num_tails() as i32)?;
        let mut written: usize = 12;
        for i in 0..self.num_tails() {
            let next_free = self.tails2[i].next_free;
            let data = self.tails2[i].data;
            buf.write_i32::<BigEndian>(next_free)?;
            buf.write_i32::<BigEndian>(data)?;
            let length = self.tails2[i].suffix.len();
            buf.write_i16::<BigEndian>(length as i16)?;
            written += 10;
            buf.write_all(self.tails2[i].suffix.as_bytes())?;
            written += length;
        }
        Ok(written)
    }
}

#[no_mangle]
pub unsafe extern "C" fn tail_serialize(t: *const Tail, ptr: *mut *mut u8) -> libc::c_int {
    let tail: &Tail = &*t;
    let buf: &mut [u8] = core::slice::from_raw_parts_mut(*ptr, tail.get_serialized_size());
    if tail.serialize_to_slice(buf).is_ok() {
        0
    } else {
        -1
    }
}
impl Tail {
    pub unsafe fn get_suffix(&self, mut index: TrieIndex) -> *const TrieChar {
        index -= 1 as libc::c_int;
        if (index < self.num_tails() as TrieIndex) as libc::c_int as libc::c_long != 0 {
            self.tails2[index as usize].suffix.as_ptr()
        } else {
            std::ptr::null_mut::<TrieChar>()
        }
    }
    pub fn get_suffix2(&self, mut index: TrieIndex) -> Option<&[TrieChar]> {
        index -= 1;
        self.tails2.get(index as usize).map(|s| s.suffix.as_bytes())
    }
    pub unsafe fn set_suffix(&mut self, mut index: TrieIndex, suffix: *const TrieChar) -> Bool {
        index -= 1 as libc::c_int;
        if ((index as usize) < self.num_tails()) as libc::c_int as libc::c_long != 0 {
            let tmp: *mut TrieChar;
            if !suffix.is_null() {
                tmp = trie_char_strdup(suffix);
                if tmp.is_null() as libc::c_int as libc::c_long != 0 {
                    return DA_FALSE;
                }
            }
            if !suffix.is_null() {
                self.tails2[index as usize].suffix.replace_from_ptr(suffix);
            } else {
                self.tails2[index as usize].suffix.clear();
            }
            return DA_TRUE;
        }
        DA_FALSE
    }
}
impl Tail {
    pub unsafe fn add_suffix(
        &mut self,
        // mut t: *mut Tail,
        suffix: *const TrieChar,
    ) -> TrieIndex {
        let new_block: TrieIndex = self.alloc_block();
        if (0 as libc::c_int == new_block) as libc::c_int as libc::c_long != 0 {
            return 0 as libc::c_int;
        }
        self.set_suffix(new_block, suffix);
        new_block
    }
    unsafe fn alloc_block(&mut self) -> TrieIndex {
        let block: TrieIndex;
        if 0 as libc::c_int != self.first_free {
            block = self.first_free;
            self.first_free = self.tails2[block as usize].next_free;
        } else {
            block = self.num_tails() as TrieIndex;
            self.tails2.push(TailBlock::default());
        }
        block + 1 as libc::c_int
    }
}
// }
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
    pub fn get_data(&self, mut index: TrieIndex) -> TrieData {
        index -= 1 as libc::c_int;
        if ((index as usize) < self.num_tails()) as libc::c_int as libc::c_long != 0 {
            self.tails2[index as usize].data
        } else {
            -(1 as libc::c_int)
        }
    }
    pub fn get_data2(&self, mut index: TrieIndex) -> Option<TrieData> {
        index -= 1 as libc::c_int;
        if ((index as usize) < self.num_tails()) as libc::c_int as libc::c_long != 0 {
            Some(self.tails2[index as usize].data)
        } else {
            None
        }
    }
    pub fn set_data(&mut self, mut index: TrieIndex, data: TrieData) -> Bool {
        index -= 1 as libc::c_int;
        if ((index as usize) < self.num_tails()) as libc::c_int as libc::c_long != 0 {
            self.tails2[index as usize].data = data;
            return DA_TRUE;
        }
        DA_FALSE
    }
}
impl Tail {
    pub unsafe fn delete(&mut self, index: TrieIndex) {
        self.free_block(index);
    }
    pub unsafe fn walk_str(
        &self,
        s: TrieIndex,
        suffix_idx: *mut libc::c_short,
        str: *const TrieChar,
        len: libc::c_int,
    ) -> libc::c_int {
        let suffix = self.get_suffix(s);
        if suffix.is_null() as libc::c_int as libc::c_long != 0 {
            return DA_FALSE as libc::c_int;
        }
        let mut i = 0 as libc::c_int;
        let mut j = *suffix_idx;
        while i < len {
            if *str.offset(i as isize) as libc::c_int != *suffix.offset(j as isize) as libc::c_int {
                break;
            }
            i += 1;
            if '\0' as i32 == *suffix.offset(j as isize) as libc::c_int {
                break;
            }
            j += 1;
        }
        *suffix_idx = j;
        i
    }
    pub fn walk_str2(&self, index: TrieIndex, suffix_idx: &mut usize, s: &[TrieChar]) -> usize {
        let mut i = 0;
        if let Some(suffix) = self.get_suffix2(index) {
            let mut j = *suffix_idx;
            while i < s.len() {
                if s.get(i) != suffix.get(j) {
                    break;
                }
                i += 1;
                j += 1;
            }
            *suffix_idx = j;
        }
        i
    }
    pub fn walk_char2(&self, s: TrieIndex, suffix_idx: &mut usize, c: TrieChar) -> bool {
        if let Some(suffix) = self.get_suffix2(s) {
            if c == b'\0' && { *suffix_idx } == suffix.len() {
                return true;
            }
            if let Some(suffix_char) = suffix.get(*suffix_idx) {
                if *suffix_char == c {
                    *suffix_idx += 1;
                    return true;
                }
            }
        }
        false
    }
    pub unsafe fn walk_char(
        &self,
        s: TrieIndex,
        suffix_idx: *mut libc::c_short,
        c: TrieChar,
    ) -> Bool {
        let suffix = self.get_suffix(s);
        if suffix.is_null() as libc::c_int as libc::c_long != 0 {
            dbg!("suffix is_null");
            return DA_FALSE;
        }
        let suffix_char = *suffix.offset(*suffix_idx as isize);
        if suffix_char as libc::c_int == c as libc::c_int {
            if '\0' as i32 != suffix_char as libc::c_int {
                *suffix_idx += 1;
            }
            return DA_TRUE;
        }
        DA_FALSE
    }
}

#[cfg(test)]
mod tests {
    use crate::DatrieResult;

    use super::*;

    #[test]
    fn get_serialized_size_works() -> DatrieResult<()> {
        let mut tail = Tail::new();
        assert_eq!(tail.get_serialized_size(), 12);
        unsafe { tail.alloc_block() };
        assert_eq!(tail.get_serialized_size(), 22);
        unsafe {
            tail.add_suffix([b'a', b'p', b'\0'].as_ptr());
        }
        assert_eq!(tail.get_serialized_size(), 34);
        Ok(())
    }

    #[test]
    fn walk_char() {
        let mut tail = Tail::new();

        unsafe {
            tail.add_suffix([b'a', b'p', b'\0'].as_ptr());
            assert!(!tail.get_suffix(1).is_null());
        }
        // walk 'a'
        let mut suffix_idx = 0;
        let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'a') };

        assert_eq!(res, DA_TRUE);
        assert_eq!(suffix_idx, 1);

        let mut suffix_idx2 = 0;
        assert!(tail.walk_char2(1, &mut suffix_idx2, b'a'));
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // walk 'a' to 'p'
        let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'p') };

        assert_eq!(res, DA_TRUE);
        assert_eq!(suffix_idx, 2);

        assert!(tail.walk_char2(1, &mut suffix_idx2, b'p'));
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // walk 'p' to '\0'
        let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'\0') };

        assert_eq!(res, DA_TRUE);
        assert_eq!(suffix_idx, 2);

        assert!(tail.walk_char2(1, &mut suffix_idx2, b'\0'));
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // try walk 'a' to 'b'
        let mut suffix_idx = 1;
        let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'b') };

        assert_eq!(res, DA_FALSE);
        assert_eq!(suffix_idx, 1);

        let mut suffix_idx2 = 1;
        assert!(!tail.walk_char2(1, &mut suffix_idx2, b'b'));
        assert_eq!(suffix_idx as usize, suffix_idx2);
    }
    #[test]
    fn walk_str() {
        let mut tail = Tail::new();
        unsafe {
            tail.add_suffix(b"apa\0".as_ptr());
            tail.add_suffix(b"bad\0".as_ptr());
            assert!(!tail.get_suffix(1).is_null());
        }
        // walk "apa" with (0,"a")
        let mut suffix_idx = 0;
        let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"a".as_ptr(), 1) };

        assert_eq!(res, 1);
        assert_eq!(suffix_idx, 1);
        let mut suffix_idx2 = 0;
        assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"a"), 1);
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // walk "apa" with (0,"ap")
        let mut suffix_idx = 0;
        let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"ap".as_ptr(), 2) };

        assert_eq!(res, 2);
        assert_eq!(suffix_idx, 2);
        let mut suffix_idx2 = 0;
        assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"ap"), 2);
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // walk "apa" with (0,"al")
        let mut suffix_idx = 0;
        let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"al".as_ptr(), 2) };

        assert_eq!(res, 1);
        assert_eq!(suffix_idx, 1);
        let mut suffix_idx2 = 0;
        assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"al"), 1);
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // walk "apa" with (1,"pa")
        let mut suffix_idx = 1;
        let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"pa".as_ptr(), 2) };

        assert_eq!(res, 2);
        assert_eq!(suffix_idx, 3);
        let mut suffix_idx2 = 1;
        assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"pa"), 2);
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // walk "apa" with (1,"la")
        let mut suffix_idx = 1;
        let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"la".as_ptr(), 2) };

        assert_eq!(res, 0);
        assert_eq!(suffix_idx, 1);
        let mut suffix_idx2 = 1;
        assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"la"), 0);
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // walk "apa" with (1,"pap")
        let mut suffix_idx = 1;
        let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"pap".as_ptr(), 2) };

        assert_eq!(res, 2);
        assert_eq!(suffix_idx, 3);
        let mut suffix_idx2 = 1;
        assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"pap"), 2);
        assert_eq!(suffix_idx as usize, suffix_idx2);
        // walk "apa" with (5,"pap")
        let mut suffix_idx = 5;
        let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"pap".as_ptr(), 2) };

        assert_eq!(res, 0);
        assert_eq!(suffix_idx, 5);
        let mut suffix_idx2 = 5;
        assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"pap"), 0);
        assert_eq!(suffix_idx as usize, suffix_idx2);
    }
}
