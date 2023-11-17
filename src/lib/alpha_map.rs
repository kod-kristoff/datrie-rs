use ::libc;
use std::{fs, io::SeekFrom, ptr};

use crate::{
    fileutils::{ReadExt, ReadSeekExt},
    DatrieError, DatrieResult, ErrorKind,
};

extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: libc::c_int) -> libc::c_int;
    fn ftell(__stream: *mut FILE) -> libc::c_long;
    fn serialize_int32_be_incr(buff: *mut *mut uint8, val: int32);
    fn file_read_int32(file: *mut FILE, o_val: *mut int32) -> Bool;
    fn file_write_int32(file: *mut FILE, val: int32) -> Bool;
    fn trie_char_strlen(str: *const TrieChar) -> size_t;
}
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
pub type size_t = libc::c_ulong;

pub type FILE = libc::FILE;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type uint8 = libc::c_uchar;
pub type uint32 = libc::c_uint;
pub type int32 = libc::c_int;
pub type AlphaChar = uint32;
pub type TrieChar = libc::c_uchar;
pub type TrieIndex = int32;
// #[derive(Copy, Clone)]
// #[repr(C)]
#[derive(Debug)]
pub struct AlphaMap {
    pub first_range: *mut AlphaRange,
    pub alpha_begin: AlphaChar,
    pub alpha_end: AlphaChar,
    pub alpha_map_sz: libc::c_int,
    pub alpha_to_trie_map: *mut TrieIndex,
    pub trie_map_sz: libc::c_int,
    pub trie_to_alpha_map: *mut AlphaChar,
}
#[derive(Copy, Clone, Debug)]
// #[repr(C)]
pub struct AlphaRange {
    pub next: *mut AlphaRange,
    pub begin: AlphaChar,
    pub end: AlphaChar,
}
pub unsafe fn alpha_char_strlen(str: *const AlphaChar) -> libc::c_int {
    let mut p: *const AlphaChar = str;
    while *p != 0 {
        p = p.offset(1);
    }
    return p.offset_from(str) as libc::c_long as libc::c_int;
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
    return 0 as libc::c_int;
}
impl AlphaMap {
    pub fn new() -> AlphaMap {
        AlphaMap {
            first_range: ptr::null_mut(),
            alpha_begin: 0,
            alpha_end: 0,
            alpha_map_sz: 0,
            alpha_to_trie_map: ptr::null_mut(),
            trie_map_sz: 0,
            trie_to_alpha_map: ptr::null_mut(),
        }
    }
}

impl Clone for AlphaMap {
    fn clone(&self) -> AlphaMap {
        unsafe {
            let mut current_block: u64;
            let mut range: *mut AlphaRange = 0 as *mut AlphaRange;
            let mut alpha_map = AlphaMap::new();
            range = (*self).first_range;
            loop {
                if range.is_null() {
                    current_block = 15619007995458559411;
                    break;
                }
                if alpha_map_add_range_only(&mut alpha_map, (*range).begin, (*range).end)
                    != 0 as libc::c_int
                {
                    current_block = 7638541459528378975;
                    break;
                }
                range = (*range).next;
            }
            match current_block {
                15619007995458559411 => {
                    if !(alpha_map_recalc_work_area(&mut alpha_map) != 0 as libc::c_int) {
                        return alpha_map;
                    }
                }
                _ => {}
            }
            // alpha_map_free(alpha_map);
            // return 0 as *mut AlphaMap;
            todo!("handle failure")
        }
    }
}
impl Drop for AlphaMap {
    fn drop(&mut self) {
        unsafe {
            let mut p: *mut AlphaRange = 0 as *mut AlphaRange;
            let mut q: *mut AlphaRange = 0 as *mut AlphaRange;
            p = (*self).first_range;
            while !p.is_null() {
                q = (*p).next;
                free(p as *mut libc::c_void);
                p = q;
            }
            if !((*self).alpha_to_trie_map).is_null() {
                free((*self).alpha_to_trie_map as *mut libc::c_void);
            }
            if !((*self).trie_to_alpha_map).is_null() {
                free((*self).trie_to_alpha_map as *mut libc::c_void);
            }
            // free(alpha_map as *mut libc::c_void);
        }
    }
}
impl AlphaMap {
    pub unsafe fn fread_bin(mut file: *mut FILE) -> DatrieResult<AlphaMap> {
        let mut current_block: u64;
        let mut save_pos: libc::c_long = 0;
        let mut sig: uint32 = 0;
        let mut total: int32 = 0;
        let mut i: int32 = 0;
        let mut alpha_map: AlphaMap;
        save_pos = ftell(file);
        if !(file_read_int32(file, &mut sig as *mut uint32 as *mut int32) as u64 == 0
            || 0xd9fcd9fc as libc::c_uint != sig)
        {
            alpha_map = AlphaMap::new();
            // if !(alpha_map.is_null() as libc::c_int as libc::c_long != 0) {
            if !(file_read_int32(file, &mut total) as u64 == 0) {
                i = 0 as libc::c_int;
                loop {
                    if !(i < total) {
                        current_block = 1917311967535052937;
                        break;
                    }
                    let mut b: int32 = 0;
                    let mut e: int32 = 0;
                    if file_read_int32(file, &mut b) as u64 == 0
                        || file_read_int32(file, &mut e) as u64 == 0
                    {
                        current_block = 10306619946931033911;
                        break;
                    }
                    alpha_map_add_range_only(&mut alpha_map, b as AlphaChar, e as AlphaChar);
                    i += 1;
                    i;
                }
                match current_block {
                    10306619946931033911 => {}
                    _ => {
                        if !((alpha_map_recalc_work_area(&mut alpha_map) != 0 as libc::c_int)
                            as libc::c_int as libc::c_long
                            != 0)
                        {
                            return Ok(alpha_map);
                        }
                    }
                }
            }
            // alpha_map_free(alpha_map);
            // }
        }
        fseek(file, save_pos, 0 as libc::c_int);
        return Err(DatrieError::new(
            ErrorKind::Io,
            "failed to load file".into(),
        ));
        // return 0 as *mut AlphaMap;
    }
    pub fn fread_bin_safe<R: ReadSeekExt>(reader: &mut R) -> DatrieResult<AlphaMap> {
        let save_pos = reader.seek(SeekFrom::Current(0))?;
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
        if sig != 0xd9fcd9fc {
            return Err(DatrieError::new(
                ErrorKind::InvalidFileSignature,
                format!("Unexpected AlphaMap signature: '{}'", sig),
            ));
        }
        let mut alpha_map = AlphaMap::new();
        let mut total = 0;
        reader.read_int32(&mut total)?;

        for _ in 0..total {
            let mut b = 0;
            let mut e = 0;
            reader.read_int32(&mut b)?;
            reader.read_int32(&mut e)?;
            alpha_map.add_range_only_safe(b as AlphaChar, e as AlphaChar)?;
        }
        alpha_map.recalc_work_area_safe()?;
        Ok(alpha_map)
    }

    fn add_range_only_safe(&mut self, b: AlphaChar, e: AlphaChar) -> DatrieResult<()> {
        let res = unsafe { alpha_map_add_range_only(self as *mut AlphaMap, b, e) };

        if res != 0 as libc::c_int {
            return Err(DatrieError::new(
                ErrorKind::Bug,
                format!("add_range_only returned '{res}'"),
            ));
        }
        Ok(())
    }
    fn recalc_work_area_safe(&mut self) -> DatrieResult<()> {
        let res = unsafe { alpha_map_recalc_work_area(self as *mut AlphaMap) };

        if res != 0 as libc::c_int {
            return Err(DatrieError::new(
                ErrorKind::Bug,
                format!("recalc_work_area returned '{res}'"),
            ));
        }
        Ok(())
    }
}

pub unsafe fn alpha_map_fwrite_bin(alpha_map: *const AlphaMap, file: *mut FILE) -> libc::c_int {
    let mut range: *mut AlphaRange = 0 as *mut AlphaRange;
    if file_write_int32(file, 0xd9fcd9fc as libc::c_uint as int32) as u64 == 0
        || file_write_int32(file, AlphaMap::get_total_ranges(&*alpha_map) as i32) as u64 == 0
    {
        return -(1 as libc::c_int);
    }
    range = (*alpha_map).first_range;
    while !range.is_null() {
        if file_write_int32(file, (*range).begin as int32) as u64 == 0
            || file_write_int32(file, (*range).end as int32) as u64 == 0
        {
            return -(1 as libc::c_int);
        }
        range = (*range).next;
    }
    return 0 as libc::c_int;
}
impl AlphaMap {
    const SIGNATURE_SIZE: usize = 4;
    pub fn get_serialized_size(&self) -> usize {
        let ranges_count = self.get_total_ranges();
        return Self::SIGNATURE_SIZE
            + ::core::mem::size_of::<int32>()
            + (::core::mem::size_of::<AlphaChar>() * 2 * ranges_count);
    }
    fn get_total_ranges(&self) -> usize {
        let mut n = 0;
        let mut range = self.first_range;
        while !range.is_null() {
            n += 1;
            range = unsafe { (*range).next };
        }
        return n;
    }
}

pub unsafe fn alpha_map_serialize_bin(alpha_map: *const AlphaMap, mut ptr: *mut *mut uint8) {
    let mut range: *mut AlphaRange = 0 as *mut AlphaRange;
    serialize_int32_be_incr(ptr, 0xd9fcd9fc as libc::c_uint as int32);
    serialize_int32_be_incr(ptr, AlphaMap::get_total_ranges(&*alpha_map) as i32);
    range = (*alpha_map).first_range;
    while !range.is_null() {
        serialize_int32_be_incr(ptr, (*range).begin as int32);
        serialize_int32_be_incr(ptr, (*range).end as int32);
        range = (*range).next;
    }
}
unsafe fn alpha_map_add_range_only(
    mut alpha_map: *mut AlphaMap,
    mut begin: AlphaChar,
    mut end: AlphaChar,
) -> libc::c_int {
    let mut q: *mut AlphaRange = 0 as *mut AlphaRange;
    let mut r: *mut AlphaRange = 0 as *mut AlphaRange;
    let mut begin_node: *mut AlphaRange = 0 as *mut AlphaRange;
    let mut end_node: *mut AlphaRange = 0 as *mut AlphaRange;
    if begin > end {
        return -(1 as libc::c_int);
    }
    end_node = 0 as *mut AlphaRange;
    begin_node = end_node;
    q = 0 as *mut AlphaRange;
    r = (*alpha_map).first_range;
    while !r.is_null() && (*r).begin <= begin {
        if begin <= (*r).end {
            begin_node = r;
            break;
        } else if ((*r).end).wrapping_add(1 as libc::c_int as libc::c_uint) == begin {
            (*r).end = begin;
            begin_node = r;
            break;
        } else {
            q = r;
            r = (*r).next;
        }
    }
    if begin_node.is_null()
        && !r.is_null()
        && (*r).begin <= end.wrapping_add(1 as libc::c_int as libc::c_uint)
    {
        (*r).begin = begin;
        begin_node = r;
    }
    while !r.is_null() && (*r).begin <= end.wrapping_add(1 as libc::c_int as libc::c_uint) {
        if end <= (*r).end {
            end_node = r;
        } else if r != begin_node {
            if !q.is_null() {
                (*q).next = (*r).next;
                free(r as *mut libc::c_void);
                r = (*q).next;
            } else {
                (*alpha_map).first_range = (*r).next;
                free(r as *mut libc::c_void);
                r = (*alpha_map).first_range;
            }
            continue;
        }
        q = r;
        r = (*r).next;
    }
    if end_node.is_null() && !q.is_null() && begin <= (*q).end {
        (*q).end = end;
        end_node = q;
    }
    if !begin_node.is_null() && !end_node.is_null() {
        if begin_node != end_node {
            if (*begin_node).next == end_node {
            } else {
                panic!(
                    "begin_node->next == end_node" // b"alpha-map.c\0" as *const u8 as *const libc::c_char,
                                                   // 396 as libc::c_int as libc::c_uint,
                                                   // (*::core::mem::transmute::<&[u8; 63], &[libc::c_char; 63]>(
                                                   //     b"int alpha_map_add_range_only(AlphaMap *, AlphaChar, AlphaChar)\0",
                                                   // ))
                                                   // .as_ptr(),
                );
                // __assert_fail(
                //     b"begin_node->next == end_node\0" as *const u8 as *const libc::c_char,
                //     b"alpha-map.c\0" as *const u8 as *const libc::c_char,
                //     396 as libc::c_int as libc::c_uint,
                //     (*::core::mem::transmute::<&[u8; 63], &[libc::c_char; 63]>(
                //         b"int alpha_map_add_range_only(AlphaMap *, AlphaChar, AlphaChar)\0",
                //     ))
                //     .as_ptr(),
                // );
            }
            'c_3243: {
                if (*begin_node).next == end_node {
                } else {
                    panic!(
                        "begin_node->next == end_node" // b"alpha-map.c\0" as *const u8 as *const libc::c_char,
                                                       // 396 as libc::c_int as libc::c_uint,
                                                       // (*::core::mem::transmute::<&[u8; 63], &[libc::c_char; 63]>(
                                                       //     b"int alpha_map_add_range_only(AlphaMap *, AlphaChar, AlphaChar)\0",
                                                       // ))
                                                       // .as_ptr(),
                    );
                    // __assert_fail(
                    //     b"begin_node->next == end_node\0" as *const u8 as *const libc::c_char,
                    //     b"alpha-map.c\0" as *const u8 as *const libc::c_char,
                    //     396 as libc::c_int as libc::c_uint,
                    //     (*::core::mem::transmute::<&[u8; 63], &[libc::c_char; 63]>(
                    //         b"int alpha_map_add_range_only(AlphaMap *, AlphaChar, AlphaChar)\0",
                    //     ))
                    //     .as_ptr(),
                    // );
                }
            };
            (*begin_node).end = (*end_node).end;
            (*begin_node).next = (*end_node).next;
            free(end_node as *mut libc::c_void);
        }
    } else if begin_node.is_null() && end_node.is_null() {
        let mut range: *mut AlphaRange =
            malloc(::core::mem::size_of::<AlphaRange>() as libc::c_ulong) as *mut AlphaRange;
        if range.is_null() as libc::c_int as libc::c_long != 0 {
            return -(1 as libc::c_int);
        }
        (*range).begin = begin;
        (*range).end = end;
        if !q.is_null() {
            (*q).next = range;
        } else {
            (*alpha_map).first_range = range;
        }
        (*range).next = r;
    }
    return 0 as libc::c_int;
}
unsafe fn alpha_map_recalc_work_area(mut alpha_map: *mut AlphaMap) -> libc::c_int {
    let mut current_block: u64;
    let mut range: *mut AlphaRange = 0 as *mut AlphaRange;
    if !((*alpha_map).alpha_to_trie_map).is_null() {
        free((*alpha_map).alpha_to_trie_map as *mut libc::c_void);
        (*alpha_map).alpha_to_trie_map = 0 as *mut TrieIndex;
    }
    if !((*alpha_map).trie_to_alpha_map).is_null() {
        free((*alpha_map).trie_to_alpha_map as *mut libc::c_void);
        (*alpha_map).trie_to_alpha_map = 0 as *mut AlphaChar;
    }
    range = (*alpha_map).first_range;
    if !range.is_null() {
        let alpha_begin: AlphaChar = (*range).begin;
        let mut n_alpha: libc::c_int = 0;
        let mut n_trie: libc::c_int = 0;
        let mut i: libc::c_int = 0;
        let mut a: AlphaChar = 0;
        let mut trie_char: TrieIndex = 0;
        (*alpha_map).alpha_begin = alpha_begin;
        n_trie = 0 as libc::c_int;
        loop {
            n_trie = (n_trie as libc::c_uint).wrapping_add(
                ((*range).end)
                    .wrapping_sub((*range).begin)
                    .wrapping_add(1 as libc::c_int as libc::c_uint),
            ) as libc::c_int as libc::c_int;
            if ((*range).next).is_null() {
                break;
            }
            range = (*range).next;
        }
        if n_trie < '\0' as i32 {
            n_trie = '\0' as i32 + 1 as libc::c_int;
        } else {
            n_trie += 1;
            n_trie;
        }
        (*alpha_map).alpha_end = (*range).end;
        n_alpha = ((*range).end)
            .wrapping_sub(alpha_begin)
            .wrapping_add(1 as libc::c_int as libc::c_uint) as libc::c_int;
        (*alpha_map).alpha_map_sz = n_alpha;
        (*alpha_map).alpha_to_trie_map = malloc(
            (n_alpha as libc::c_ulong)
                .wrapping_mul(::core::mem::size_of::<TrieIndex>() as libc::c_ulong),
        ) as *mut TrieIndex;
        if ((*alpha_map).alpha_to_trie_map).is_null() as libc::c_int as libc::c_long != 0 {
            current_block = 12045534306736471162;
        } else {
            i = 0 as libc::c_int;
            while i < n_alpha {
                *((*alpha_map).alpha_to_trie_map).offset(i as isize) = 0x7fffffff as libc::c_int;
                i += 1;
                i;
            }
            (*alpha_map).trie_map_sz = n_trie;
            (*alpha_map).trie_to_alpha_map = malloc(
                (n_trie as libc::c_ulong)
                    .wrapping_mul(::core::mem::size_of::<AlphaChar>() as libc::c_ulong),
            ) as *mut AlphaChar;
            if ((*alpha_map).trie_to_alpha_map).is_null() as libc::c_int as libc::c_long != 0 {
                free((*alpha_map).alpha_to_trie_map as *mut libc::c_void);
                (*alpha_map).alpha_to_trie_map = 0 as *mut TrieIndex;
                current_block = 12045534306736471162;
            } else {
                trie_char = 0 as libc::c_int;
                range = (*alpha_map).first_range;
                while !range.is_null() {
                    a = (*range).begin;
                    while a <= (*range).end {
                        if '\0' as i32 == trie_char {
                            trie_char += 1;
                            trie_char;
                        }
                        *((*alpha_map).alpha_to_trie_map)
                            .offset(a.wrapping_sub(alpha_begin) as isize) = trie_char;
                        *((*alpha_map).trie_to_alpha_map).offset(trie_char as isize) = a;
                        trie_char += 1;
                        trie_char;
                        a = a.wrapping_add(1);
                        a;
                    }
                    range = (*range).next;
                }
                while trie_char < n_trie {
                    let fresh0 = trie_char;
                    trie_char = trie_char + 1;
                    *((*alpha_map).trie_to_alpha_map).offset(fresh0 as isize) =
                        !(0 as libc::c_int as AlphaChar);
                }
                *((*alpha_map).trie_to_alpha_map).offset('\0' as i32 as isize) =
                    0 as libc::c_int as AlphaChar;
                current_block = 572715077006366937;
            }
        }
        match current_block {
            572715077006366937 => {}
            _ => return -(1 as libc::c_int),
        }
    }
    return 0 as libc::c_int;
}

pub unsafe fn alpha_map_add_range(
    mut alpha_map: *mut AlphaMap,
    mut begin: AlphaChar,
    mut end: AlphaChar,
) -> libc::c_int {
    let mut res: libc::c_int = alpha_map_add_range_only(alpha_map, begin, end);
    if res != 0 as libc::c_int {
        return res;
    }
    return alpha_map_recalc_work_area(alpha_map);
}
impl AlphaMap {
    pub fn add_range(
        &mut self,
        // mut alpha_map: *mut AlphaMap,
        begin: AlphaChar,
        end: AlphaChar,
    ) -> DatrieResult<()> {
        self.add_range_only_safe(begin, end)?;
        self.recalc_work_area_safe()?;
        Ok(())
    }
}
pub unsafe fn alpha_map_char_to_trie(
    mut alpha_map: *const AlphaMap,
    mut ac: AlphaChar,
) -> TrieIndex {
    let mut alpha_begin: TrieIndex = 0;
    if (0 as libc::c_int as libc::c_uint == ac) as libc::c_int as libc::c_long != 0 {
        return '\0' as i32;
    }
    if ((*alpha_map).alpha_to_trie_map).is_null() as libc::c_int as libc::c_long != 0 {
        return 0x7fffffff as libc::c_int;
    }
    alpha_begin = (*alpha_map).alpha_begin as TrieIndex;
    if alpha_begin as libc::c_uint <= ac && ac <= (*alpha_map).alpha_end {
        return *((*alpha_map).alpha_to_trie_map)
            .offset(ac.wrapping_sub(alpha_begin as libc::c_uint) as isize);
    }
    return 0x7fffffff as libc::c_int;
}

pub unsafe fn alpha_map_trie_to_char(
    mut alpha_map: *const AlphaMap,
    mut tc: TrieChar,
) -> AlphaChar {
    if (tc as libc::c_int) < (*alpha_map).trie_map_sz {
        return *((*alpha_map).trie_to_alpha_map).offset(tc as isize);
    }
    return !(0 as libc::c_int as AlphaChar);
}

pub unsafe fn alpha_map_char_to_trie_str(
    mut alpha_map: *const AlphaMap,
    mut str: *const AlphaChar,
) -> *mut TrieChar {
    let mut current_block: u64;
    let mut trie_str: *mut TrieChar = 0 as *mut TrieChar;
    let mut p: *mut TrieChar = 0 as *mut TrieChar;
    trie_str =
        malloc((alpha_char_strlen(str) + 1 as libc::c_int) as libc::c_ulong) as *mut TrieChar;
    if trie_str.is_null() as libc::c_int as libc::c_long != 0 {
        return 0 as *mut TrieChar;
    }
    p = trie_str;
    loop {
        if !(*str != 0) {
            current_block = 4906268039856690917;
            break;
        }
        let mut tc: TrieIndex = alpha_map_char_to_trie(alpha_map, *str);
        if 0x7fffffff as libc::c_int == tc {
            current_block = 13430631152357385211;
            break;
        }
        *p = tc as TrieChar;
        p = p.offset(1);
        p;
        str = str.offset(1);
        str;
    }
    match current_block {
        13430631152357385211 => {
            free(trie_str as *mut libc::c_void);
            return 0 as *mut TrieChar;
        }
        _ => {
            *p = '\0' as i32 as TrieChar;
            return trie_str;
        }
    };
}

pub unsafe fn alpha_map_trie_to_char_str(
    alpha_map: *const AlphaMap,
    mut str: *const TrieChar,
) -> *mut AlphaChar {
    let mut alpha_str: *mut AlphaChar = 0 as *mut AlphaChar;
    let mut p: *mut AlphaChar = 0 as *mut AlphaChar;
    alpha_str = malloc(
        (trie_char_strlen(str))
            .wrapping_add(1 as libc::c_int as libc::c_ulong)
            .wrapping_mul(::core::mem::size_of::<AlphaChar>() as libc::c_ulong),
    ) as *mut AlphaChar;
    if alpha_str.is_null() as libc::c_int as libc::c_long != 0 {
        return 0 as *mut AlphaChar;
    }
    p = alpha_str;
    while *str != 0 {
        *p = alpha_map_trie_to_char(alpha_map, *str);
        p = p.offset(1);
        str = str.offset(1);
    }
    *p = 0 as libc::c_int as AlphaChar;
    return alpha_str;
}

#[cfg(test)]
mod tests {
    use crate::DatrieResult;

    use super::AlphaMap;

    #[test]
    fn get_serialized_size_works() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x00, 0xff)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }

    #[test]
    fn get_total_ranges_works() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x00, 0xff)?;
        let size = alpha_map.get_total_ranges();
        assert_eq!(size, 1);
        Ok(())
    }
}
