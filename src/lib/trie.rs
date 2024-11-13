use std::ffi::CStr;
use std::path::Path;
use std::{fs, io};

pub use crate::trie_str::TrieChar; //, TrieString};
pub(crate) use crate::trie_str::{TrieCharStr, TrieCharString};

use crate::fileutils::{CFile, ReadExt};

use crate::{alpha_map::*, darray::*, AlphaStr};
use crate::{trie_string::*, DatrieError, DatrieResult, ErrorKind};
use ::libc;

use self::tail::Tail;

mod tail;

extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
}
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type AlphaChar = u32;
pub type TrieIndex = i32;
pub type TrieData = i32;
pub type FILE = libc::FILE;
// #[derive(Copy, Clone)]
// #[repr(C)]
#[derive(Debug)]
pub struct Trie {
    pub alpha_map: AlphaMap,
    pub da: Box<DArray>,
    pub tail: Tail,
    pub is_dirty: bool,
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
        let alpha_map = alpha_map.clone();
        let da = Box::new(DArray::new()?);
        let tail = Tail::new();

        Ok(Trie {
            alpha_map,
            da,
            tail,
            is_dirty: true,
        })
    }

    pub unsafe fn new_from_file(path: *const libc::c_char) -> DatrieResult<Trie> {
        let trie_file = fopen(path, b"rb\0" as *const u8 as *const libc::c_char);

        let cfile = CFile::new(trie_file, true);
        if let Some(mut cfile) = cfile {
            let result = Trie::fread_safe(&mut cfile);

            result
        } else {
            Err(DatrieError::new(
                ErrorKind::Io,
                "failed to open file".into(),
            ))
        }
    }
    pub fn from_path(path: &Path) -> DatrieResult<Trie> {
        let trie_file = fs::File::open(path)?;
        let mut reader = io::BufReader::new(trie_file);
        Trie::fread_safe(&mut reader)
    }

    pub fn fread_safe<R: ReadExt + io::Seek>(reader: &mut R) -> DatrieResult<Trie> {
        let alpha_map = AlphaMap::fread_bin_safe(reader)?;
        let da = Box::new(DArray::fread_safe(reader)?);
        let tail = Tail::fread_safe(reader)?;
        Ok(Trie {
            alpha_map,
            da,
            tail,
            is_dirty: false,
        })
    }
}

impl Trie {
    pub unsafe fn save(&mut self, path: &CStr) -> DatrieResult<()> {
        let file: *mut FILE = fopen(path.as_ptr(), b"wb+\0" as *const u8 as *const libc::c_char);

        let cfile = CFile::new(file, true);
        if let Some(mut cfile) = cfile {
            let res = self.serialize_safe(&mut cfile);

            res
        } else {
            Err(DatrieError::new(
                ErrorKind::Io,
                format!("failed to open '{:?}'", path),
            ))
        }
    }
    pub fn save_safe(&mut self, path: &Path) -> DatrieResult<()> {
        let mut file = fs::File::create(path)?;
        self.serialize_safe(&mut file)?;
        Ok(())
    }

    pub fn get_serialized_size(&self) -> usize {
        self.alpha_map.get_serialized_size()
            + self.da.get_serialized_size()
            + self.tail.get_serialized_size()
    }

    pub fn serialize_safe(&mut self, mut writer: impl std::io::Write) -> DatrieResult<()> {
        self.alpha_map.serialize(&mut writer)?;
        self.da.serialize(&mut writer)?;
        self.tail.serialize(&mut writer)?;
        self.is_dirty = false;
        Ok(())
    }
    pub fn serialize_to_slice(&mut self, buf: &mut [u8]) -> DatrieResult<usize> {
        let mut start = self.alpha_map.serialize_to_slice(buf)?;
        start += self.da.serialize_to_slice(&mut buf[start..])?;
        start += self.tail.serialize(&mut buf[start..])?;
        self.is_dirty = false;
        Ok(start)
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    pub unsafe fn retrieve(&self, key: &AlphaStr, o_data: *mut TrieData) -> bool {
        let mut s = self.da.get_root();
        let key_slice = key.to_slice_with_nul();
        let mut p = key_slice;

        while self.da.get_base(s) >= 0 as libc::c_int {
            let Some(tc) = self.alpha_map.char_to_trie(p[0]) else {
                return false;
            };

            if self.da.walk(&mut s, tc as TrieChar) as u64 == 0 {
                return false;
            }
            if p[0] == 0 {
                break;
            }
            p = &p[1..];
        }
        s = -self.da.get_base(s);
        let mut suffix_idx: libc::c_short = 0;
        loop {
            let Some(tc_0) = self.alpha_map.char_to_trie(p[0]) else {
                return false;
            };

            if unsafe { self.tail.walk_char(s, &mut suffix_idx, tc_0 as TrieChar) } as u64 == 0 {
                return false;
            }

            if p[0] == 0 {
                break;
            }
            p = &p[1..];
        }
        if !o_data.is_null() {
            unsafe {
                *o_data = self.tail.get_data(s);
            }
        }
        true
    }

    pub fn store2(&mut self, key: &AlphaStr, data: TrieData) -> bool {
        unsafe { self.store_conditionally(key.as_ptr(), data, true) }
    }
    pub fn store(&mut self, key: &AlphaStr, data: TrieData) -> bool {
        self.store_conditionally2(key, data, true)
    }

    pub fn store_if_absent(&mut self, key: &AlphaStr, data: TrieData) -> bool {
        unsafe { self.store_conditionally(key.as_ptr(), data, false) }
    }

    unsafe fn store_conditionally(
        &mut self,
        key: *const AlphaChar,
        data: TrieData,
        is_overwrite: bool,
    ) -> bool {
        let mut s = self.da.get_root();
        let mut p: *const AlphaChar = key;
        while self.da.get_base(s) >= 0 as libc::c_int {
            let tc: TrieIndex = match self.alpha_map.char_to_trie(*p) {
                Some(tc) => tc,
                None => return false,
            };
            if self.da.walk(&mut s, tc as TrieChar) as u64 == 0 {
                let key_str = self.alpha_map.char_to_trie_str(p);
                if key_str.is_null() {
                    return false;
                }
                dbg!(TrieCharStr::from_ptr(key_str));
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
        dbg!(*sep);
        let t = -(*self.da).get_base(s);
        // suffix_idx = 0 as libc::c_int as libc::c_short;
        let mut suffix_idx: libc::c_short = 0;
        loop {
            let tc_0: TrieIndex = match self.alpha_map.char_to_trie(*p) {
                Some(tc) => tc,
                None => return false,
            };
            dbg!(&tc_0);
            if self.tail.walk_char(t, &mut suffix_idx, tc_0 as TrieChar) as u64 == 0 {
                dbg!(&suffix_idx);
                // let mut tail_str: *mut TrieChar = 0 as *mut TrieChar;
                // let mut res_0: Bool = DA_FALSE;
                let tail_str = self.alpha_map.char_to_trie_str(sep);
                if tail_str.is_null() {
                    return false;
                }
                dbg!(TrieCharStr::from_ptr(tail_str));
                let res_0 = self.branch_in_tail(s, tail_str, data);
                free(tail_str as *mut libc::c_void);
                return res_0;
            }
            if 0 as libc::c_int as libc::c_uint == *p {
                break;
            }
            p = p.offset(1);
        }
        if !is_overwrite {
            return false;
        }
        self.tail.set_data(t, data);
        self.is_dirty = true;
        true
    }
    fn store_conditionally2(&mut self, key: &AlphaStr, data: TrieData, is_overwrite: bool) -> bool {
        let mut s = self.da.get_root();
        let mut p = key.to_slice_with_nul();
        while self.da.get_base(s) >= 0 as libc::c_int {
            let Some(tc) = self.alpha_map.char_to_trie(p[0]) else {
                return false;
            };
            if unsafe { self.da.walk(&mut s, tc as TrieChar) } as u64 == 0 {
                let Some(key_str) = self
                    .alpha_map
                    .char_to_trie_str2(AlphaStr::from_slice_with_nul(p).unwrap())
                else {
                    return false;
                };
                dbg!(&key_str);
                let res = self.branch_in_branch2(s, key_str.as_trie_str(), data);
                return res;
            }
            if p[0] == 0 {
                break;
            }
            p = &p[1..];
        }
        let sep = p;
        dbg!(sep[0]);
        let t = -(*self.da).get_base(s);
        // suffix_idx = 0 as libc::c_int as libc::c_short;
        let mut suffix_idx = 0;
        loop {
            let Some(tc_0) = self.alpha_map.char_to_trie(p[0]) else {
                return false;
            };
            dbg!(&tc_0);
            if !self.tail.walk_char2(t, &mut suffix_idx, tc_0 as TrieChar) {
                dbg!(&suffix_idx);
                // let mut tail_str: *mut TrieChar = 0 as *mut TrieChar;
                // let mut res_0: Bool = DA_FALSE;
                if let Some(tail_str) = self
                    .alpha_map
                    .char_to_trie_str2(AlphaStr::from_slice_with_nul(sep).unwrap())
                {
                    dbg!(&tail_str);
                    let res_0 = self.branch_in_tail2(s, tail_str, data);
                    // free(tail_str as *mut libc::c_void);
                    return res_0;
                } else {
                    return false;
                }
            }
            if p[0] == 0 {
                break;
            }
            p = &p[1..];
        }
        if is_overwrite {
            return false;
        }
        self.tail.set_data(t, data);
        self.is_dirty = true;
        true
    }
    unsafe fn branch_in_branch(
        &mut self,
        sep_node: TrieIndex,
        // TODO accept TrieCharStr (or TrieCharString)
        mut suffix: *const TrieChar,

        data: TrieData,
    ) -> bool {
        dbg!(*suffix);
        let new_da = self.da.insert_branch(sep_node, *suffix);
        if 0 as libc::c_int == new_da {
            return false;
        }
        if '\0' as i32 != *suffix as libc::c_int {
            suffix = suffix.offset(1);
        }
        dbg!(TrieCharStr::from_ptr(suffix));
        let new_tail = self.tail.add_suffix(suffix);
        self.tail.set_data(new_tail, data);
        self.da.set_base(new_da, -new_tail);
        self.is_dirty = true;
        true
    }
    fn branch_in_branch2(
        &mut self,
        sep_node: TrieIndex,
        suffix: &TrieCharStr,

        data: TrieData,
    ) -> bool {
        let mut suffix_bytes = suffix.to_bytes_with_nul();
        dbg!(suffix_bytes[0]);
        let new_da = unsafe { self.da.insert_branch(sep_node, suffix_bytes[0]) };
        if 0 as libc::c_int == new_da {
            return false;
        }
        if suffix_bytes[0] != 0 {
            suffix_bytes = &suffix_bytes[1..];
        }
        // if '\0' as i32 != *suffix as libc::c_int {
        //     suffix = suffix.offset(1);
        // }
        dbg!(suffix_bytes);
        let new_tail = self
            .tail
            .add_suffix2(TrieCharString::from_vec_with_nul(suffix_bytes.to_vec()).unwrap());
        self.tail.set_data(new_tail, data);
        self.da.set_base(new_da, -new_tail);
        self.is_dirty = true;
        true
    }
    unsafe fn branch_in_tail(
        &mut self,
        sep_node: TrieIndex,
        // TODO accept TrieCharStr (or TrieCharString)
        mut suffix: *const TrieChar,
        data: TrieData,
    ) -> bool {
        let current_block: u64;

        let old_tail = -(*self.da).get_base(sep_node);
        let old_suffix = self.tail.get_suffix(old_tail);
        if old_suffix.is_null() {
            return false;
        }
        let mut p = old_suffix;
        let mut s = sep_node;
        dbg!(TrieCharStr::from_ptr(suffix));
        loop {
            dbg!(TrieCharStr::from_ptr(p));

            if *p as libc::c_int != *suffix as libc::c_int {
                current_block = 6937071982253665452;
                break;
            }
            dbg!(*p);
            let t: TrieIndex = self.da.insert_branch(s, *p);
            if 0 as libc::c_int == t {
                current_block = 13151848498364941746;
                break;
            }
            s = t;
            p = p.offset(1);
            suffix = suffix.offset(1);
        }
        dbg!(TrieCharStr::from_ptr(p));
        if current_block == 6937071982253665452 {
            dbg!(*p);
            let old_da = self.da.insert_branch(s, *p);
            if 0 as libc::c_int != old_da {
                if '\0' as i32 != *p as libc::c_int {
                    p = p.offset(1);
                }
                dbg!(TrieCharStr::from_ptr(p));
                self.tail.set_suffix(old_tail, p);
                self.da.set_base(old_da, -old_tail);
                return self.branch_in_branch(s, suffix, data);
            }
        }
        self.da.prune_upto(sep_node, s);
        self.da.set_base(sep_node, -old_tail);
        false
    }
    fn branch_in_tail2(
        &mut self,
        sep_node: TrieIndex,
        // TODO accept TrieCharStr (or TrieCharString)
        suffix: TrieCharString,
        data: TrieData,
    ) -> bool {
        let current_block: u64;

        let old_tail = -(*self.da).get_base(sep_node);
        let Some(old_suffix) = self.tail.take_suffix(old_tail) else {
            return false;
        };
        let mut p = old_suffix.to_bytes_with_nul();
        let mut suffix_bytes = suffix.to_bytes_with_nul();
        let mut s = sep_node;
        dbg!(&suffix_bytes);
        loop {
            dbg!(p);

            if p[0] != suffix_bytes[0] {
                current_block = 6937071982253665452;
                break;
            }
            dbg!(&p[0]);
            let t: TrieIndex = unsafe { self.da.insert_branch(s, p[0]) };
            if 0 as libc::c_int == t {
                current_block = 13151848498364941746;
                break;
            }
            s = t;
            p = &p[1..];
            suffix_bytes = &suffix_bytes[1..];
        }
        dbg!(&p);
        if current_block == 6937071982253665452 {
            dbg!(p);
            let old_da = unsafe { self.da.insert_branch(s, p[0]) };
            if 0 as libc::c_int != old_da {
                if p[0] != 0 {
                    p = &p[1..];
                }

                dbg!(&p);
                self.tail.set_suffix2(
                    old_tail,
                    TrieCharString::from_vec_with_nul(p.to_vec()).unwrap(),
                );
                self.da.set_base(old_da, -old_tail);
                return self.branch_in_branch2(
                    s,
                    &TrieCharStr::from_bytes_with_nul(suffix_bytes).unwrap(),
                    data,
                );
            }
        }
        self.da.prune_upto(sep_node, s);
        self.da.set_base(sep_node, -old_tail);
        false
    }
}

impl Trie {
    pub fn delete(&mut self, key: &AlphaStr) -> bool {
        // let mut s: TrieIndex = 0;
        // let mut t: TrieIndex = 0;
        // let mut suffix_idx: libc::c_short = 0;
        // let mut p: *const AlphaChar = 0 as *const AlphaChar;
        let mut s = (*self.da).get_root();
        let key_slice = key.to_slice_with_nul();
        let mut p = key_slice;
        while (*self.da).get_base(s) >= 0 as libc::c_int {
            let Some(tc) = self.alpha_map.char_to_trie(p[0]) else {
                return false;
            };
            if unsafe { self.da.walk(&mut s, tc as TrieChar) } as u64 == 0 {
                return false;
            }
            if p[0] == 0 {
                break;
            }
            p = &p[1..];
        }
        let t = -(*self.da).get_base(s);
        // suffix_idx = 0 as libc::c_int as libc::c_short;
        let mut suffix_idx: libc::c_short = 0;
        loop {
            let Some(tc_0) = self.alpha_map.char_to_trie(p[0]) else {
                return false;
            };
            if unsafe { self.tail.walk_char(t, &mut suffix_idx, tc_0 as TrieChar) as u64 == 0 } {
                return false;
            }
            if p[0] == 0 {
                break;
            }
            p = &p[1..];
        }
        unsafe {
            self.tail.delete(t);
        }
        self.da.set_base(s, 0 as libc::c_int);
        self.da.prune(s);
        self.is_dirty = true;
        true
    }

    pub unsafe fn enumerate(
        &self,
        // mut trie: *const Trie,
        enum_func: TrieEnumFunc,
        user_data: *mut libc::c_void,
    ) -> bool {
        // let mut root: *mut TrieState = 0 as *mut TrieState;
        // let mut iter: *mut TrieIterator = 0 as *mut TrieIterator;
        let mut cont: bool = true;
        let root = self.root();
        if root.is_null() {
            return false;
        }
        let iter = TrieIterator::new(root);
        if iter.is_null() {
            TrieState::free(root);
            false
        } else {
            while cont && TrieIterator::next(iter) as libc::c_uint != 0 {
                let key: *mut AlphaChar = TrieIterator::get_key(iter);
                let data: TrieData = TrieIterator::get_data(iter);
                cont =
                    enum_func.expect("non-null function pointer")(key, data, user_data) == DA_TRUE;
                free(key as *mut libc::c_void);
            }
            TrieIterator::free(iter);
            TrieState::free(root);
            cont
        }
    }

    pub unsafe fn root(&self) -> *mut TrieState {
        TrieState::new(
            self,
            (*self.da).get_root(),
            0 as libc::c_int as libc::c_short,
            DA_FALSE as libc::c_int as libc::c_short,
        )
    }
}

impl TrieState {
    unsafe fn new(
        trie: *const Trie,
        index: TrieIndex,
        suffix_idx: libc::c_short,
        is_suffix: libc::c_short,
    ) -> *mut TrieState {
        let s: *mut TrieState =
            malloc(::core::mem::size_of::<TrieState>() as libc::c_ulong) as *mut TrieState;
        if s.is_null() as libc::c_int as libc::c_long != 0 {
            return std::ptr::null_mut::<TrieState>();
        }
        (*s).trie = trie;
        (*s).index = index;
        (*s).suffix_idx = suffix_idx;
        (*s).is_suffix = is_suffix;
        s
    }

    pub unsafe fn trie_state_copy(dst: *mut TrieState, src: *const TrieState) {
        *dst = *src;
    }
    pub unsafe fn trie_state_clone(s: *const TrieState) -> *mut TrieState {
        TrieState::new((*s).trie, (*s).index, (*s).suffix_idx, (*s).is_suffix)
    }

    pub unsafe fn free(s: *mut TrieState) {
        free(s as *mut libc::c_void);
    }

    pub unsafe fn rewind(s: *mut TrieState) {
        (*s).index = (*(*(*s).trie).da).get_root();
        (*s).is_suffix = DA_FALSE as libc::c_int as libc::c_short;
    }

    pub unsafe fn walk(s: *mut TrieState, c: AlphaChar) -> Bool {
        let alpha_map: &AlphaMap = unsafe { &(*(*s).trie).alpha_map };
        let tc: TrieIndex = match alpha_map.char_to_trie(c) {
            Some(tc) => tc,
            None => return DA_FALSE,
        };
        if (*s).is_suffix == 0 {
            let ret: Bool = (*(*s).trie).da.walk(&mut (*s).index, tc as TrieChar);
            if ret as libc::c_uint != 0
                && (*(*(*s).trie).da).get_base((*s).index) < 0 as libc::c_int
            {
                (*s).index = -(*(*(*s).trie).da).get_base((*s).index);
                (*s).suffix_idx = 0 as libc::c_int as libc::c_short;
                (*s).is_suffix = DA_TRUE as libc::c_int as libc::c_short;
            }
            ret
        } else {
            (*(*s).trie).tail.walk_char(
                // (*(*s).trie).tail.as_ref(),
                (*s).index,
                &mut (*s).suffix_idx,
                tc as TrieChar,
            )
        }
    }

    pub unsafe fn is_walkable(s: *const TrieState, c: AlphaChar) -> Bool {
        let alpha_map: &AlphaMap = unsafe { &(*(*s).trie).alpha_map };
        let tc: TrieIndex = match alpha_map.char_to_trie(c) {
            Some(tc) => tc,
            None => return DA_FALSE,
        };

        if (*s).is_suffix == 0 {
            ((*(*(*s).trie).da)
                .get_check((*(*(*s).trie).da).get_base((*s).index) + tc as TrieChar as libc::c_int)
                == (*s).index) as libc::c_int as Bool
        } else {
            (*((*(*s).trie).tail.get_suffix((*s).index)).offset((*s).suffix_idx as isize)
                as libc::c_int
                == tc as TrieChar as libc::c_int) as libc::c_int as Bool
        }
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
        let alpha_map: &AlphaMap = unsafe { &(*(*s).trie).alpha_map };
        if (*s).is_suffix == 0 {
            // let syms = da_output_symbols((*(*s).trie).da, (*s).index);
            let syms = (*(*(*s).trie).da).output_symbols((*s).index);
            syms_num = syms.num() as libc::c_int;
            let mut i = 0 as libc::c_int;
            while i < syms_num && i < chars_nelm {
                let tc: TrieChar = syms.get(i as usize);
                *chars.offset(i as isize) = alpha_map.trie_to_char(tc);
                i += 1;
            }
            // symbols_free(syms);
        } else {
            let suffix: *const TrieChar = (*(*s).trie).tail.get_suffix((*s).index);
            *chars.offset(0 as libc::c_int as isize) =
                alpha_map.trie_to_char(*suffix.offset((*s).suffix_idx as isize));
            syms_num = 1 as libc::c_int;
        }
        syms_num
    }

    pub unsafe fn is_single(s: *const TrieState) -> Bool {
        (*s).is_suffix as Bool
    }

    pub unsafe fn get_data(s: *const TrieState) -> TrieData {
        if s.is_null() {
            return -(1 as libc::c_int);
        }
        if (*s).is_suffix == 0 {
            let mut index: TrieIndex = (*s).index;
            if (*(*s).trie).da.walk(&mut index, '\0' as i32 as TrieChar) as u64 != 0
                && (*(*(*s).trie).da).get_base(index) < 0 as libc::c_int
            {
                index = -(*(*(*s).trie).da).get_base(index);
                return ((*(*s).trie).tail).get_data(index);
            }
        } else if *((*(*s).trie).tail.get_suffix((*s).index)).offset((*s).suffix_idx as isize)
            as libc::c_int
            == '\0' as i32
        {
            return (*(*s).trie).tail.get_data((*s).index);
        }
        -(1 as libc::c_int)
    }
}
impl TrieIterator {
    pub unsafe fn new(s: *mut TrieState) -> *mut TrieIterator {
        let iter: *mut TrieIterator =
            malloc(::core::mem::size_of::<TrieIterator>() as libc::c_ulong) as *mut TrieIterator;
        if iter.is_null() as libc::c_int as libc::c_long != 0 {
            return std::ptr::null_mut::<TrieIterator>();
        }
        (*iter).root = s;
        (*iter).state = std::ptr::null_mut::<TrieState>();
        (*iter).key = std::ptr::null_mut::<TrieString>();
        iter
    }

    pub unsafe fn free(iter: *mut TrieIterator) {
        if !((*iter).state).is_null() {
            TrieState::free((*iter).state);
        }
        if !((*iter).key).is_null() {
            trie_string_free((*iter).key);
        }
        free(iter as *mut libc::c_void);
    }

    pub unsafe fn next(iter: *mut TrieIterator) -> Bool {
        let mut s: *mut TrieState = (*iter).state;
        let sep: TrieIndex;
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
        DA_TRUE
    }

    pub unsafe fn get_key(iter: *const TrieIterator) -> *mut AlphaChar {
        let mut tail_str: *const TrieChar;
        let alpha_key: *mut AlphaChar;
        let mut alpha_p: *mut AlphaChar;
        let s = (*iter).state;
        if s.is_null() {
            return std::ptr::null_mut::<AlphaChar>();
        }
        if (*s).is_suffix != 0 {
            tail_str = (*(*s).trie).tail.get_suffix((*s).index);
            if tail_str.is_null() {
                return std::ptr::null_mut::<AlphaChar>();
            }
            tail_str = tail_str.offset((*s).suffix_idx as libc::c_int as isize);
            alpha_key = libc::malloc(
                (::core::mem::size_of::<AlphaChar>())
                    .wrapping_mul((trie_char_strlen(tail_str)).wrapping_add(1)),
            ) as *mut AlphaChar;
            alpha_p = alpha_key;
        } else {
            let tail_idx = -(*(*(*s).trie).da).get_base((*s).index);
            tail_str = (*(*s).trie).tail.get_suffix(tail_idx);
            if tail_str.is_null() {
                return std::ptr::null_mut::<AlphaChar>();
            }
            let key_len = trie_string_length((*iter).key) as usize;
            let mut key_p = trie_string_get_val((*iter).key) as *const TrieChar;
            alpha_key = libc::malloc(
                (::core::mem::size_of::<AlphaChar>()).wrapping_mul(
                    (key_len)
                        .wrapping_add(trie_char_strlen(tail_str))
                        .wrapping_add(1),
                ),
            ) as *mut AlphaChar;
            alpha_p = alpha_key;
            let mut i = key_len;
            let alpha_map: &AlphaMap = unsafe { &(*(*s).trie).alpha_map };
            while i > 0 {
                let fresh0 = key_p;
                key_p = key_p.offset(1);
                let fresh1 = alpha_p;
                alpha_p = alpha_p.offset(1);
                *fresh1 = alpha_map.trie_to_char(*fresh0);
                i -= 1;
            }
        }
        let alpha_map: &AlphaMap = unsafe { &(*(*s).trie).alpha_map };
        while '\0' as i32 != *tail_str as libc::c_int {
            let fresh2 = tail_str;
            tail_str = tail_str.offset(1);
            let fresh3 = alpha_p;
            alpha_p = alpha_p.offset(1);
            *fresh3 = alpha_map.trie_to_char(*fresh2);
        }
        *alpha_p = 0 as libc::c_int as AlphaChar;
        alpha_key
    }

    pub unsafe fn get_data(iter: *const TrieIterator) -> TrieData {
        let s: *const TrieState = (*iter).state;
        let tail_index: TrieIndex;
        if s.is_null() {
            return -(1 as libc::c_int);
        }
        if (*s).is_suffix == 0 {
            if (*(*(*s).trie).da).get_base((*s).index) >= 0 as libc::c_int {
                return -(1 as libc::c_int);
            }
            tail_index = -(*(*(*s).trie).da).get_base((*s).index);
        } else {
            tail_index = (*s).index;
        }
        (*(*s).trie).tail.get_data(tail_index)
    }
}
#[cfg(test)]
mod tests;
