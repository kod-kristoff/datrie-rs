use ::libc;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::SeekFrom;

use crate::{
    fileutils::{ReadExt, ReadSeekExt},
    trie::TrieCharString,
    AlphaStr, DatrieError, DatrieResult, ErrorKind,
};

pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;

pub type AlphaChar = u32;
pub type TrieChar = libc::c_uchar;
pub type TrieIndex = i32;

#[derive(Debug, Default, Clone)]
pub struct AlphaMap {
    ranges: Vec<AlphaRange>,
    alpha_to_trie_map: Vec<TrieIndex>,
    trie_to_alpha_map: Vec<AlphaChar>,
}

#[derive(Copy, Clone, Debug)]
struct AlphaRange {
    pub begin: AlphaChar,
    pub end: AlphaChar,
}
pub unsafe fn alpha_char_strlen(str: *const AlphaChar) -> isize {
    let mut p: *const AlphaChar = str;
    while *p != 0 {
        p = p.offset(1);
    }
    p.offset_from(str)
}
pub unsafe fn alpha_char_strcmp(
    mut str1: *const AlphaChar,
    mut str2: *const AlphaChar,
) -> libc::c_int {
    while *str1 != 0 && *str1 == *str2 {
        str1 = str1.offset(1);
        str2 = str2.offset(1);
    }
    if *str1 < *str2 {
        return -(1 as libc::c_int);
    }
    if *str1 > *str2 {
        return 1 as libc::c_int;
    }
    0 as libc::c_int
}

impl AlphaMap {
    /// Magic number signature for the AlphaMap binary format (0xd9fcd9fc)
    /// Introduced in the initial binary serialization format
    const SIGNATURE: u32 = 0xd9fcd9fc;
    const SIGNATURE_SIZE: usize = 4;
    pub fn fread_bin_safe<R: ReadSeekExt>(reader: &mut R) -> DatrieResult<AlphaMap> {
        let save_pos = reader.stream_position()?;
        AlphaMap::do_fread_bin_safe(reader).map_err(|err| {
            if let Err(io_err) = reader.seek(SeekFrom::Start(save_pos)) {
                return io_err.into();
            }
            err
        })
    }
    fn do_fread_bin_safe<R: ReadExt>(reader: &mut R) -> DatrieResult<AlphaMap> {
        let mut sig = 0;
        reader.read_uint32(&mut sig)?;
        if sig != Self::SIGNATURE {
            return Err(DatrieError::new(
                ErrorKind::InvalidFileSignature,
                format!("Unexpected AlphaMapOld signature: '{}'", sig),
            ));
        }
        let mut alpha_map = AlphaMap::default();
        let mut total = 0;
        reader.read_int32(&mut total)?;

        for _ in 0..total {
            let mut b = 0;
            let mut e = 0;
            reader.read_int32(&mut b)?;
            reader.read_int32(&mut e)?;
            alpha_map.add_range_only(b as AlphaChar, e as AlphaChar)?;
        }
        alpha_map.recalc_work_area();
        Ok(alpha_map)
    }
}

impl AlphaMap {
    pub(crate) fn get_serialized_size(&self) -> usize {
        let ranges_count = self.get_total_ranges();
        Self::SIGNATURE_SIZE
            + ::core::mem::size_of::<i32>()
            + (::core::mem::size_of::<AlphaChar>() * 2 * ranges_count)
    }
    fn get_total_ranges(&self) -> usize {
        self.ranges.len()
    }
    pub(crate) fn serialize(&self, buf: &mut dyn std::io::Write) -> DatrieResult<()> {
        buf.write_i32::<BigEndian>(Self::SIGNATURE as i32)?;
        buf.write_i32::<BigEndian>(self.get_total_ranges() as i32)?;
        for range in &self.ranges {
            buf.write_i32::<BigEndian>(range.begin as i32)?;
            buf.write_i32::<BigEndian>(range.end as i32)?;
        }

        Ok(())
    }
    pub(crate) fn serialize_to_slice(&self, mut buf: &mut [u8]) -> DatrieResult<usize> {
        buf.write_i32::<BigEndian>(Self::SIGNATURE as i32).unwrap();
        buf.write_i32::<BigEndian>(self.get_total_ranges() as i32)?;
        let mut written = 8;
        for range in &self.ranges {
            buf.write_i32::<BigEndian>(range.begin as i32)?;
            buf.write_i32::<BigEndian>(range.end as i32)?;
            written += 8;
        }

        Ok(written)
    }
}

impl AlphaMap {
    pub fn add_range(&mut self, begin: AlphaChar, end: AlphaChar) -> DatrieResult<()> {
        self.add_range_only(begin, end)?;
        // dbg!(&self.ranges);
        self.recalc_work_area();
        Ok(())
    }
    fn add_range_only(&mut self, begin: AlphaChar, end: AlphaChar) -> DatrieResult<()> {
        // dbg!(&begin, &end);
        let mut range_added = false;
        for range in self.ranges.iter_mut() {
            // dbg!(&range);
            if begin <= range.begin && range.end <= end {
                range.begin = begin;
                range.end = end;
                range_added = true;
                break;
            }
            if range.begin <= begin && begin < range.end {
                range.end = end;
                range_added = true;
                break;
            }
            if range.begin <= end && end < range.end {
                range.begin = begin;
                range_added = true;
                break;
            }
            if range.begin == end + 1 {
                range.begin = begin;
                range_added = true;
                break;
            }
            if range.end + 1 == begin {
                range.end = end;
                range_added = true;
                break;
            }
        }
        if !range_added {
            self.ranges.push(AlphaRange { begin, end });
        }
        self.ranges.sort_by(|a, b| a.begin.cmp(&b.begin));

        self.ranges = {
            let mut new_ranges = Vec::new();
            let mut range_opt: Option<AlphaRange> = None;
            for range in &self.ranges {
                // dbg!(&range_opt);
                // dbg!(&range);
                if let Some(mut prev_range) = range_opt.take() {
                    if prev_range.end + 1 < range.begin {
                        new_ranges.push(prev_range);
                        range_opt = Some(*range);
                    } else {
                        prev_range.end = range.end;
                        range_opt = Some(prev_range);
                    }
                } else {
                    range_opt = Some(*range);
                }
            }
            if let Some(range) = range_opt.take() {
                new_ranges.push(range);
            }
            new_ranges
        };
        Ok(())
    }
    fn recalc_work_area(&mut self) {
        let mut n_trie = self.ranges.iter().fold(0u32, |n, x| {
            n.wrapping_add(x.end.wrapping_sub(x.begin).wrapping_add(1))
        });
        n_trie += 1;
        // dbg!(&n_trie);
        let alpha_begin = self.ranges[0].begin;
        let alpha_end = self.ranges[self.ranges.len() - 1].end;
        let n_alpha = alpha_end.wrapping_sub(alpha_begin).wrapping_add(1);
        // dbg!(&n_alpha);
        self.alpha_to_trie_map = vec![Self::ERROR_CHAR; n_alpha as usize];
        self.trie_to_alpha_map = vec![Self::ERROR_CHAR as u32; n_trie as usize];
        let mut trie_char = 0;
        for range in &self.ranges {
            let mut a = range.begin;
            while a <= range.end {
                if trie_char == 0 {
                    trie_char += 1;
                }
                self.alpha_to_trie_map[a.wrapping_sub(alpha_begin) as usize] = trie_char;
                self.trie_to_alpha_map[trie_char as usize] = a;
                trie_char += 1;
                a += 1;
            }
        }
        while trie_char < n_trie as i32 {
            self.trie_to_alpha_map[trie_char as usize] = 1;
            trie_char += 1;
        }
        self.trie_to_alpha_map[0] = 0;
    }
    const ERROR_CHAR: TrieIndex = 0x7fffffff;
    pub(crate) fn char_to_trie(&self, ac: AlphaChar) -> Option<TrieIndex> {
        // dbg!(&ac);
        if ac == 0 {
            return Some(0);
        }
        let alpha_begin = self.ranges[0].begin;
        // dbg!(&alpha_begin);
        let alpha_end = self.ranges[self.ranges.len() - 1].end;
        // dbg!(&alpha_end);
        if alpha_begin <= ac && ac <= alpha_end {
            // dbg!(&self.alpha_to_trie_map);
            return Some(self.alpha_to_trie_map[ac.wrapping_sub(alpha_begin) as usize]);
        }
        None
    }
}

impl AlphaMap {
    pub(crate) fn trie_to_char(&self, tc: TrieChar) -> AlphaChar {
        if (tc as usize) < self.trie_to_alpha_map.len() {
            return self.trie_to_alpha_map[tc as usize];
        }
        !(0 as AlphaChar)
    }
    pub(crate) fn trie_to_char2(&self, tc: TrieChar) -> Option<AlphaChar> {
        self.trie_to_alpha_map.get(tc as usize).copied()
    }
    pub(crate) unsafe fn char_to_trie_str(&self, mut str: *const AlphaChar) -> *mut TrieChar {
        let current_block: u64;
        let trie_str = libc::malloc((alpha_char_strlen(str) + 1) as usize) as *mut TrieChar;
        if trie_str.is_null() as libc::c_int as libc::c_long != 0 {
            return std::ptr::null_mut::<TrieChar>();
        }
        let mut p = trie_str;
        loop {
            if *str == 0 {
                current_block = 4906268039856690917;
                break;
            }
            if let Some(tc) = self.char_to_trie(*str) {
                *p = tc as TrieChar;
                dbg!(*p);
                p = p.offset(1);
                str = str.offset(1);
            } else {
                current_block = 13430631152357385211;
                break;
            }
        }
        match current_block {
            13430631152357385211 => {
                libc::free(trie_str as *mut libc::c_void);
                std::ptr::null_mut::<TrieChar>()
            }
            _ => {
                *p = '\0' as i32 as TrieChar;
                trie_str
            }
        }
    }
    pub(crate) fn char_to_trie_str2(&self, str: &AlphaStr) -> Option<TrieCharString> {
        let mut buf = Vec::with_capacity(str.count_slice() + 1);
        dbg!(str);
        let mut str = str.to_slice_with_nul();
        loop {
            dbg!(str[0]);
            if str[0] == 0 {
                break;
            }
            if let Some(tc) = self.char_to_trie(str[0]) {
                debug_assert_ne!(tc, 0);
                dbg!(&tc);
                let tc = tc as TrieChar;
                dbg!(&tc);
                // debug_assert_ne!(tc, 0);
                if tc != 0 {
                    buf.push(tc as TrieChar);
                } else {
                    eprintln!("ignoring tc==0 ...");
                }
                str = &str[1..];
            }
        }
        // TODO: use from_vec_unchecked?
        dbg!(&buf);
        match TrieCharString::new(buf) {
            Ok(str) => Some(str),
            Err(err) => {
                eprintln!("alpha_map:char_to_trie_str2: failed create TrieCharString: {err:?}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests;
