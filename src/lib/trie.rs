use std::path::Path;
use std::{fs, io};

use crate::fileutils::{ReadExt, ReadSeekExt};
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
pub type TrieChar = libc::c_uchar;
pub type TrieIndex = int32;
pub type TrieData = int32;
pub type FILE = libc::FILE;
// #[derive(Copy, Clone)]
// #[repr(C)]
#[derive(Debug)]
pub struct Trie {
    pub alpha_map: Box<AlphaMap>,
    pub da: *mut DArray,
    pub tail: *mut Tail,
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
        let da = unsafe { da_new() };
        if da.is_null() {
            return Err(DatrieError::new(ErrorKind::Memory, "malloc failed".into()));
        }
        let tail = unsafe { tail_new() };
        if tail.is_null() {
            unsafe {
                da_free(da);
            }
            return Err(DatrieError::new(ErrorKind::Memory, "malloc failed".into()));
        }
        Ok(Trie {
            alpha_map,
            da,
            tail,
            is_dirty: DA_TRUE,
        })
    }

    pub unsafe fn new_from_file(path: *const libc::c_char) -> DatrieResult<Trie> {
        let mut trie_file = fopen(path, b"rb\0" as *const u8 as *const libc::c_char);
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
        let da = da_fread(file);
        if da.is_null() {
            return Err(DatrieError::new(ErrorKind::Memory, "malloc failed".into()));
        }
        let tail = tail_fread(file);
        if tail.is_null() {
            da_free(da);
            return Err(DatrieError::new(ErrorKind::Memory, "malloc failed".into()));
        }

        return Ok(Trie {
            alpha_map,
            da,
            tail,
            is_dirty: DA_FALSE,
        });
    }
    pub fn fread_safe<R: ReadExt + io::Seek>(reader: &mut R) -> DatrieResult<Trie> {
        let alpha_map = Box::new(AlphaMap::fread_bin_safe(reader)?);
        let da = DArray::fread_safe(reader)?;
        let tail = Tail::fread_safe(reader)?;
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
impl Drop for Trie {
    fn drop(&mut self) {
        unsafe {
            // drop((*trie).alpha_map.as_mut());
            da_free((*self).da);
            tail_free((*self).tail);
            // free(trie as *mut libc::c_void);
        }
    }
}
impl Trie {
    pub unsafe fn save(mut trie: *mut Trie, mut path: *const libc::c_char) -> libc::c_int {
        let mut file: *mut FILE = 0 as *mut FILE;
        let mut res: libc::c_int = 0 as libc::c_int;
        file = fopen(path, b"wb+\0" as *const u8 as *const libc::c_char);
        if file.is_null() {
            return -(1 as libc::c_int);
        }
        res = Trie::fwrite(trie, file);
        fclose(file);
        return res;
    }
    pub fn save_safe(&mut self, path: &Path) -> DatrieResult<()> {
        let mut file = fs::File::create(path)?;
        self.serialize_safe(&mut file)?;
        Ok(())
    }
    pub unsafe fn get_serialized_size(mut trie: *mut Trie) -> size_t {
        return (alpha_map_get_serialized_size((*trie).alpha_map.as_ref()))
            .wrapping_add(da_get_serialized_size((*trie).da))
            .wrapping_add(tail_get_serialized_size((*trie).tail));
    }
    pub unsafe fn serialize(mut trie: *mut Trie, mut ptr: *mut uint8) {
        let mut ptr1: *mut uint8 = ptr;
        alpha_map_serialize_bin((*trie).alpha_map.as_ref(), &mut ptr1);
        da_serialize((*trie).da, &mut ptr1);
        tail_serialize((*trie).tail, &mut ptr1);
        (*trie).is_dirty = DA_FALSE;
    }
    pub fn serialize_safe(&mut self, write: &mut dyn std::io::Write) -> DatrieResult<()> {
        let size = unsafe { Self::get_serialized_size(self as *mut Trie) } as usize;
        let buf: Vec<u8> = Vec::with_capacity(size);
        let mut buf = std::mem::ManuallyDrop::new(buf);
        let buf_cap = buf.capacity();
        let buf_ptr = buf.as_mut_ptr();
        unsafe {
            Self::serialize(self as *mut Trie, buf_ptr);
        }
        let buf = unsafe { Vec::from_raw_parts(buf_ptr, size, buf_cap) };
        write.write_all(&buf)?;
        Ok(())
    }
    pub unsafe fn fwrite(mut trie: *mut Trie, mut file: *mut FILE) -> libc::c_int {
        if alpha_map_fwrite_bin((*trie).alpha_map.as_ref(), file) != 0 as libc::c_int {
            return -(1 as libc::c_int);
        }
        if da_fwrite((*trie).da, file) != 0 as libc::c_int {
            return -(1 as libc::c_int);
        }
        if tail_fwrite((*trie).tail, file) != 0 as libc::c_int {
            return -(1 as libc::c_int);
        }
        (*trie).is_dirty = DA_FALSE;
        return 0 as libc::c_int;
    }
    pub unsafe fn is_dirty(mut trie: *const Trie) -> Bool {
        return (*trie).is_dirty;
    }
    pub unsafe fn retrieve(
        mut trie: *const Trie,
        mut key: *const AlphaChar,
        mut o_data: *mut TrieData,
    ) -> Bool {
        let mut s: TrieIndex = 0;
        let mut suffix_idx: libc::c_short = 0;
        let mut p: *const AlphaChar = 0 as *const AlphaChar;
        s = da_get_root((*trie).da);
        p = key;
        while !(da_get_base((*trie).da, s) < 0 as libc::c_int) {
            let mut tc: TrieIndex = alpha_map_char_to_trie((*trie).alpha_map.as_ref(), *p);
            if 0x7fffffff as libc::c_int == tc {
                return DA_FALSE;
            }
            if da_walk((*trie).da, &mut s, tc as TrieChar) as u64 == 0 {
                return DA_FALSE;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
            p;
        }
        s = -da_get_base((*trie).da, s);
        suffix_idx = 0 as libc::c_int as libc::c_short;
        loop {
            let mut tc_0: TrieIndex = alpha_map_char_to_trie((*trie).alpha_map.as_ref(), *p);
            if 0x7fffffff as libc::c_int == tc_0 {
                return DA_FALSE;
            }
            if tail_walk_char((*trie).tail, s, &mut suffix_idx, tc_0 as TrieChar) as u64 == 0 {
                return DA_FALSE;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
            p;
        }
        if !o_data.is_null() {
            *o_data = tail_get_data((*trie).tail, s);
        }
        return DA_TRUE;
    }

    pub unsafe fn store(
        mut trie: *mut Trie,
        mut key: *const AlphaChar,
        mut data: TrieData,
    ) -> Bool {
        return Trie::store_conditionally(trie, key, data, DA_TRUE);
    }

    pub unsafe fn store_if_absent(
        mut trie: *mut Trie,
        mut key: *const AlphaChar,
        mut data: TrieData,
    ) -> Bool {
        return Trie::store_conditionally(trie, key, data, DA_FALSE);
    }
    unsafe fn store_conditionally(
        mut trie: *mut Trie,
        mut key: *const AlphaChar,
        mut data: TrieData,
        mut is_overwrite: Bool,
    ) -> Bool {
        dbg!(&*trie);
        dbg!(&*key);
        let mut s: TrieIndex = 0;
        let mut t: TrieIndex = 0;
        let mut suffix_idx: libc::c_short = 0;
        let mut p: *const AlphaChar = 0 as *const AlphaChar;
        let mut sep: *const AlphaChar = 0 as *const AlphaChar;
        s = da_get_root((*trie).da);
        p = key;
        while !(da_get_base((*trie).da, s) < 0 as libc::c_int) {
            let mut tc: TrieIndex = alpha_map_char_to_trie((*trie).alpha_map.as_ref(), *p);
            if 0x7fffffff as libc::c_int == tc {
                return DA_FALSE;
            }
            if da_walk((*trie).da, &mut s, tc as TrieChar) as u64 == 0 {
                let mut key_str: *mut TrieChar = 0 as *mut TrieChar;
                let mut res: Bool = DA_FALSE;
                key_str = alpha_map_char_to_trie_str((*trie).alpha_map.as_ref(), p);
                if key_str.is_null() {
                    return DA_FALSE;
                }
                res = Trie::branch_in_branch(trie, s, key_str, data);
                free(key_str as *mut libc::c_void);
                return res;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
            p;
        }
        sep = p;
        t = -da_get_base((*trie).da, s);
        suffix_idx = 0 as libc::c_int as libc::c_short;
        loop {
            let mut tc_0: TrieIndex = alpha_map_char_to_trie((*trie).alpha_map.as_ref(), *p);
            if 0x7fffffff as libc::c_int == tc_0 {
                return DA_FALSE;
            }
            if tail_walk_char((*trie).tail, t, &mut suffix_idx, tc_0 as TrieChar) as u64 == 0 {
                let mut tail_str: *mut TrieChar = 0 as *mut TrieChar;
                let mut res_0: Bool = DA_FALSE;
                tail_str = alpha_map_char_to_trie_str((*trie).alpha_map.as_ref(), sep);
                if tail_str.is_null() {
                    return DA_FALSE;
                }
                res_0 = Trie::branch_in_tail(trie, s, tail_str, data);
                free(tail_str as *mut libc::c_void);
                return res_0;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
            p;
        }
        if is_overwrite as u64 == 0 {
            return DA_FALSE;
        }
        tail_set_data((*trie).tail, t, data);
        (*trie).is_dirty = DA_TRUE;
        return DA_TRUE;
    }
    unsafe fn branch_in_branch(
        mut trie: *mut Trie,
        mut sep_node: TrieIndex,
        mut suffix: *const TrieChar,
        mut data: TrieData,
    ) -> Bool {
        let mut new_da: TrieIndex = 0;
        let mut new_tail: TrieIndex = 0;
        new_da = da_insert_branch((*trie).da, sep_node, *suffix);
        if 0 as libc::c_int == new_da {
            return DA_FALSE;
        }
        if '\0' as i32 != *suffix as libc::c_int {
            suffix = suffix.offset(1);
            suffix;
        }
        new_tail = tail_add_suffix((*trie).tail, suffix);
        tail_set_data((*trie).tail, new_tail, data);
        da_set_base((*trie).da, new_da, -new_tail);
        (*trie).is_dirty = DA_TRUE;
        return DA_TRUE;
    }
    unsafe fn branch_in_tail(
        mut trie: *mut Trie,
        mut sep_node: TrieIndex,
        mut suffix: *const TrieChar,
        mut data: TrieData,
    ) -> Bool {
        let mut current_block: u64;
        let mut old_tail: TrieIndex = 0;
        let mut old_da: TrieIndex = 0;
        let mut s: TrieIndex = 0;
        let mut old_suffix: *const TrieChar = 0 as *const TrieChar;
        let mut p: *const TrieChar = 0 as *const TrieChar;
        old_tail = -da_get_base((*trie).da, sep_node);
        old_suffix = tail_get_suffix((*trie).tail, old_tail);
        if old_suffix.is_null() {
            return DA_FALSE;
        }
        p = old_suffix;
        s = sep_node;
        loop {
            if !(*p as libc::c_int == *suffix as libc::c_int) {
                current_block = 6937071982253665452;
                break;
            }
            let mut t: TrieIndex = da_insert_branch((*trie).da, s, *p);
            if 0 as libc::c_int == t {
                current_block = 13151848498364941746;
                break;
            }
            s = t;
            p = p.offset(1);
            p;
            suffix = suffix.offset(1);
            suffix;
        }
        match current_block {
            6937071982253665452 => {
                old_da = da_insert_branch((*trie).da, s, *p);
                if !(0 as libc::c_int == old_da) {
                    if '\0' as i32 != *p as libc::c_int {
                        p = p.offset(1);
                        p;
                    }
                    tail_set_suffix((*trie).tail, old_tail, p);
                    da_set_base((*trie).da, old_da, -old_tail);
                    return Trie::branch_in_branch(trie, s, suffix, data);
                }
            }
            _ => {}
        }
        da_prune_upto((*trie).da, sep_node, s);
        da_set_base((*trie).da, sep_node, -old_tail);
        return DA_FALSE;
    }
}

impl Trie {
    pub unsafe fn delete(mut trie: *mut Trie, mut key: *const AlphaChar) -> Bool {
        let mut s: TrieIndex = 0;
        let mut t: TrieIndex = 0;
        let mut suffix_idx: libc::c_short = 0;
        let mut p: *const AlphaChar = 0 as *const AlphaChar;
        s = da_get_root((*trie).da);
        p = key;
        while !(da_get_base((*trie).da, s) < 0 as libc::c_int) {
            let mut tc: TrieIndex = alpha_map_char_to_trie((*trie).alpha_map.as_ref(), *p);
            if 0x7fffffff as libc::c_int == tc {
                return DA_FALSE;
            }
            if da_walk((*trie).da, &mut s, tc as TrieChar) as u64 == 0 {
                return DA_FALSE;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
            p;
        }
        t = -da_get_base((*trie).da, s);
        suffix_idx = 0 as libc::c_int as libc::c_short;
        loop {
            let mut tc_0: TrieIndex = alpha_map_char_to_trie((*trie).alpha_map.as_ref(), *p);
            if 0x7fffffff as libc::c_int == tc_0 {
                return DA_FALSE;
            }
            if tail_walk_char((*trie).tail, t, &mut suffix_idx, tc_0 as TrieChar) as u64 == 0 {
                return DA_FALSE;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
            p;
        }
        tail_delete((*trie).tail, t);
        da_set_base((*trie).da, s, 0 as libc::c_int);
        da_prune((*trie).da, s);
        (*trie).is_dirty = DA_TRUE;
        return DA_TRUE;
    }

    pub unsafe fn enumerate(
        mut trie: *const Trie,
        mut enum_func: TrieEnumFunc,
        mut user_data: *mut libc::c_void,
    ) -> Bool {
        let mut root: *mut TrieState = 0 as *mut TrieState;
        let mut iter: *mut TrieIterator = 0 as *mut TrieIterator;
        let mut cont: Bool = DA_TRUE;
        root = Trie::root(&*trie);
        if root.is_null() as libc::c_int as libc::c_long != 0 {
            return DA_FALSE;
        }
        iter = TrieIterator::new(root);
        if iter.is_null() as libc::c_int as libc::c_long != 0 {
            TrieState::free(root);
            return DA_FALSE;
        } else {
            while cont as libc::c_uint != 0 && TrieIterator::next(iter) as libc::c_uint != 0 {
                let mut key: *mut AlphaChar = TrieIterator::get_key(iter);
                let mut data: TrieData = TrieIterator::get_data(iter);
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
            da_get_root((*self).da),
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

    pub unsafe fn trie_state_copy(mut dst: *mut TrieState, mut src: *const TrieState) {
        *dst = *src;
    }
    pub unsafe fn trie_state_clone(mut s: *const TrieState) -> *mut TrieState {
        return TrieState::new((*s).trie, (*s).index, (*s).suffix_idx, (*s).is_suffix);
    }

    pub unsafe fn free(mut s: *mut TrieState) {
        free(s as *mut libc::c_void);
    }

    pub unsafe fn rewind(mut s: *mut TrieState) {
        (*s).index = da_get_root((*(*s).trie).da);
        (*s).is_suffix = DA_FALSE as libc::c_int as libc::c_short;
    }

    pub unsafe fn walk(mut s: *mut TrieState, mut c: AlphaChar) -> Bool {
        let mut tc: TrieIndex = alpha_map_char_to_trie((*(*s).trie).alpha_map.as_ref(), c);
        if (0x7fffffff as libc::c_int == tc) as libc::c_int as libc::c_long != 0 {
            return DA_FALSE;
        }
        if (*s).is_suffix == 0 {
            let mut ret: Bool = DA_FALSE;
            ret = da_walk((*(*s).trie).da, &mut (*s).index, tc as TrieChar);
            if ret as libc::c_uint != 0
                && da_get_base((*(*s).trie).da, (*s).index) < 0 as libc::c_int
            {
                (*s).index = -da_get_base((*(*s).trie).da, (*s).index);
                (*s).suffix_idx = 0 as libc::c_int as libc::c_short;
                (*s).is_suffix = DA_TRUE as libc::c_int as libc::c_short;
            }
            return ret;
        } else {
            return tail_walk_char(
                (*(*s).trie).tail,
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
            return (da_get_check(
                (*(*s).trie).da,
                da_get_base((*(*s).trie).da, (*s).index) + tc as TrieChar as libc::c_int,
            ) == (*s).index) as libc::c_int as Bool;
        } else {
            return (*(tail_get_suffix((*(*s).trie).tail, (*s).index))
                .offset((*s).suffix_idx as isize) as libc::c_int
                == tc as TrieChar as libc::c_int) as libc::c_int as Bool;
        };
    }

    pub unsafe fn is_terminal(s: *const TrieState) -> bool {
        TrieState::is_walkable(s, 0) == DA_TRUE
    }

    pub unsafe fn walkable_chars(
        mut s: *const TrieState,
        mut chars: *mut AlphaChar,
        mut chars_nelm: libc::c_int,
    ) -> libc::c_int {
        let mut syms_num: libc::c_int = 0 as libc::c_int;
        if (*s).is_suffix == 0 {
            let mut syms: *mut Symbols = da_output_symbols((*(*s).trie).da, (*s).index);
            let mut i: libc::c_int = 0;
            syms_num = symbols_num(syms);
            i = 0 as libc::c_int;
            while i < syms_num && i < chars_nelm {
                let mut tc: TrieChar = symbols_get(syms, i);
                *chars.offset(i as isize) =
                    alpha_map_trie_to_char((*(*s).trie).alpha_map.as_ref(), tc);
                i += 1;
                i;
            }
            symbols_free(syms);
        } else {
            let mut suffix: *const TrieChar = tail_get_suffix((*(*s).trie).tail, (*s).index);
            *chars.offset(0 as libc::c_int as isize) = alpha_map_trie_to_char(
                (*(*s).trie).alpha_map.as_ref(),
                *suffix.offset((*s).suffix_idx as isize),
            );
            syms_num = 1 as libc::c_int;
        }
        return syms_num;
    }

    pub unsafe fn is_single(mut s: *const TrieState) -> Bool {
        return (*s).is_suffix as Bool;
    }

    pub unsafe fn get_data(mut s: *const TrieState) -> TrieData {
        if s.is_null() {
            return -(1 as libc::c_int);
        }
        if (*s).is_suffix == 0 {
            let mut index: TrieIndex = (*s).index;
            if da_walk((*(*s).trie).da, &mut index, '\0' as i32 as TrieChar) as u64 != 0 {
                if da_get_base((*(*s).trie).da, index) < 0 as libc::c_int {
                    index = -da_get_base((*(*s).trie).da, index);
                    return tail_get_data((*(*s).trie).tail, index);
                }
            }
        } else if *(tail_get_suffix((*(*s).trie).tail, (*s).index)).offset((*s).suffix_idx as isize)
            as libc::c_int
            == '\0' as i32
        {
            return tail_get_data((*(*s).trie).tail, (*s).index);
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
            sep = da_first_separate((*(*s).trie).da, (*s).index, (*iter).key);
            if 0 as libc::c_int == sep {
                return DA_FALSE;
            }
            (*s).index = sep;
            return DA_TRUE;
        }
        if (*s).is_suffix != 0 {
            return DA_FALSE;
        }
        sep = da_next_separate(
            (*(*s).trie).da,
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
            tail_str = tail_get_suffix((*(*s).trie).tail, (*s).index);
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
            tail_idx = -da_get_base((*(*s).trie).da, (*s).index);
            tail_str = tail_get_suffix((*(*s).trie).tail, tail_idx);
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
                i;
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

    pub unsafe fn get_data(mut iter: *const TrieIterator) -> TrieData {
        let mut s: *const TrieState = (*iter).state;
        let mut tail_index: TrieIndex = 0;
        if s.is_null() {
            return -(1 as libc::c_int);
        }
        if (*s).is_suffix == 0 {
            if !(da_get_base((*(*s).trie).da, (*s).index) < 0 as libc::c_int) {
                return -(1 as libc::c_int);
            }
            tail_index = -da_get_base((*(*s).trie).da, (*s).index);
        } else {
            tail_index = (*s).index;
        }
        return tail_get_data((*(*s).trie).tail, tail_index);
    }
}
