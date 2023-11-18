use std::io::{self, SeekFrom};

use ::libc;

use crate::{
    fileutils::ReadExt,
    trie_string::{trie_string_append_char, trie_string_cut_last, TrieString},
    DatrieError, DatrieResult,
};

extern "C" {
    fn memmove(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong)
        -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: libc::c_int) -> libc::c_int;
    fn ftell(__stream: *mut FILE) -> libc::c_long;
    fn serialize_int32_be_incr(buff: *mut *mut uint8, val: int32);
    fn file_read_int32(file: *mut FILE, o_val: *mut int32) -> Bool;
    fn file_write_int32(file: *mut FILE, val: int32) -> Bool;
}
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
pub type FILE = libc::FILE;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type uint8 = libc::c_uchar;
pub type uint32 = libc::c_uint;
pub type int32 = libc::c_int;
pub type TrieChar = libc::c_uchar;
pub type TrieIndex = int32;
#[derive(Copy, Clone)]
// #[repr(C)]
pub struct Symbols {
    pub num_symbols: libc::c_short,
    pub symbols: [TrieChar; 256],
}
#[derive(Copy, Clone)]
// #[repr(C)]
pub struct DArray {
    pub num_cells: TrieIndex,
    pub cells: *mut DACell,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DACell {
    pub base: TrieIndex,
    pub check: TrieIndex,
}
impl Symbols {
    fn new() -> Symbols {
        Symbols {
            num_symbols: 0,
            symbols: [0; 256],
        }
    }
    fn add(&mut self, c: TrieChar) {
        let mut lower = 0;
        let mut upper = self.num_symbols as usize;
        while lower < upper {
            let middle = (lower + upper) / 2;
            if c > self.symbols[middle] {
                lower = middle + 1;
            } else if c < self.symbols[middle] {
                upper = middle;
            } else {
                return;
            }
        }
        if lower < self.num_symbols as usize {
            unsafe {
                memmove(
                    self.symbols
                        .as_mut_ptr()
                        .offset(lower as isize)
                        .offset(1 as isize) as *mut libc::c_void,
                    self.symbols.as_mut_ptr().offset(lower as isize) as *const libc::c_void,
                    self.num_symbols as u64 - lower as u64,
                );
            }
        }
        self.symbols[lower] = c;
        self.num_symbols += 1;
    }
}
// unsafe fn symbols_add(syms: *mut Symbols, c: TrieChar) {
//     let mut lower: libc::c_short = 0;
//     let mut upper: libc::c_short = 0;
//     lower = 0 as libc::c_int as libc::c_short;
//     upper = (*syms).num_symbols;
//     while (lower as libc::c_int) < upper as libc::c_int {
//         let mut middle: libc::c_short = 0;
//         middle =
//             ((lower as libc::c_int + upper as libc::c_int) / 2 as libc::c_int) as libc::c_short;
//         if c as libc::c_int > (*syms).symbols[middle as usize] as libc::c_int {
//             lower = (middle as libc::c_int + 1 as libc::c_int) as libc::c_short;
//         } else if (c as libc::c_int) < (*syms).symbols[middle as usize] as libc::c_int {
//             upper = middle;
//         } else {
//             return;
//         }
//     }
//     if (lower as libc::c_int) < (*syms).num_symbols as libc::c_int {
//         memmove(
//             ((*syms).symbols)
//                 .as_mut_ptr()
//                 .offset(lower as libc::c_int as isize)
//                 .offset(1 as libc::c_int as isize) as *mut libc::c_void,
//             ((*syms).symbols)
//                 .as_mut_ptr()
//                 .offset(lower as libc::c_int as isize) as *const libc::c_void,
//             ((*syms).num_symbols as libc::c_int - lower as libc::c_int) as libc::c_ulong,
//         );
//     }
//     (*syms).symbols[lower as usize] = c;
//     (*syms).num_symbols += 1;
// }
impl Symbols {
    pub fn num(&self) -> usize {
        self.num_symbols as usize
    }
    pub fn get(&self, index: usize) -> TrieChar {
        self.symbols[index]
    }
}
impl DArray {
    pub fn new() -> DatrieResult<DArray> {
        let num_cells = 3;
        let cells = unsafe {
            malloc(
                (num_cells as libc::c_ulong)
                    .wrapping_mul(::core::mem::size_of::<DACell>() as libc::c_ulong),
            ) as *mut DACell
        };
        if cells.is_null() {
            return Err(DatrieError::new(
                crate::ErrorKind::Memory,
                "DArray::new malloc failed".into(),
            ));
        }

        unsafe {
            (*cells.offset(0 as libc::c_int as isize)).base =
                0xdafcdafc as libc::c_uint as TrieIndex;
            (*cells.offset(0 as libc::c_int as isize)).check = num_cells;
            (*cells.offset(1 as libc::c_int as isize)).base = -(1 as libc::c_int);
            (*cells.offset(1 as libc::c_int as isize)).check = -(1 as libc::c_int);
            (*cells.offset(2 as libc::c_int as isize)).base = 3 as libc::c_int;
            (*cells.offset(2 as libc::c_int as isize)).check = 0 as libc::c_int;
        }
        Ok(DArray { num_cells, cells })
    }
}
#[no_mangle]
pub unsafe extern "C" fn da_new() -> *mut DArray {
    let d: *mut DArray = malloc(::core::mem::size_of::<DArray>() as libc::c_ulong) as *mut DArray;
    if d.is_null() as libc::c_int as libc::c_long != 0 {
        return 0 as *mut DArray;
    }
    (*d).num_cells = 3 as libc::c_int;
    (*d).cells = malloc(
        ((*d).num_cells as libc::c_ulong)
            .wrapping_mul(::core::mem::size_of::<DACell>() as libc::c_ulong),
    ) as *mut DACell;
    if ((*d).cells).is_null() as libc::c_int as libc::c_long != 0 {
        free(d as *mut libc::c_void);
        return 0 as *mut DArray;
    } else {
        (*((*d).cells).offset(0 as libc::c_int as isize)).base =
            0xdafcdafc as libc::c_uint as TrieIndex;
        (*((*d).cells).offset(0 as libc::c_int as isize)).check = (*d).num_cells;
        (*((*d).cells).offset(1 as libc::c_int as isize)).base = -(1 as libc::c_int);
        (*((*d).cells).offset(1 as libc::c_int as isize)).check = -(1 as libc::c_int);
        (*((*d).cells).offset(2 as libc::c_int as isize)).base = 3 as libc::c_int;
        (*((*d).cells).offset(2 as libc::c_int as isize)).check = 0 as libc::c_int;
        return d;
    };
}
impl DArray {
    pub fn fread_safe<R: ReadExt + io::Seek>(reader: &mut R) -> DatrieResult<*mut DArray> {
        let save_pos = reader.seek(SeekFrom::Current(0))?;
        DArray::do_fread_safe(reader).map_err(|err| {
            if let Err(io_err) = reader.seek(SeekFrom::Start(save_pos)) {
                return io_err.into();
            }
            err
        })
    }
    fn do_fread_safe<R: ReadExt>(reader: &mut R) -> DatrieResult<*mut DArray> {
        let mut current_block: u64;
        let mut save_pos: libc::c_long = 0;
        let mut d: *mut DArray = 0 as *mut DArray;
        let mut n = 0;
        reader.read_uint32(&mut n)?;
        if 0xdafcdafc != n {
            return Err(DatrieError::new(
                crate::ErrorKind::InvalidFileSignature,
                format!("unexpected DArray signature '{}'", n),
            ));
        }
        let d = unsafe { malloc(::core::mem::size_of::<DArray>() as libc::c_ulong) as *mut DArray };
        if !(d.is_null() as libc::c_int as libc::c_long != 0) {
            unsafe {
                // if let Ok(num_cells) = reader.read_int32() {
                if reader.read_int32(&mut (*d).num_cells).is_ok() {
                    // unsafe {
                    //     (*d).num_cells = num_cells;
                    // }
                    if !((*d).num_cells as libc::c_ulong
                        > (18446744073709551615 as libc::c_ulong)
                            .wrapping_div(::core::mem::size_of::<DACell>() as libc::c_ulong))
                    {
                        // unsafe {
                        (*d).cells = malloc(
                            ((*d).num_cells as libc::c_ulong).wrapping_mul(::core::mem::size_of::<
                                DACell,
                            >(
                            )
                                as libc::c_ulong),
                        ) as *mut DACell;
                        // }
                        if !(((*d).cells).is_null() as libc::c_int as libc::c_long != 0) {
                            (*((*d).cells).offset(0 as libc::c_int as isize)).base =
                                0xdafcdafc as libc::c_uint as TrieIndex;
                            (*((*d).cells).offset(0 as libc::c_int as isize)).check =
                                (*d).num_cells;
                            let mut n = 1 as libc::c_int;
                            loop {
                                if !(n < (*d).num_cells) {
                                    current_block = 11050875288958768710;
                                    break;
                                }
                                if reader
                                    .read_int32(&mut (*((*d).cells).offset(n as isize)).base)
                                    .is_err()
                                    || reader
                                        .read_int32(&mut (*((*d).cells).offset(n as isize)).check)
                                        .is_err()
                                {
                                    current_block = 9985172916848320936;
                                    break;
                                }
                                n += 1;
                            }
                            match current_block {
                                11050875288958768710 => return Ok(d),
                                _ => {
                                    free((*d).cells as *mut libc::c_void);
                                }
                            }
                        }
                    }
                }
            }
            unsafe {
                free(d as *mut libc::c_void);
            }
        }
        return Err(DatrieError::new(
            crate::ErrorKind::Bug,
            "reading darray failed".into(),
        ));
    }
}
#[no_mangle]
pub unsafe extern "C" fn da_fread(file: *mut FILE) -> *mut DArray {
    let mut current_block: u64;
    let mut save_pos: libc::c_long = 0;
    let mut d: *mut DArray = 0 as *mut DArray;
    let mut n: TrieIndex = 0;
    save_pos = ftell(file);
    if !(file_read_int32(file, &mut n) as u64 == 0 || 0xdafcdafc as libc::c_uint != n as uint32) {
        d = malloc(::core::mem::size_of::<DArray>() as libc::c_ulong) as *mut DArray;
        if !(d.is_null() as libc::c_int as libc::c_long != 0) {
            if !(file_read_int32(file, &mut (*d).num_cells) as u64 == 0) {
                if !((*d).num_cells as libc::c_ulong
                    > (18446744073709551615 as libc::c_ulong)
                        .wrapping_div(::core::mem::size_of::<DACell>() as libc::c_ulong))
                {
                    (*d).cells = malloc(
                        ((*d).num_cells as libc::c_ulong)
                            .wrapping_mul(::core::mem::size_of::<DACell>() as libc::c_ulong),
                    ) as *mut DACell;
                    if !(((*d).cells).is_null() as libc::c_int as libc::c_long != 0) {
                        (*((*d).cells).offset(0 as libc::c_int as isize)).base =
                            0xdafcdafc as libc::c_uint as TrieIndex;
                        (*((*d).cells).offset(0 as libc::c_int as isize)).check = (*d).num_cells;
                        n = 1 as libc::c_int;
                        loop {
                            if !(n < (*d).num_cells) {
                                current_block = 11050875288958768710;
                                break;
                            }
                            if file_read_int32(file, &mut (*((*d).cells).offset(n as isize)).base)
                                as u64
                                == 0
                                || file_read_int32(
                                    file,
                                    &mut (*((*d).cells).offset(n as isize)).check,
                                ) as u64
                                    == 0
                            {
                                current_block = 9985172916848320936;
                                break;
                            }
                            n += 1;
                        }
                        match current_block {
                            11050875288958768710 => return d,
                            _ => {
                                free((*d).cells as *mut libc::c_void);
                            }
                        }
                    }
                }
            }
            free(d as *mut libc::c_void);
        }
    }
    fseek(file, save_pos, 0 as libc::c_int);
    return 0 as *mut DArray;
}
#[no_mangle]
pub unsafe extern "C" fn da_free(d: *mut DArray) {
    free((*d).cells as *mut libc::c_void);
    free(d as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn da_fwrite(d: *const DArray, file: *mut FILE) -> libc::c_int {
    let mut i: TrieIndex = 0;
    while i < (*d).num_cells {
        if file_write_int32(file, (*((*d).cells).offset(i as isize)).base) as u64 == 0
            || file_write_int32(file, (*((*d).cells).offset(i as isize)).check) as u64 == 0
        {
            return -(1 as libc::c_int);
        }
        i += 1;
    }
    return 0 as libc::c_int;
}
impl DArray {
    pub fn get_serialized_size(&self) -> usize {
        if self.num_cells > 0 {
            return 4 * self.num_cells as usize * 2;
        } else {
            return 0;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn da_serialize(d: *const DArray, ptr: *mut *mut uint8) {
    let mut i: TrieIndex = 0;
    while i < (*d).num_cells {
        serialize_int32_be_incr(ptr, (*((*d).cells).offset(i as isize)).base);
        serialize_int32_be_incr(ptr, (*((*d).cells).offset(i as isize)).check);
        i += 1;
    }
}
impl DArray {
    pub fn get_root(&self) -> TrieIndex {
        return 2 as libc::c_int;
    }
    pub fn get_base(&self, s: TrieIndex) -> TrieIndex {
        return if s < self.num_cells {
            unsafe { (*((self).cells).offset(s as isize)).base }
        } else {
            0 as libc::c_int
        };
    }
    pub fn get_check(&self, s: TrieIndex) -> TrieIndex {
        return if s < self.num_cells {
            unsafe { (*(self.cells).offset(s as isize)).check }
        } else {
            0 as libc::c_int
        };
    }
}
#[no_mangle]
pub unsafe extern "C" fn da_get_root(_d: *const DArray) -> TrieIndex {
    return 2 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn da_get_base(d: *const DArray, s: TrieIndex) -> TrieIndex {
    return if (s < (*d).num_cells) as libc::c_int as libc::c_long != 0 {
        (*((*d).cells).offset(s as isize)).base
    } else {
        0 as libc::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn da_get_check(mut d: *const DArray, mut s: TrieIndex) -> TrieIndex {
    return if (s < (*d).num_cells) as libc::c_int as libc::c_long != 0 {
        (*((*d).cells).offset(s as isize)).check
    } else {
        0 as libc::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn da_set_base(mut d: *mut DArray, mut s: TrieIndex, mut val: TrieIndex) {
    if (s < (*d).num_cells) as libc::c_int as libc::c_long != 0 {
        (*((*d).cells).offset(s as isize)).base = val;
    }
}
#[no_mangle]
pub unsafe extern "C" fn da_set_check(mut d: *mut DArray, mut s: TrieIndex, mut val: TrieIndex) {
    if (s < (*d).num_cells) as libc::c_int as libc::c_long != 0 {
        (*((*d).cells).offset(s as isize)).check = val;
    }
}
#[no_mangle]
pub unsafe extern "C" fn da_walk(
    mut d: *const DArray,
    mut s: *mut TrieIndex,
    mut c: TrieChar,
) -> Bool {
    let mut next: TrieIndex = 0;
    next = da_get_base(d, *s) + c as libc::c_int;
    if da_get_check(d, next) == *s {
        *s = next;
        return DA_TRUE;
    }
    return DA_FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn da_insert_branch(
    mut d: *mut DArray,
    mut s: TrieIndex,
    mut c: TrieChar,
) -> TrieIndex {
    let mut base: TrieIndex = 0;
    let mut next: TrieIndex = 0;
    base = da_get_base(d, s);
    if base > 0 as libc::c_int {
        next = base + c as libc::c_int;
        if da_get_check(d, next) == s {
            return next;
        }
        if base > 0x7fffffff as libc::c_int - c as libc::c_int
            || da_check_free_cell(d, next) as u64 == 0
        {
            // let mut symbols: *mut Symbols = 0 as *mut Symbols;
            let mut new_base: TrieIndex = 0;
            let mut symbols = (*d).output_symbols(s);
            symbols.add(c);
            new_base = da_find_free_base(d, &symbols);
            if (0 as libc::c_int == new_base) as libc::c_int as libc::c_long != 0 {
                return 0 as libc::c_int;
            }
            da_relocate_base(d, s, new_base);
            next = new_base + c as libc::c_int;
        }
    } else {
        // let mut symbols_0: *mut Symbols = 0 as *mut Symbols;
        let mut new_base_0: TrieIndex = 0;
        let mut symbols_0 = Symbols::new();
        symbols_0.add(c);
        new_base_0 = da_find_free_base(d, &symbols_0);
        if (0 as libc::c_int == new_base_0) as libc::c_int as libc::c_long != 0 {
            return 0 as libc::c_int;
        }
        da_set_base(d, s, new_base_0);
        next = new_base_0 + c as libc::c_int;
    }
    da_alloc_cell(d, next);
    da_set_check(d, next, s);
    return next;
}
unsafe extern "C" fn da_check_free_cell(mut d: *mut DArray, mut s: TrieIndex) -> Bool {
    return (da_extend_pool(d, s) as libc::c_uint != 0 && da_get_check(d, s) < 0 as libc::c_int)
        as libc::c_int as Bool;
}
unsafe extern "C" fn da_has_children(mut d: *const DArray, mut s: TrieIndex) -> Bool {
    let mut base: TrieIndex = 0;
    let mut c: TrieIndex = 0;
    let mut max_c: TrieIndex = 0;
    base = da_get_base(d, s);
    if 0 as libc::c_int == base || base < 0 as libc::c_int {
        return DA_FALSE;
    }
    max_c = if (255 as libc::c_int) < (*d).num_cells - base {
        255 as libc::c_int
    } else {
        (*d).num_cells - base
    };
    c = 0 as libc::c_int;
    while c <= max_c {
        if da_get_check(d, base + c) == s {
            return DA_TRUE;
        }
        c += 1;
    }
    return DA_FALSE;
}
impl DArray {
    pub fn output_symbols(&self, s: TrieIndex) -> Symbols {
        let mut syms = Symbols::new();
        let base = self.get_base(s);
        let max_c = if 255 < (self.num_cells - base) {
            255
        } else {
            self.num_cells - base
        };
        for c in 0..=max_c {
            if self.get_check(base + c) == s {
                let fresh0 = syms.num_symbols as usize;
                syms.num_symbols += 1;
                syms.symbols[fresh0] = c as TrieChar;
            }
        }
        return syms;
    }
}

unsafe extern "C" fn da_find_free_base(mut d: *mut DArray, mut symbols: &Symbols) -> TrieIndex {
    let mut first_sym: TrieChar = 0;
    let mut s: TrieIndex = 0;
    first_sym = symbols.get(0 as usize);
    s = -da_get_check(d, 1 as libc::c_int);
    while s != 1 as libc::c_int && s < first_sym as TrieIndex + 3 as libc::c_int {
        s = -da_get_check(d, s);
    }
    if s == 1 as libc::c_int {
        s = first_sym as libc::c_int + 3 as libc::c_int;
        loop {
            if da_extend_pool(d, s) as u64 == 0 {
                return 0 as libc::c_int;
            }
            if da_get_check(d, s) < 0 as libc::c_int {
                break;
            }
            s += 1;
            s;
        }
    }
    while da_fit_symbols(d, s - first_sym as libc::c_int, symbols) as u64 == 0 {
        if -da_get_check(d, s) == 1 as libc::c_int {
            if (da_extend_pool(d, (*d).num_cells) as u64 == 0) as libc::c_int as libc::c_long != 0 {
                return 0 as libc::c_int;
            }
        }
        s = -da_get_check(d, s);
    }
    return s - first_sym as libc::c_int;
}
unsafe extern "C" fn da_fit_symbols(
    mut d: *mut DArray,
    mut base: TrieIndex,
    symbols: &Symbols,
) -> Bool {
    let mut i = 0;
    while i < symbols.num() {
        let mut sym: TrieChar = symbols.get(i);
        if base > 0x7fffffff as libc::c_int - sym as libc::c_int
            || da_check_free_cell(d, base + sym as libc::c_int) as u64 == 0
        {
            return DA_FALSE;
        }
        i += 1;
    }
    return DA_TRUE;
}
unsafe extern "C" fn da_relocate_base(
    mut d: *mut DArray,
    mut s: TrieIndex,
    mut new_base: TrieIndex,
) {
    let mut old_base: TrieIndex = 0;
    // let mut symbols: *mut Symbols = 0 as *mut Symbols;
    // let mut i: libc::c_int = 0;
    old_base = da_get_base(d, s);
    let symbols = (*d).output_symbols(s);
    // i = 0 as libc::c_int;
    let mut i = 0;
    while i < symbols.num() {
        let mut old_next: TrieIndex = 0;
        let mut new_next: TrieIndex = 0;
        let mut old_next_base: TrieIndex = 0;
        old_next = old_base + symbols.get(i) as libc::c_int;
        new_next = new_base + symbols.get(i) as libc::c_int;
        old_next_base = da_get_base(d, old_next);
        da_alloc_cell(d, new_next);
        da_set_check(d, new_next, s);
        da_set_base(d, new_next, old_next_base);
        if old_next_base > 0 as libc::c_int {
            let mut c: TrieIndex = 0;
            let mut max_c: TrieIndex = if (255 as libc::c_int) < (*d).num_cells - old_next_base {
                255 as libc::c_int
            } else {
                (*d).num_cells - old_next_base
            };
            c = 0 as libc::c_int;
            while c <= max_c {
                if da_get_check(d, old_next_base + c) == old_next {
                    da_set_check(d, old_next_base + c, new_next);
                }
                c += 1;
            }
        }
        da_free_cell(d, old_next);
        i += 1;
    }
    da_set_base(d, s, new_base);
}
unsafe extern "C" fn da_extend_pool(d: *mut DArray, to_index: TrieIndex) -> Bool {
    let mut new_block: *mut libc::c_void = 0 as *mut libc::c_void;
    let mut new_begin: TrieIndex = 0;
    let mut i: TrieIndex = 0;
    let mut free_tail: TrieIndex = 0;
    if (to_index <= 0 as libc::c_int || 0x7fffffff as libc::c_int <= to_index) as libc::c_int
        as libc::c_long
        != 0
    {
        return DA_FALSE;
    }
    if to_index < (*d).num_cells {
        return DA_TRUE;
    }
    new_block = realloc(
        (*d).cells as *mut libc::c_void,
        ((to_index + 1 as libc::c_int) as libc::c_ulong)
            .wrapping_mul(::core::mem::size_of::<DACell>() as libc::c_ulong),
    );
    if new_block.is_null() as libc::c_int as libc::c_long != 0 {
        return DA_FALSE;
    }
    (*d).cells = new_block as *mut DACell;
    new_begin = (*d).num_cells;
    (*d).num_cells = to_index + 1 as libc::c_int;
    i = new_begin;
    while i < to_index {
        da_set_check(d, i, -(i + 1 as libc::c_int));
        da_set_base(d, i + 1 as libc::c_int, -i);
        i += 1;
        i;
    }
    free_tail = -da_get_base(d, 1 as libc::c_int);
    da_set_check(d, free_tail, -new_begin);
    da_set_base(d, new_begin, -free_tail);
    da_set_check(d, to_index, -(1 as libc::c_int));
    da_set_base(d, 1 as libc::c_int, -to_index);
    (*((*d).cells).offset(0 as libc::c_int as isize)).check = (*d).num_cells;
    return DA_TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn da_prune(mut d: *mut DArray, mut s: TrieIndex) {
    da_prune_upto(d, da_get_root(d), s);
}
#[no_mangle]
pub unsafe extern "C" fn da_prune_upto(mut d: *mut DArray, mut p: TrieIndex, mut s: TrieIndex) {
    while p != s && da_has_children(d, s) as u64 == 0 {
        let mut parent: TrieIndex = 0;
        parent = da_get_check(d, s);
        da_free_cell(d, s);
        s = parent;
    }
}
unsafe extern "C" fn da_alloc_cell(mut d: *mut DArray, mut cell: TrieIndex) {
    let mut prev: TrieIndex = 0;
    let mut next: TrieIndex = 0;
    prev = -da_get_base(d, cell);
    next = -da_get_check(d, cell);
    da_set_check(d, prev, -next);
    da_set_base(d, next, -prev);
}
unsafe extern "C" fn da_free_cell(d: *mut DArray, cell: TrieIndex) {
    let mut i: TrieIndex = 0;
    let mut prev: TrieIndex = 0;
    i = -da_get_check(d, 1 as libc::c_int);
    while i != 1 as libc::c_int && i < cell {
        i = -da_get_check(d, i);
    }
    prev = -da_get_base(d, i);
    da_set_check(d, cell, -i);
    da_set_base(d, cell, -prev);
    da_set_check(d, prev, -cell);
    da_set_base(d, i, -cell);
}
#[no_mangle]
pub unsafe extern "C" fn da_first_separate(
    d: *mut DArray,
    mut root: TrieIndex,
    keybuff: *mut TrieString,
) -> TrieIndex {
    let mut base: TrieIndex = 0;
    let mut c: TrieIndex = 0;
    let mut max_c: TrieIndex = 0;
    loop {
        base = da_get_base(d, root);
        if !(base >= 0 as libc::c_int) {
            break;
        }
        max_c = if (255 as libc::c_int) < (*d).num_cells - base {
            255 as libc::c_int
        } else {
            (*d).num_cells - base
        };
        c = 0 as libc::c_int;
        while c <= max_c {
            if da_get_check(d, base + c) == root {
                break;
            }
            c += 1;
            c;
        }
        if c > max_c {
            return 0 as libc::c_int;
        }
        trie_string_append_char(keybuff, c as TrieChar);
        root = base + c;
    }
    return root;
}
#[no_mangle]
pub unsafe extern "C" fn da_next_separate(
    mut d: *mut DArray,
    mut root: TrieIndex,
    mut sep: TrieIndex,
    mut keybuff: *mut TrieString,
) -> TrieIndex {
    let mut parent: TrieIndex = 0;
    let mut base: TrieIndex = 0;
    let mut c: TrieIndex = 0;
    let mut max_c: TrieIndex = 0;
    while sep != root {
        parent = da_get_check(d, sep);
        base = da_get_base(d, parent);
        c = sep - base;
        trie_string_cut_last(keybuff);
        max_c = if (255 as libc::c_int) < (*d).num_cells - base {
            255 as libc::c_int
        } else {
            (*d).num_cells - base
        };
        loop {
            c += 1;
            if !(c <= max_c) {
                break;
            }
            if da_get_check(d, base + c) == parent {
                trie_string_append_char(keybuff, c as TrieChar);
                return da_first_separate(d, base + c, keybuff);
            }
        }
        sep = parent;
    }
    return 0 as libc::c_int;
}

#[cfg(test)]
mod tests {
    use crate::DatrieResult;

    use super::DArray;

    #[test]
    fn get_serialized_size_works() -> DatrieResult<()> {
        let da = DArray::new()?;
        // da.add_range(0x00, 0xff)?;
        let size = da.get_serialized_size();
        assert_eq!(size, 24);
        Ok(())
    }
}
