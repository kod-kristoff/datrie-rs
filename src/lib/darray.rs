use std::io::{self, SeekFrom};

use ::libc;
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use core::mem::size_of;

use crate::{
    fileutils::{CFile, ReadExt},
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
#[derive(Clone, Debug)]
// #[repr(C)]
pub struct DArray {
    pub num_cells: TrieIndex,
    pub cells: *mut DACell,
    pub cells2: Vec<DACell>,
}
#[derive(Copy, Clone, Debug)]
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
impl Symbols {
    pub fn num(&self) -> usize {
        self.num_symbols as usize
    }
    pub fn get(&self, index: usize) -> TrieChar {
        self.symbols[index]
    }
}
impl DArray {
    const SIGNATURE: u32 = 0xdafcdafc;

    pub fn new() -> DatrieResult<DArray> {
        let num_cells = 3;
        let cells2 = vec![
            DACell {
                base: Self::SIGNATURE as TrieIndex,
                check: num_cells,
            },
            DACell {
                base: -1,
                check: -1,
            },
            DACell { base: 3, check: 0 },
        ];
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
        Ok(DArray {
            num_cells,
            cells,
            cells2,
        })
    }
}
impl Drop for DArray {
    fn drop(&mut self) {
        unsafe {
            free(self.cells as *mut libc::c_void);
        }
    }
}
impl DArray {
    pub fn fread_safe<R: ReadExt + io::Seek>(reader: &mut R) -> DatrieResult<DArray> {
        let save_pos = reader.seek(SeekFrom::Current(0))?;
        DArray::do_fread_safe(reader).map_err(|err| {
            if let Err(io_err) = reader.seek(SeekFrom::Start(save_pos)) {
                return io_err.into();
            }
            err
        })
    }
    fn do_fread_safe<R: ReadExt>(reader: &mut R) -> DatrieResult<DArray> {
        let mut current_block: u64;
        let mut save_pos: libc::c_long = 0;
        // let mut d: *mut DArray = 0 as *mut DArray;
        let mut n = 0;
        reader.read_uint32(&mut n)?;
        if 0xdafcdafc != n {
            return Err(DatrieError::new(
                crate::ErrorKind::InvalidFileSignature,
                format!("unexpected DArray signature '{}'", n),
            ));
        }
        // let d = unsafe { malloc(::core::mem::size_of::<DArray>() as libc::c_ulong) as *mut DArray };
        // if !(d.is_null() as libc::c_int as libc::c_long != 0) {
        let mut num_cells = 0;
        reader.read_int32(&mut num_cells)?;
        // unsafe {
        // if let Ok(num_cells) = reader.read_int32() {
        // if reader.read_int32(&mut (*d).num_cells).is_ok() {
        // unsafe {
        //     (*d).num_cells = num_cells;
        // }
        if num_cells as libc::c_ulong
            > (18446744073709551615 as libc::c_ulong)
                .wrapping_div(::core::mem::size_of::<DACell>() as libc::c_ulong)
        {
            return Err(DatrieError::new(
                crate::ErrorKind::Bug,
                "reading darray failed: num_cells to big".into(),
            ));
        }
        // unsafe {
        let mut cells2 = Vec::with_capacity(num_cells as usize);
        let cells = unsafe {
            malloc(
                (num_cells as libc::c_ulong)
                    .wrapping_mul(::core::mem::size_of::<DACell>() as libc::c_ulong),
            ) as *mut DACell
        };
        // }
        if !cells.is_null() {
            unsafe {
                (*cells.offset(0 as libc::c_int as isize)).base =
                    0xdafcdafc as libc::c_uint as TrieIndex;
                (*cells.offset(0 as libc::c_int as isize)).check = num_cells;
            }
            cells2.push(DACell {
                base: Self::SIGNATURE as TrieIndex,
                check: num_cells,
            });
            let mut n = 1isize;
            loop {
                if !(n < num_cells as isize) {
                    current_block = 11050875288958768710;
                    break;
                }
                let mut base = 0;
                let mut check = 0;
                if reader.read_int32(&mut base).is_err() || reader.read_int32(&mut check).is_err() {
                    current_block = 9985172916848320936;
                    break;
                }
                unsafe {
                    (*cells.offset(n)).base = base;
                    (*cells.offset(n)).check = check;
                }
                cells2.push(DACell { base, check });
                n += 1;
            }
            match current_block {
                11050875288958768710 => {
                    return Ok(DArray {
                        num_cells,
                        cells,
                        cells2,
                    })
                }
                _ => unsafe {
                    free(cells as *mut libc::c_void);
                },
            }
        }
        // }
        //     }
        // }
        // unsafe {
        //     free(d as *mut libc::c_void);
        // }
        // }
        return Err(DatrieError::new(
            crate::ErrorKind::Bug,
            "reading darray failed".into(),
        ));
    }

    pub unsafe fn fread(file: *mut FILE) -> DatrieResult<DArray> {
        // let mut current_block: u64;
        // let mut save_pos: libc::c_long = 0;
        // let mut d: *mut DArray = 0 as *mut DArray;
        // let mut n: TrieIndex = 0;
        let save_pos = ftell(file);
        let mut cfile = CFile::new(file);
        match Self::do_fread(&mut cfile) {
            Ok(da) => Ok(da),
            Err(err) => {
                unsafe {
                    libc::fseek(file, save_pos, libc::SEEK_SET);
                }
                Err(err)
            }
        }
    }
    fn do_fread<R: ReadExt>(reader: &mut R) -> DatrieResult<DArray> {
        let mut current_block: u64;
        let mut save_pos: libc::c_long = 0;
        // let mut d: *mut DArray = 0 as *mut DArray;
        let mut n = 0;
        reader.read_uint32(&mut n)?;
        if Self::SIGNATURE != n {
            return Err(DatrieError::new(
                crate::ErrorKind::InvalidFileSignature,
                format!("unexpected DArray signature '{}'", n),
            ));
        }
        // let d = unsafe { malloc(::core::mem::size_of::<DArray>() as libc::c_ulong) as *mut DArray };
        // if !(d.is_null() as libc::c_int as libc::c_long != 0) {
        //     unsafe {
        // if let Ok(num_cells) = reader.read_int32() {
        let mut num_cells = 0;
        reader.read_int32(&mut num_cells)?;

        // if reader.read_int32(&mut num_cells).is_ok() {
        // unsafe {
        //     (*d).num_cells = num_cells;
        // }
        if num_cells as libc::c_ulong
            > (18446744073709551615 as libc::c_ulong)
                .wrapping_div(size_of::<DACell>() as libc::c_ulong)
        {
            return Err(DatrieError::new(
                crate::ErrorKind::Bug,
                "reading darray failed: num_cells is too large".into(),
            ));
        }
        // unsafe {
        let cells = unsafe {
            malloc((num_cells as libc::c_ulong).wrapping_mul(size_of::<DACell>() as libc::c_ulong))
                as *mut DACell
        };
        let mut cells2 = Vec::with_capacity(num_cells as usize);
        // }
        if cells.is_null() {
            return Err(DatrieError::new(
                crate::ErrorKind::Memory,
                "reading darray failed: malloc failed".into(),
            ));
        }
        unsafe {
            (*cells.offset(0)).base = Self::SIGNATURE as i32;
            (*cells.offset(0)).check = num_cells;
        }
        cells2.push(DACell {
            base: Self::SIGNATURE as TrieIndex,
            check: num_cells,
        });
        // if !(((*d)cells).is_null() as libc::c_int as libc::c_long != 0) {
        //     (*((*d).cells).offset(0 as libc::c_int as isize)).base =
        //         0xdafcdafc as libc::c_uint as TrieIndex;
        //     (*((*d).cells).offset(0 as libc::c_int as isize)).check = (*d).num_cells;
        let mut n: isize = 1;
        loop {
            if !(n < num_cells as isize) {
                current_block = 11050875288958768710;
                break;
            }
            let mut base = 0;
            let mut check = 0;
            if reader.read_int32(&mut base).is_err() || reader.read_int32(&mut check).is_err() {
                current_block = 9985172916848320936;
                break;
            }
            unsafe {
                (*cells.offset(n)).base = base;
                (*cells.offset(n)).check = check;
            }
            cells2.push(DACell { base, check });
            // unsafe {
            //     if reader
            //         .read_int32(&mut (*cells.offset(n as isize)).base)
            //         .is_err()
            //         || reader
            //             .read_int32(&mut (*cells.offset(n as isize)).check)
            //             .is_err()
            //     {
            //         current_block = 9985172916848320936;
            //         break;
            //     }
            // }
            n += 1;
        }
        match current_block {
            11050875288958768710 => {
                return Ok(DArray {
                    num_cells,
                    cells,
                    cells2,
                })
            }
            _ => unsafe {
                free(cells as *mut libc::c_void);
            },
        }
        // }
        // }
        // }
        // }
        //     unsafe {
        //         free(d as *mut libc::c_void);
        //     }
        // }
        return Err(DatrieError::new(
            crate::ErrorKind::Bug,
            format!("reading darray failed: reading cell '{}' failed", n),
        ));
    }

    // pub unsafe fn fwrite(&self, file: *mut FILE) -> DatrieResult<()> {
    //     for i in 0..self.num_cells() {

    //     }
    // }
}

pub unsafe fn da_fwrite(d: *const DArray, file: *mut FILE) -> libc::c_int {
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
    pub fn serialize(&self, writer: &mut dyn std::io::Write) -> DatrieResult<()> {
        for i in 0..self.num_cells {
            let base = unsafe { (*self.cells.offset(i as isize)).base };
            let check = unsafe { (*self.cells.offset(i as isize)).check };
            writer.write_i32::<BigEndian>(base)?;
            writer.write_i32::<BigEndian>(check)?;
        }
        Ok(())
    }
    pub fn serialize_to_slice(&self, mut buf: &mut [u8]) -> DatrieResult<usize> {
        let mut written = 0;
        for i in 0..self.num_cells {
            let base = unsafe { (*self.cells.offset(i as isize)).base };
            let check = unsafe { (*self.cells.offset(i as isize)).check };
            buf.write_i32::<BigEndian>(base)?;
            buf.write_i32::<BigEndian>(check)?;
            written += 8;
        }
        Ok(written)
    }
}
//
pub unsafe fn da_serialize(d: *const DArray, ptr: *mut *mut uint8) {
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
        return if s < self.num_cells() as TrieIndex {
            let base = unsafe { (*((self).cells).offset(s as isize)).base };
            assert_eq!(base, self.cells2[s as usize].base);
            base
        } else {
            0 as libc::c_int
        };
    }
    pub fn get_check(&self, s: TrieIndex) -> TrieIndex {
        return if s < self.num_cells() as TrieIndex {
            let check = unsafe { (*(self.cells).offset(s as isize)).check };
            assert_eq!(check, self.cells2[s as usize].check);
            check
        } else {
            0 as libc::c_int
        };
    }
    pub fn num_cells(&self) -> usize {
        assert_eq!(self.num_cells as usize, self.cells2.len());
        self.cells2.len()
    }
    pub fn set_base(&mut self, s: TrieIndex, val: TrieIndex) {
        if (s < self.num_cells) as libc::c_int as libc::c_long != 0 {
            unsafe {
                (*self.cells.offset(s as isize)).base = val;
            }
        }
        let s = s as usize;
        if s < self.num_cells() {
            self.cells2[s].base = val;
        }
    }
    pub fn set_check(&mut self, s: TrieIndex, val: TrieIndex) {
        if (s < self.num_cells) as libc::c_int as libc::c_long != 0 {
            unsafe {
                (*self.cells.offset(s as isize)).check = val;
            }
        }
        let s = s as usize;
        if s < self.num_cells() {
            self.cells2[s].check = val;
        }
    }
    pub fn walk(
        &self,
        // mut d: *const DArray,
        s: *mut TrieIndex,
        c: TrieChar,
    ) -> Bool {
        let next: TrieIndex = unsafe { self.get_base(*s) } + c as libc::c_int;
        if unsafe { self.get_check(next) == *s } {
            unsafe {
                *s = next;
            }
            return DA_TRUE;
        }
        return DA_FALSE;
    }
    pub unsafe fn insert_branch(
        &mut self,
        // mut d: *mut DArray,
        s: TrieIndex,
        c: TrieChar,
    ) -> TrieIndex {
        // let mut base: TrieIndex = 0;
        let mut next: TrieIndex = 0;
        let mut base = self.get_base(s);
        if base > 0 as libc::c_int {
            next = base + c as libc::c_int;
            if self.get_check(next) == s {
                return next;
            }
            if base > 0x7fffffff as libc::c_int - c as libc::c_int
                || self.check_free_cell(next) as u64 == 0
            {
                // let mut symbols: *mut Symbols = 0 as *mut Symbols;
                let mut new_base: TrieIndex = 0;
                let mut symbols = self.output_symbols(s);
                symbols.add(c);
                new_base = self.find_free_base(&symbols);
                if (0 as libc::c_int == new_base) as libc::c_int as libc::c_long != 0 {
                    return 0 as libc::c_int;
                }
                self.relocate_base(s, new_base);
                next = new_base + c as libc::c_int;
            }
        } else {
            // let mut symbols_0: *mut Symbols = 0 as *mut Symbols;
            let mut new_base_0: TrieIndex = 0;
            let mut symbols_0 = Symbols::new();
            symbols_0.add(c);
            new_base_0 = self.find_free_base(&symbols_0);
            if (0 as libc::c_int == new_base_0) as libc::c_int as libc::c_long != 0 {
                return 0 as libc::c_int;
            }
            self.set_base(s, new_base_0);
            next = new_base_0 + c as libc::c_int;
        }
        self.alloc_cell(next);
        self.set_check(next, s);
        return next;
    }
}

// pub unsafe fn set_base(mut d: *mut DArray, mut s: TrieIndex, mut val: TrieIndex) {
//     if (s < (*d).num_cells) as libc::c_int as libc::c_long != 0 {
//         (*((*d).cells).offset(s as isize)).base = val;
//     }
// }
//
// pub unsafe fn set_check(mut d: *mut DArray, mut s: TrieIndex, mut val: TrieIndex) {
//     if (s < (*d).num_cells) as libc::c_int as libc::c_long != 0 {
//         (*((*d).cells).offset(s as isize)).check = val;
//     }
// }

// pub unsafe fn da_walk(mut d: *const DArray, mut s: *mut TrieIndex, mut c: TrieChar) -> Bool {
//     let mut next: TrieIndex = (*d).get_base(*s) + c as libc::c_int;
//     if (*d).get_check(next) == *s {
//         *s = next;
//         return DA_TRUE;
//     }
//     return DA_FALSE;
// }
impl DArray {
    // pub unsafe fn da_insert_branch(
    //     &mut self,
    //     // mut d: *mut DArray,
    //     mut s: TrieIndex,
    //     mut c: TrieChar,
    // ) -> TrieIndex {
    //     // let mut base: TrieIndex = 0;
    //     let mut next: TrieIndex = 0;
    //     let mut base = self.get_base(s);
    //     if base > 0 as libc::c_int {
    //         next = base + c as libc::c_int;
    //         if self.get_check(next) == s {
    //             return next;
    //         }
    //         if base > 0x7fffffff as libc::c_int - c as libc::c_int
    //             || self.check_free_cell(next) as u64 == 0
    //         {
    //             // let mut symbols: *mut Symbols = 0 as *mut Symbols;
    //             let mut new_base: TrieIndex = 0;
    //             let mut symbols = self.output_symbols(s);
    //             symbols.add(c);
    //             new_base = self.find_free_base(&symbols);
    //             if (0 as libc::c_int == new_base) as libc::c_int as libc::c_long != 0 {
    //                 return 0 as libc::c_int;
    //             }
    //             self.relocate_base(s, new_base);
    //             next = new_base + c as libc::c_int;
    //         }
    //     } else {
    //         // let mut symbols_0: *mut Symbols = 0 as *mut Symbols;
    //         let mut new_base_0: TrieIndex = 0;
    //         let mut symbols_0 = Symbols::new();
    //         symbols_0.add(c);
    //         new_base_0 = self.find_free_base(&symbols_0);
    //         if (0 as libc::c_int == new_base_0) as libc::c_int as libc::c_long != 0 {
    //             return 0 as libc::c_int;
    //         }
    //         self.set_base(s, new_base_0);
    //         next = new_base_0 + c as libc::c_int;
    //     }
    //     self.alloc_cell(next);
    //     self.set_check(next, s);
    //     return next;
    // }
    unsafe fn check_free_cell(&mut self, mut s: TrieIndex) -> Bool {
        return (self.extend_pool(s) as libc::c_uint != 0 && self.get_check(s) < 0 as libc::c_int)
            as libc::c_int as Bool;
    }
    fn has_children(&self, s: TrieIndex) -> Bool {
        let mut base: TrieIndex = 0;
        let mut c: TrieIndex = 0;
        let mut max_c: TrieIndex = 0;
        let mut base = self.get_base(s);
        if 0 as libc::c_int == base || base < 0 as libc::c_int {
            return DA_FALSE;
        }
        max_c = if (255 as libc::c_int) < self.num_cells - base {
            255 as libc::c_int
        } else {
            self.num_cells - base
        };
        c = 0 as libc::c_int;
        while c <= max_c {
            if self.get_check(base + c) == s {
                return DA_TRUE;
            }
            c += 1;
        }
        return DA_FALSE;
    }
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

    unsafe fn find_free_base(&mut self, mut symbols: &Symbols) -> TrieIndex {
        let mut first_sym: TrieChar = 0;
        let mut s: TrieIndex = 0;
        first_sym = symbols.get(0 as usize);
        s = -self.get_check(1 as libc::c_int);
        while s != 1 as libc::c_int && s < first_sym as TrieIndex + 3 as libc::c_int {
            s = -self.get_check(s);
        }
        if s == 1 as libc::c_int {
            s = first_sym as libc::c_int + 3 as libc::c_int;
            loop {
                if self.extend_pool(s) as u64 == 0 {
                    return 0 as libc::c_int;
                }
                if self.get_check(s) < 0 as libc::c_int {
                    break;
                }
                s += 1;
                s;
            }
        }
        while self.fit_symbols(s - first_sym as libc::c_int, symbols) as u64 == 0 {
            if -self.get_check(s) == 1 as libc::c_int {
                if (self.extend_pool(self.num_cells) as u64 == 0) as libc::c_int as libc::c_long
                    != 0
                {
                    return 0 as libc::c_int;
                }
            }
            s = -self.get_check(s);
        }
        return s - first_sym as libc::c_int;
    }
    unsafe fn fit_symbols(&mut self, mut base: TrieIndex, symbols: &Symbols) -> Bool {
        let mut i = 0;
        while i < symbols.num() {
            let mut sym: TrieChar = symbols.get(i);
            if base > 0x7fffffff as libc::c_int - sym as libc::c_int
                || self.check_free_cell(base + sym as libc::c_int) as u64 == 0
            {
                return DA_FALSE;
            }
            i += 1;
        }
        return DA_TRUE;
    }
    unsafe fn relocate_base(&mut self, mut s: TrieIndex, mut new_base: TrieIndex) {
        let mut old_base: TrieIndex = 0;
        // let mut symbols: *mut Symbols = 0 as *mut Symbols;
        // let mut i: libc::c_int = 0;
        old_base = self.get_base(s);
        let symbols = self.output_symbols(s);
        // i = 0 as libc::c_int;
        let mut i = 0;
        while i < symbols.num() {
            let mut old_next: TrieIndex = 0;
            let mut new_next: TrieIndex = 0;
            let mut old_next_base: TrieIndex = 0;
            old_next = old_base + symbols.get(i) as libc::c_int;
            new_next = new_base + symbols.get(i) as libc::c_int;
            old_next_base = self.get_base(old_next);
            self.alloc_cell(new_next);
            self.set_check(new_next, s);
            self.set_base(new_next, old_next_base);
            if old_next_base > 0 as libc::c_int {
                let mut c: TrieIndex = 0;
                let mut max_c: TrieIndex = if (255 as libc::c_int) < self.num_cells - old_next_base
                {
                    255 as libc::c_int
                } else {
                    self.num_cells - old_next_base
                };
                c = 0 as libc::c_int;
                while c <= max_c {
                    if self.get_check(old_next_base + c) == old_next {
                        self.set_check(old_next_base + c, new_next);
                    }
                    c += 1;
                }
            }
            self.free_cell(old_next);
            i += 1;
        }
        self.set_base(s, new_base);
    }
    unsafe fn extend_pool(&mut self, to_index: TrieIndex) -> Bool {
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
        if to_index < self.num_cells {
            return DA_TRUE;
        }
        new_block = realloc(
            self.cells as *mut libc::c_void,
            ((to_index + 1 as libc::c_int) as libc::c_ulong)
                .wrapping_mul(::core::mem::size_of::<DACell>() as libc::c_ulong),
        );
        if new_block.is_null() as libc::c_int as libc::c_long != 0 {
            return DA_FALSE;
        }
        self.cells = new_block as *mut DACell;
        new_begin = self.num_cells;
        self.num_cells = to_index + 1 as libc::c_int;
        self.cells2.reserve((to_index + 1) as usize);
        for _ in new_begin..(to_index + 1) {
            self.cells2.push(DACell { base: 0, check: 0 });
        }
        assert_eq!(self.num_cells as usize, self.cells2.len());

        i = new_begin;
        while i < to_index {
            self.set_check(i, -(i + 1 as libc::c_int));
            self.set_base(i + 1 as libc::c_int, -i);
            i += 1;
        }
        free_tail = -self.get_base(1 as libc::c_int);
        self.set_check(free_tail, -new_begin);
        self.set_base(new_begin, -free_tail);
        self.set_check(to_index, -(1 as libc::c_int));
        self.set_base(1 as libc::c_int, -to_index);
        // (*self.cells.offset(0 as libc::c_int as isize)).check = self.num_cells;
        self.set_check(0, self.num_cells);
        return DA_TRUE;
    }

    pub fn prune(&mut self, s: TrieIndex) {
        self.prune_upto(self.get_root(), s);
    }

    pub fn prune_upto(&mut self, p: TrieIndex, mut s: TrieIndex) {
        while p != s && self.has_children(s) as u64 == 0 {
            // let mut parent: TrieIndex = 0;
            let parent = self.get_check(s);
            self.free_cell(s);
            s = parent;
        }
    }
    fn alloc_cell(&mut self, cell: TrieIndex) {
        // let mut prev: TrieIndex = 0;
        // let mut next: TrieIndex = 0;
        let prev = -self.get_base(cell);
        let next = -self.get_check(cell);
        self.set_check(prev, -next);
        self.set_base(next, -prev);
    }
    fn free_cell(&mut self, cell: TrieIndex) {
        // let mut i: TrieIndex = 0;
        // let mut prev: TrieIndex = 0;
        let mut i = -self.get_check(1 as libc::c_int);
        while i != 1 as libc::c_int && i < cell {
            i = -self.get_check(i);
        }
        let prev = -self.get_base(i);
        self.set_check(cell, -i);
        self.set_base(cell, -prev);
        self.set_check(prev, -cell);
        self.set_base(i, -cell);
    }
}
impl DArray {
    pub unsafe fn first_separate(
        &self,
        // d: *mut DArray,
        mut root: TrieIndex,
        keybuff: *mut TrieString,
    ) -> TrieIndex {
        let mut base: TrieIndex = 0;
        let mut c: TrieIndex = 0;
        let mut max_c: TrieIndex = 0;
        loop {
            base = self.get_base(root);
            if !(base >= 0 as libc::c_int) {
                break;
            }
            max_c = if (255 as libc::c_int) < self.num_cells - base {
                255 as libc::c_int
            } else {
                self.num_cells - base
            };
            c = 0 as libc::c_int;
            while c <= max_c {
                if self.get_check(base + c) == root {
                    break;
                }
                c += 1;
            }
            if c > max_c {
                return 0 as libc::c_int;
            }
            trie_string_append_char(keybuff, c as TrieChar);
            root = base + c;
        }
        return root;
    }
    pub unsafe fn next_separate(
        &self,
        // mut d: *mut DArray,
        mut root: TrieIndex,
        mut sep: TrieIndex,
        mut keybuff: *mut TrieString,
    ) -> TrieIndex {
        let mut parent: TrieIndex = 0;
        let mut base: TrieIndex = 0;
        let mut c: TrieIndex = 0;
        let mut max_c: TrieIndex = 0;
        while sep != root {
            parent = self.get_check(sep);
            base = self.get_base(parent);
            c = sep - base;
            trie_string_cut_last(keybuff);
            max_c = if (255 as libc::c_int) < self.num_cells - base {
                255 as libc::c_int
            } else {
                self.num_cells - base
            };
            loop {
                c += 1;
                if !(c <= max_c) {
                    break;
                }
                if self.get_check(base + c) == parent {
                    trie_string_append_char(keybuff, c as TrieChar);
                    return self.first_separate(base + c, keybuff);
                }
            }
            sep = parent;
        }
        return 0 as libc::c_int;
    }
}

// pub unsafe fn first_separate(
//     d: *mut DArray,
//     mut root: TrieIndex,
//     keybuff: *mut TrieString,
// ) -> TrieIndex {
//     let mut base: TrieIndex = 0;
//     let mut c: TrieIndex = 0;
//     let mut max_c: TrieIndex = 0;
//     loop {
//         base = (*d).get_base(root);
//         if !(base >= 0 as libc::c_int) {
//             break;
//         }
//         max_c = if (255 as libc::c_int) < (*d).num_cells - base {
//             255 as libc::c_int
//         } else {
//             (*d).num_cells - base
//         };
//         c = 0 as libc::c_int;
//         while c <= max_c {
//             if (*d).get_check(base + c) == root {
//                 break;
//             }
//             c += 1;
//             c;
//         }
//         if c > max_c {
//             return 0 as libc::c_int;
//         }
//         trie_string_append_char(keybuff, c as TrieChar);
//         root = base + c;
//     }
//     return root;
// }

// pub unsafe fn next_separate(
//     mut d: *mut DArray,
//     mut root: TrieIndex,
//     mut sep: TrieIndex,
//     mut keybuff: *mut TrieString,
// ) -> TrieIndex {
//     let mut parent: TrieIndex = 0;
//     let mut base: TrieIndex = 0;
//     let mut c: TrieIndex = 0;
//     let mut max_c: TrieIndex = 0;
//     while sep != root {
//         parent = (*d).get_check(sep);
//         base = (*d).get_base(parent);
//         c = sep - base;
//         trie_string_cut_last(keybuff);
//         max_c = if (255 as libc::c_int) < (*d).num_cells - base {
//             255 as libc::c_int
//         } else {
//             (*d).num_cells - base
//         };
//         loop {
//             c += 1;
//             if !(c <= max_c) {
//                 break;
//             }
//             if (*d).get_check(base + c) == parent {
//                 trie_string_append_char(keybuff, c as TrieChar);
//                 return first_separate(d, base + c, keybuff);
//             }
//         }
//         sep = parent;
//     }
//     return 0 as libc::c_int;
// }

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
