mod trie_str;

pub use trie_str::{TrieChar, TrieCharString}; //, TrieString};

use std::path::Path;
use std::{fs, io};

use crate::fileutils::{CFile, ReadExt};
use crate::{alpha_map::*, darray::*, tail::*};
use crate::{trie_string::*, DatrieError, DatrieResult, ErrorKind};
use ::libc;

extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn fclose(__stream: *mut FILE) -> libc::c_int;
    fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
}
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type uint8 = libc::c_uchar;
pub type uint32 = libc::c_uint;
pub type int32 = libc::c_int;
pub type AlphaChar = uint32;
pub type TrieIndex = int32;
pub type TrieData = int32;
pub type FILE = libc::FILE;
// #[derive(Copy, Clone)]
// #[repr(C)]
#[derive(Debug)]
pub struct Trie {
    pub alpha_map: Box<AlphaMap>,
    pub da: Box<DArray>,
    pub tail: Box<Tail>,
    pub is_dirty: Bool,
}
pub type TrieEnumFunc =
    Option<unsafe extern "C" fn(*const AlphaChar, TrieData, *mut libc::c_void) -> Bool>;
#[derive(Copy, Clone)]
// #[repr(C)]
pub struct TrieState {
    pub trie: *const Trie,
    pub index: TrieIndex,
    pub suffix_idx: libc::c_short,
    pub is_suffix: libc::c_short,
}
#[derive(Copy, Clone)]
// #[repr(C)]
pub struct TrieIterator {
    pub root: *const TrieState,
    pub state: *mut TrieState,
    pub key: *mut TrieString,
}
impl Trie {
    pub fn new(alpha_map: &AlphaMap) -> DatrieResult<Trie> {
        let alpha_map = Box::new(alpha_map.clone());
        let da = Box::new(DArray::new()?);
        let tail = Box::new(Tail::new());
        // let da = unsafe { da_new() };
        // if da.is_null() {
        //     return Err(DatrieError::new(ErrorKind::Memory, "malloc failed".into()));
        // }
        // let tail = unsafe { tail_new() };
        // if tail.is_null() {
        //     // unsafe {
        //     //     da_free(da);
        //     // }
        //     return Err(DatrieError::new(ErrorKind::Memory, "malloc failed".into()));
        // }
        Ok(Trie {
            alpha_map,
            da,
            tail,
            is_dirty: DA_TRUE,
        })
    }

    pub unsafe fn new_from_file(path: *const libc::c_char) -> DatrieResult<Trie> {
        let trie_file = fopen(path, b"rb\0" as *const u8 as *const libc::c_char);
        if trie_file.is_null() {
            return Err(DatrieError::new(
                ErrorKind::Io,
                "failed to open file".into(),
            ));
        }
        let result = Trie::fread(trie_file);
        fclose(trie_file);
        return result;
    }
    pub fn from_path(path: &Path) -> DatrieResult<Trie> {
        let trie_file = fs::File::open(path)?;
        let mut reader = io::BufReader::new(trie_file);
        Trie::fread_safe(&mut reader)
    }
    pub unsafe fn fread(file: *mut FILE) -> DatrieResult<Trie> {
        // let mut trie: *mut Trie = 0 as *mut Trie;
        // trie = malloc(::core::mem::size_of::<Trie>() as libc::c_ulong) as *mut Trie;
        // if trie.is_null() as libc::c_int as libc::c_long != 0 {
        //     return 0 as *mut Trie;
        // }
        let alpha_map = Box::new(AlphaMap::fread_bin(file)?);
        let da = Box::new(DArray::fread(file)?);
        // let da = da_fread(file);

        // if da.is_null() {
        //     return Err(DatrieError::new(ErrorKind::Memory, "malloc failed".into()));
        // }
        let mut cfile = CFile::new(file);
        let tail = Box::new(Tail::fread_safe(&mut cfile)?);
        // let tail = tail_fread(file);
        // if tail.is_null() {
        //     // da_free(da);
        //     return Err(DatrieError::new(ErrorKind::Memory, "malloc failed".into()));
        // }

        return Ok(Trie {
            alpha_map,
            da,
            tail,
            is_dirty: DA_FALSE,
        });
    }
    pub fn fread_safe<R: ReadExt + io::Seek>(reader: &mut R) -> DatrieResult<Trie> {
        let alpha_map = Box::new(AlphaMap::fread_bin_safe(reader)?);
        let da = Box::new(DArray::fread_safe(reader)?);
        let tail = Box::new(Tail::fread_safe(reader)?);
        Ok(Trie {
            alpha_map,
            da,
            tail,
            is_dirty: DA_FALSE,
        })
    }
}
// pub unsafe fn free(mut trie: *mut Trie) {
//     // drop((*trie).alpha_map.as_mut());
//     da_free((*trie).da);
//     tail_free((*trie).tail);
//     free(trie as *mut libc::c_void);
// }
// impl Drop for Trie {
//     fn drop(&mut self) {
//         unsafe {
//             // drop((*trie).alpha_map.as_mut());
//             // da_free((*self).da);
//             tail_free((*self).tail);
//             // free(trie as *mut libc::c_void);
//         }
//     }
// }
impl Trie {
    pub unsafe fn save(trie: *mut Trie, path: *const libc::c_char) -> libc::c_int {
        let file: *mut FILE = fopen(path, b"wb+\0" as *const u8 as *const libc::c_char);
        if file.is_null() {
            return -(1 as libc::c_int);
        }
        let res = Trie::fwrite(trie, file);
        fclose(file);
        return res;
    }
    pub fn save_safe(&mut self, path: &Path) -> DatrieResult<()> {
        let mut file = fs::File::create(path)?;
        self.serialize_safe(&mut file)?;
        Ok(())
    }
    // pub unsafe fn get_serialized_size(trie: *const Trie) -> size_t {
    //     return (alpha_map_get_serialized_size((*trie).alpha_map.as_ref()))
    //         .wrapping_add(da_get_serialized_size((*trie).da))
    //         .wrapping_add(tail_get_serialized_size((*trie).tail));
    // }
    pub fn get_serialized_size(&self) -> usize {
        return self.alpha_map.get_serialized_size()
            + self.da.get_serialized_size()
            + self.tail.get_serialized_size();
    }
    pub unsafe fn serialize(trie: *mut Trie, ptr: *mut uint8) {
        let mut ptr1: *mut uint8 = ptr;
        alpha_map_serialize_bin((*trie).alpha_map.as_ref(), &mut ptr1);
        da_serialize((*trie).da.as_ref(), &mut ptr1);
        tail_serialize((*trie).tail.as_ref(), &mut ptr1);
        (*trie).is_dirty = DA_FALSE;
    }
    pub fn serialize_safe(&mut self, mut writer: impl std::io::Write) -> DatrieResult<()> {
        self.alpha_map.serialize(&mut writer)?;
        self.da.serialize(&mut writer)?;
        self.tail.serialize(&mut writer)?;
        Ok(())
    }
    pub fn serialize_to_slice(&mut self, buf: &mut [u8]) -> DatrieResult<usize> {
        let mut start = self.alpha_map.serialize_to_slice(buf)?;
        start += self.da.serialize_to_slice(&mut buf[start..])?;
        start += self.tail.serialize_to_slice(&mut buf[start..])?;
        Ok(start)
    }
    pub unsafe fn fwrite(trie: *mut Trie, file: *mut FILE) -> libc::c_int {
        if alpha_map_fwrite_bin((*trie).alpha_map.as_ref(), file) != 0 as libc::c_int {
            return -(1 as libc::c_int);
        }
        if da_fwrite((*trie).da.as_ref(), file) != 0 as libc::c_int {
            return -(1 as libc::c_int);
        }
        if tail_fwrite((*trie).tail.as_ref(), file) != 0 as libc::c_int {
            return -(1 as libc::c_int);
        }
        (*trie).is_dirty = DA_FALSE;
        return 0 as libc::c_int;
    }
    pub fn is_dirty(&self) -> Bool {
        return self.is_dirty;
    }
    pub unsafe fn retrieve(&self, key: *const AlphaChar, o_data: *mut TrieData) -> Bool {
        // let mut s: TrieIndex = 0;
        // let mut suffix_idx: libc::c_short = 0;
        // let mut p: *const AlphaChar = 0 as *const AlphaChar;
        // s = da_get_root((*trie).da);
        let mut s = self.da.get_root();
        let mut p: *const AlphaChar = key;
        while !(self.da.get_base(s) < 0 as libc::c_int) {
            let tc: TrieIndex = unsafe { alpha_map_char_to_trie(self.alpha_map.as_ref(), *p) };
            if 0x7fffffff as libc::c_int == tc {
                return DA_FALSE;
            }
            if self.da.walk(&mut s, tc as TrieChar) as u64 == 0 {
                return DA_FALSE;
            }
            // if p.is_null() {
            //     break;
            // }
            if unsafe { 0 as libc::c_uint == *p } {
                break;
            }
            p = unsafe { p.offset(1) };
        }
        s = -self.da.get_base(s);
        let mut suffix_idx: libc::c_short = 0;
        loop {
            let tc_0: TrieIndex = unsafe { alpha_map_char_to_trie(self.alpha_map.as_ref(), *p) };

            if 0x7fffffff as libc::c_int == tc_0 {
                return DA_FALSE;
            }
            if unsafe { self.tail.walk_char(s, &mut suffix_idx, tc_0 as TrieChar) } as u64 == 0 {
                return DA_FALSE;
            }
            // if p.is_null() {
            //     break;
            // }
            if unsafe { 0 as libc::c_uint == *p } {
                break;
            }
            p = unsafe { p.offset(1) };
        }
        if !o_data.is_null() {
            unsafe {
                *o_data = self.tail.get_data(s);
            }
        }
        return DA_TRUE;
    }

    pub unsafe fn store(&mut self, key: *const AlphaChar, data: TrieData) -> Bool {
        return self.store_conditionally(key, data, DA_TRUE);
    }

    pub unsafe fn store_if_absent(&mut self, key: *const AlphaChar, data: TrieData) -> Bool {
        return self.store_conditionally(key, data, DA_FALSE);
    }
    unsafe fn store_conditionally(
        &mut self,
        // trie: *mut Trie,
        key: *const AlphaChar,
        data: TrieData,
        is_overwrite: Bool,
    ) -> Bool {
        // dbg!(&*trie);
        // dbg!(&*key);
        // let mut s: TrieIndex = 0;
        // let mut t: TrieIndex = 0;
        // let mut suffix_idx: libc::c_short = 0;
        // let mut p: *const AlphaChar = 0 as *const AlphaChar;
        // let mut sep: *const AlphaChar = 0 as *const AlphaChar;
        let mut s = self.da.get_root();
        let mut p: *const AlphaChar = key;
        while !(self.da.get_base(s) < 0 as libc::c_int) {
            let tc: TrieIndex = alpha_map_char_to_trie(self.alpha_map.as_ref(), *p);
            if 0x7fffffff as libc::c_int == tc {
                return DA_FALSE;
            }
            if self.da.walk(&mut s, tc as TrieChar) as u64 == 0 {
                // let mut key_str: *mut TrieChar = 0 as *mut TrieChar;
                // let mut res: Bool = DA_FALSE;
                let key_str = alpha_map_char_to_trie_str(self.alpha_map.as_ref(), p);
                if key_str.is_null() {
                    return DA_FALSE;
                }
                let res = self.branch_in_branch(s, key_str, data);
                free(key_str as *mut libc::c_void);
                return res;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
        }
        let sep: *const AlphaChar = p;
        let t = -(*self.da).get_base(s);
        // suffix_idx = 0 as libc::c_int as libc::c_short;
        let mut suffix_idx: libc::c_short = 0;
        loop {
            let tc_0: TrieIndex = alpha_map_char_to_trie(self.alpha_map.as_ref(), *p);
            if 0x7fffffff as libc::c_int == tc_0 {
                return DA_FALSE;
            }
            if self.tail.walk_char(t, &mut suffix_idx, tc_0 as TrieChar) as u64 == 0 {
                // let mut tail_str: *mut TrieChar = 0 as *mut TrieChar;
                // let mut res_0: Bool = DA_FALSE;
                let tail_str = alpha_map_char_to_trie_str(self.alpha_map.as_ref(), sep);
                if tail_str.is_null() {
                    return DA_FALSE;
                }
                let res_0 = self.branch_in_tail(s, tail_str, data);
                free(tail_str as *mut libc::c_void);
                return res_0;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
        }
        if is_overwrite as u64 == 0 {
            return DA_FALSE;
        }
        self.tail.set_data(t, data);
        self.is_dirty = DA_TRUE;
        return DA_TRUE;
    }
    unsafe fn branch_in_branch(
        &mut self,
        // mut trie: *mut Trie,
        sep_node: TrieIndex,
        mut suffix: *const TrieChar,
        data: TrieData,
    ) -> Bool {
        // let mut new_da: TrieIndex = 0;
        // let mut new_tail: TrieIndex = 0;
        // let self = unsafe {}
        let new_da = self.da.insert_branch(sep_node, *suffix);
        if 0 as libc::c_int == new_da {
            return DA_FALSE;
        }
        if '\0' as i32 != *suffix as libc::c_int {
            suffix = suffix.offset(1);
        }
        let new_tail = self.tail.add_suffix(suffix);
        self.tail.set_data(new_tail, data);
        self.da.set_base(new_da, -new_tail);
        self.is_dirty = DA_TRUE;
        return DA_TRUE;
    }
    unsafe fn branch_in_tail(
        &mut self,
        // mut trie: *mut Trie,
        sep_node: TrieIndex,
        mut suffix: *const TrieChar,
        data: TrieData,
    ) -> Bool {
        let current_block: u64;
        // let mut old_tail: TrieIndex = 0;
        // let mut old_da: TrieIndex = 0;
        // let mut s: TrieIndex = 0;
        // let mut old_suffix: *const TrieChar = 0 as *const TrieChar;
        // let mut p: *const TrieChar = 0 as *const TrieChar;
        let old_tail = -(*self.da).get_base(sep_node);
        let old_suffix = self.tail.get_suffix(old_tail);
        if old_suffix.is_null() {
            return DA_FALSE;
        }
        let mut p = old_suffix;
        let mut s = sep_node;
        loop {
            if !(*p as libc::c_int == *suffix as libc::c_int) {
                current_block = 6937071982253665452;
                break;
            }
            let t: TrieIndex = self.da.insert_branch(s, *p);
            if 0 as libc::c_int == t {
                current_block = 13151848498364941746;
                break;
            }
            s = t;
            p = p.offset(1);
            suffix = suffix.offset(1);
        }
        match current_block {
            6937071982253665452 => {
                let old_da = self.da.insert_branch(s, *p);
                if !(0 as libc::c_int == old_da) {
                    if '\0' as i32 != *p as libc::c_int {
                        p = p.offset(1);
                    }
                    self.tail.set_suffix(old_tail, p);
                    self.da.set_base(old_da, -old_tail);
                    return self.branch_in_branch(s, suffix, data);
                }
            }
            _ => {}
        }
        self.da.prune_upto(sep_node, s);
        self.da.set_base(sep_node, -old_tail);
        return DA_FALSE;
    }
}

impl Trie {
    pub fn delete(&mut self, key: *const AlphaChar) -> Bool {
        // let mut s: TrieIndex = 0;
        // let mut t: TrieIndex = 0;
        // let mut suffix_idx: libc::c_short = 0;
        // let mut p: *const AlphaChar = 0 as *const AlphaChar;
        let mut s = (*self.da).get_root();
        let mut p = key;
        while !((*self.da).get_base(s) < 0 as libc::c_int) {
            let tc: TrieIndex = unsafe { alpha_map_char_to_trie(self.alpha_map.as_ref(), *p) };
            if 0x7fffffff as libc::c_int == tc {
                return DA_FALSE;
            }
            if unsafe { self.da.walk(&mut s, tc as TrieChar) } as u64 == 0 {
                return DA_FALSE;
            }
            if unsafe { 0 as libc::c_int as libc::c_uint == *p } {
                break;
            }
            p = unsafe { p.offset(1) };
        }
        let t = -(*self.da).get_base(s);
        // suffix_idx = 0 as libc::c_int as libc::c_short;
        let mut suffix_idx: libc::c_short = 0;
        loop {
            let tc_0: TrieIndex = unsafe { alpha_map_char_to_trie(self.alpha_map.as_ref(), *p) };
            if 0x7fffffff as libc::c_int == tc_0 {
                return DA_FALSE;
            }
            if unsafe { self.tail.walk_char(t, &mut suffix_idx, tc_0 as TrieChar) as u64 == 0 } {
                return DA_FALSE;
            }
            if unsafe { 0 as libc::c_int as libc::c_uint == *p } {
                break;
            }
            p = unsafe { p.offset(1) };
        }
        unsafe {
            self.tail.delete(t);
        }
        self.da.set_base(s, 0 as libc::c_int);
        self.da.prune(s);
        self.is_dirty = DA_TRUE;
        return DA_TRUE;
    }

    pub unsafe fn enumerate(
        &self,
        // mut trie: *const Trie,
        enum_func: TrieEnumFunc,
        user_data: *mut libc::c_void,
    ) -> Bool {
        // let mut root: *mut TrieState = 0 as *mut TrieState;
        // let mut iter: *mut TrieIterator = 0 as *mut TrieIterator;
        let mut cont: Bool = DA_TRUE;
        let root = self.root();
        if root.is_null() as libc::c_int as libc::c_long != 0 {
            return DA_FALSE;
        }
        let iter = TrieIterator::new(root);
        if iter.is_null() as libc::c_int as libc::c_long != 0 {
            TrieState::free(root);
            return DA_FALSE;
        } else {
            while cont as libc::c_uint != 0 && TrieIterator::next(iter) as libc::c_uint != 0 {
                let key: *mut AlphaChar = TrieIterator::get_key(iter);
                let data: TrieData = TrieIterator::get_data(iter);
                cont = (Some(enum_func.expect("non-null function pointer")))
                    .expect("non-null function pointer")(
                    key, data, user_data
                );
                free(key as *mut libc::c_void);
            }
            TrieIterator::free(iter);
            TrieState::free(root);
            return cont;
        };
    }

    pub unsafe fn root(&self) -> *mut TrieState {
        return TrieState::new(
            &*self,
            (*(*self).da).get_root(),
            0 as libc::c_int as libc::c_short,
            DA_FALSE as libc::c_int as libc::c_short,
        );
    }
}

impl TrieState {
    unsafe fn new(
        mut trie: *const Trie,
        mut index: TrieIndex,
        mut suffix_idx: libc::c_short,
        mut is_suffix: libc::c_short,
    ) -> *mut TrieState {
        let mut s: *mut TrieState = 0 as *mut TrieState;
        s = malloc(::core::mem::size_of::<TrieState>() as libc::c_ulong) as *mut TrieState;
        if s.is_null() as libc::c_int as libc::c_long != 0 {
            return 0 as *mut TrieState;
        }
        (*s).trie = trie;
        (*s).index = index;
        (*s).suffix_idx = suffix_idx;
        (*s).is_suffix = is_suffix;
        return s;
    }

    pub unsafe fn trie_state_copy(dst: *mut TrieState, src: *const TrieState) {
        *dst = *src;
    }
    pub unsafe fn trie_state_clone(s: *const TrieState) -> *mut TrieState {
        return TrieState::new((*s).trie, (*s).index, (*s).suffix_idx, (*s).is_suffix);
    }

    pub unsafe fn free(s: *mut TrieState) {
        free(s as *mut libc::c_void);
    }

    pub unsafe fn rewind(s: *mut TrieState) {
        (*s).index = (*(*(*s).trie).da).get_root();
        (*s).is_suffix = DA_FALSE as libc::c_int as libc::c_short;
    }

    pub unsafe fn walk(s: *mut TrieState, c: AlphaChar) -> Bool {
        let tc: TrieIndex = alpha_map_char_to_trie((*(*s).trie).alpha_map.as_ref(), c);
        if (0x7fffffff as libc::c_int == tc) as libc::c_int as libc::c_long != 0 {
            return DA_FALSE;
        }
        if (*s).is_suffix == 0 {
            let ret: Bool = (*(*s).trie).da.walk(&mut (*s).index, tc as TrieChar);
            if ret as libc::c_uint != 0
                && (*(*(*s).trie).da).get_base((*s).index) < 0 as libc::c_int
            {
                (*s).index = -(*(*(*s).trie).da).get_base((*s).index);
                (*s).suffix_idx = 0 as libc::c_int as libc::c_short;
                (*s).is_suffix = DA_TRUE as libc::c_int as libc::c_short;
            }
            return ret;
        } else {
            return (*(*s).trie).tail.walk_char(
                // (*(*s).trie).tail.as_ref(),
                (*s).index,
                &mut (*s).suffix_idx,
                tc as TrieChar,
            );
        };
    }

    pub unsafe fn is_walkable(mut s: *const TrieState, mut c: AlphaChar) -> Bool {
        let mut tc: TrieIndex = alpha_map_char_to_trie((*(*s).trie).alpha_map.as_ref(), c);
        if (0x7fffffff as libc::c_int == tc) as libc::c_int as libc::c_long != 0 {
            return DA_FALSE;
        }
        if (*s).is_suffix == 0 {
            return ((*(*(*s).trie).da)
                .get_check((*(*(*s).trie).da).get_base((*s).index) + tc as TrieChar as libc::c_int)
                == (*s).index) as libc::c_int as Bool;
        } else {
            return (*((*(*s).trie).tail.get_suffix((*s).index)).offset((*s).suffix_idx as isize)
                as libc::c_int
                == tc as TrieChar as libc::c_int) as libc::c_int as Bool;
        };
    }

    pub unsafe fn is_terminal(s: *const TrieState) -> bool {
        TrieState::is_walkable(s, 0) == DA_TRUE
    }

    pub unsafe fn walkable_chars(
        s: *const TrieState,
        chars: *mut AlphaChar,
        chars_nelm: libc::c_int,
    ) -> libc::c_int {
        let syms_num: libc::c_int;
        if (*s).is_suffix == 0 {
            // let syms = da_output_symbols((*(*s).trie).da, (*s).index);
            let syms = (*(*(*s).trie).da).output_symbols((*s).index);
            syms_num = syms.num() as libc::c_int;
            let mut i = 0 as libc::c_int;
            while i < syms_num && i < chars_nelm {
                let tc: TrieChar = syms.get(i as usize);
                *chars.offset(i as isize) =
                    alpha_map_trie_to_char((*(*s).trie).alpha_map.as_ref(), tc);
                i += 1;
            }
            // symbols_free(syms);
        } else {
            let suffix: *const TrieChar = (*(*s).trie).tail.get_suffix((*s).index);
            *chars.offset(0 as libc::c_int as isize) = alpha_map_trie_to_char(
                (*(*s).trie).alpha_map.as_ref(),
                *suffix.offset((*s).suffix_idx as isize),
            );
            syms_num = 1 as libc::c_int;
        }
        return syms_num;
    }

    pub unsafe fn is_single(s: *const TrieState) -> Bool {
        return (*s).is_suffix as Bool;
    }

    pub unsafe fn get_data(s: *const TrieState) -> TrieData {
        if s.is_null() {
            return -(1 as libc::c_int);
        }
        if (*s).is_suffix == 0 {
            let mut index: TrieIndex = (*s).index;
            if (*(*s).trie).da.walk(&mut index, '\0' as i32 as TrieChar) as u64 != 0 {
                if (*(*(*s).trie).da).get_base(index) < 0 as libc::c_int {
                    index = -(*(*(*s).trie).da).get_base(index);
                    return ((*(*s).trie).tail).get_data(index);
                }
            }
        } else if *((*(*s).trie).tail.get_suffix((*s).index)).offset((*s).suffix_idx as isize)
            as libc::c_int
            == '\0' as i32
        {
            return (*(*s).trie).tail.get_data((*s).index);
        }
        return -(1 as libc::c_int);
    }
}
impl TrieIterator {
    pub unsafe fn new(mut s: *mut TrieState) -> *mut TrieIterator {
        let mut iter: *mut TrieIterator = 0 as *mut TrieIterator;
        iter = malloc(::core::mem::size_of::<TrieIterator>() as libc::c_ulong) as *mut TrieIterator;
        if iter.is_null() as libc::c_int as libc::c_long != 0 {
            return 0 as *mut TrieIterator;
        }
        (*iter).root = s;
        (*iter).state = 0 as *mut TrieState;
        (*iter).key = 0 as *mut TrieString;
        return iter;
    }

    pub unsafe fn free(mut iter: *mut TrieIterator) {
        if !((*iter).state).is_null() {
            TrieState::free((*iter).state);
        }
        if !((*iter).key).is_null() {
            trie_string_free((*iter).key);
        }
        free(iter as *mut libc::c_void);
    }

    pub unsafe fn next(mut iter: *mut TrieIterator) -> Bool {
        let mut s: *mut TrieState = (*iter).state;
        let mut sep: TrieIndex = 0;
        if s.is_null() {
            (*iter).state = TrieState::trie_state_clone((*iter).root);
            s = (*iter).state;
            if (*s).is_suffix != 0 {
                return DA_TRUE;
            }
            (*iter).key = trie_string_new(20 as libc::c_int);
            sep = (*(*s).trie).da.first_separate((*s).index, (*iter).key);
            // sep = da_first_separate((*(*s).trie).da.as_mut(), (*s).index, (*iter).key);
            if 0 as libc::c_int == sep {
                return DA_FALSE;
            }
            (*s).index = sep;
            return DA_TRUE;
        }
        if (*s).is_suffix != 0 {
            return DA_FALSE;
        }
        sep = (*(*s).trie).da.next_separate(
            // (*(*s).trie).da.as_mut(),
            (*(*iter).root).index,
            (*s).index,
            (*iter).key,
        );
        if 0 as libc::c_int == sep {
            return DA_FALSE;
        }
        (*s).index = sep;
        return DA_TRUE;
    }

    pub unsafe fn get_key(mut iter: *const TrieIterator) -> *mut AlphaChar {
        let mut s: *const TrieState = 0 as *const TrieState;
        let mut tail_str: *const TrieChar = 0 as *const TrieChar;
        let mut alpha_key: *mut AlphaChar = 0 as *mut AlphaChar;
        let mut alpha_p: *mut AlphaChar = 0 as *mut AlphaChar;
        s = (*iter).state;
        if s.is_null() {
            return 0 as *mut AlphaChar;
        }
        if (*s).is_suffix != 0 {
            tail_str = (*(*s).trie).tail.get_suffix((*s).index);
            if tail_str.is_null() {
                return 0 as *mut AlphaChar;
            }
            tail_str = tail_str.offset((*s).suffix_idx as libc::c_int as isize);
            alpha_key = malloc(
                (::core::mem::size_of::<AlphaChar>() as libc::c_ulong).wrapping_mul(
                    (trie_char_strlen(tail_str)).wrapping_add(1 as libc::c_int as libc::c_ulong),
                ),
            ) as *mut AlphaChar;
            alpha_p = alpha_key;
        } else {
            let mut tail_idx: TrieIndex = 0;
            let mut i: libc::c_int = 0;
            let mut key_len: libc::c_int = 0;
            let mut key_p: *const TrieChar = 0 as *const TrieChar;
            tail_idx = -(*(*(*s).trie).da).get_base((*s).index);
            tail_str = (*(*s).trie).tail.get_suffix(tail_idx);
            if tail_str.is_null() {
                return 0 as *mut AlphaChar;
            }
            key_len = trie_string_length((*iter).key);
            key_p = trie_string_get_val((*iter).key) as *const TrieChar;
            alpha_key = malloc(
                (::core::mem::size_of::<AlphaChar>() as libc::c_ulong).wrapping_mul(
                    (key_len as libc::c_ulong)
                        .wrapping_add(trie_char_strlen(tail_str))
                        .wrapping_add(1 as libc::c_int as libc::c_ulong),
                ),
            ) as *mut AlphaChar;
            alpha_p = alpha_key;
            i = key_len;
            while i > 0 as libc::c_int {
                let fresh0 = key_p;
                key_p = key_p.offset(1);
                let fresh1 = alpha_p;
                alpha_p = alpha_p.offset(1);
                *fresh1 = alpha_map_trie_to_char((*(*s).trie).alpha_map.as_ref(), *fresh0);
                i -= 1;
            }
        }
        while '\0' as i32 != *tail_str as libc::c_int {
            let fresh2 = tail_str;
            tail_str = tail_str.offset(1);
            let fresh3 = alpha_p;
            alpha_p = alpha_p.offset(1);
            *fresh3 = alpha_map_trie_to_char((*(*s).trie).alpha_map.as_ref(), *fresh2);
        }
        *alpha_p = 0 as libc::c_int as AlphaChar;
        return alpha_key;
    }

    pub unsafe fn get_data(iter: *const TrieIterator) -> TrieData {
        let s: *const TrieState = (*iter).state;
        let tail_index: TrieIndex;
        if s.is_null() {
            return -(1 as libc::c_int);
        }
        if (*s).is_suffix == 0 {
            if !((*(*(*s).trie).da).get_base((*s).index) < 0 as libc::c_int) {
                return -(1 as libc::c_int);
            }
            tail_index = -(*(*(*s).trie).da).get_base((*s).index);
        } else {
            tail_index = (*s).index;
        }
        return (*(*s).trie).tail.get_data(tail_index);
    }
}
#[cfg(test)]
mod tests {
    use crate::trie::AlphaChar;
    use crate::{trie::Trie, DatrieResult};

    use crate::alpha_map::AlphaMap;

    #[test]
    fn get_serialized_size_works() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x00, 0xff)?;
        let trie = Trie::new(&alpha_map)?;
        let size = trie.get_serialized_size();
        assert_eq!(size, 52);
        Ok(())
    }
    #[test]
    fn serialize_to_slice_works() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x00, 0xff)?;
        let mut trie = Trie::new(&alpha_map)?;
        unsafe {
            Trie::store(&mut trie, ['a' as AlphaChar, 0x0000].as_ptr(), 2);
        }
        let size = trie.get_serialized_size();
        let mut serialized_data = Vec::with_capacity(size);
        trie.serialize_safe(&mut serialized_data)?;
        let mut serialized_to_slice = vec![0; size];
        let serialized_size = trie
            .serialize_to_slice(serialized_to_slice.as_mut_slice())
            .unwrap();
        assert_eq!(serialized_size, size);
        for (i, (l, r)) in serialized_data
            .iter()
            .zip(serialized_to_slice.iter())
            .enumerate()
        {
            assert_eq!(l, r, "imdex {} fsilrd", i);
        }
        assert_eq!(serialized_data, serialized_to_slice);
        Ok(())
    }
}
