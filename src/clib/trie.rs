use ::libc;
use datrie::{
    alpha_map::AlphaMap,
    trie::{Trie, TrieEnumFunc, TrieIterator, TrieState},
};
use std::ptr;

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
pub const DA_OK: libc::c_int = 0;
pub const DA_ERR: libc::c_int = -1;
#[no_mangle]
pub unsafe extern "C" fn trie_new(alpha_map: *const AlphaMap) -> *mut Trie {
    if alpha_map.is_null() {
        return ptr::null_mut();
    }
    let alpha_map = unsafe { &*alpha_map };
    let result = Trie::new(alpha_map);
    match result {
        Ok(trie) => Box::into_raw(Box::new(trie)),
        Err(err) => {
            eprintln!("error: {:?}", err);
            std::ptr::null_mut()
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn trie_new_from_file(path: *const libc::c_char) -> *mut Trie {
    let result = Trie::new_from_file(path);
    match result {
        Ok(trie) => Box::into_raw(Box::new(trie)),
        Err(err) => {
            eprintln!("error: {:?}", err);
            std::ptr::null_mut()
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn trie_fread(file: *mut FILE) -> *mut Trie {
    let result = Trie::fread(file);
    match result {
        Ok(trie) => Box::into_raw(Box::new(trie)),
        Err(err) => {
            eprintln!("error: {:?}", err);
            std::ptr::null_mut()
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn trie_free(trie: *mut Trie) {
    if !trie.is_null() {
        unsafe { drop(Box::from_raw(trie)) }
    }
}
#[no_mangle]
pub unsafe extern "C" fn trie_save(trie: *mut Trie, path: *const libc::c_char) -> libc::c_int {
    if trie.is_null() {
        return DA_ERR;
    }
    Trie::save(trie, path)
    // let trie = unsafe { &mut *trie };
    // match trie.save(path) {
    //     Ok(()) => DA_OK,
    //     Err(_err) => DA_ERR,
    // }
}
#[no_mangle]
pub unsafe extern "C" fn trie_get_serialized_size(trie: *const Trie) -> size_t {
    if trie.is_null() {
        return 0;
    }
    let trie = unsafe { &*trie };
    trie.get_serialized_size() as size_t
}
#[no_mangle]
pub unsafe extern "C" fn trie_serialize(trie: *mut Trie, ptr: *mut uint8) {
    if trie.is_null() || ptr.is_null() {
        return;
    }
    let trie = unsafe { &mut *trie };
    let serialized_size = trie.get_serialized_size();
    let buf: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(ptr, serialized_size) };
    let _ = trie.serialize_to_slice(buf); /* ignore errors */
}
#[no_mangle]
pub unsafe extern "C" fn trie_fwrite(trie: *mut Trie, file: *mut FILE) -> libc::c_int {
    Trie::fwrite(trie, file)
}
#[no_mangle]
pub unsafe extern "C" fn trie_is_dirty(trie: *const Trie) -> Bool {
    (*trie).is_dirty
}
#[no_mangle]
pub unsafe extern "C" fn trie_retrieve(
    trie: *const Trie,
    key: *const AlphaChar,
    o_data: *mut TrieData,
) -> Bool {
    if trie.is_null() || key.is_null() {
        return DA_FALSE;
    }
    let trie = unsafe { &*trie };
    trie.retrieve(key, o_data)
}
#[no_mangle]
pub unsafe extern "C" fn trie_store(
    trie: *mut Trie,
    key: *const AlphaChar,
    data: TrieData,
) -> Bool {
    if trie.is_null() || key.is_null() {
        return DA_FALSE;
    }
    let trie = unsafe { &mut *trie };
    trie.store(key, data)
}
#[no_mangle]
pub unsafe extern "C" fn trie_store_if_absent(
    trie: *mut Trie,
    key: *const AlphaChar,
    data: TrieData,
) -> Bool {
    if trie.is_null() || key.is_null() {
        return DA_FALSE;
    }
    let trie = unsafe { &mut *trie };
    trie.store(key, data)
}
#[no_mangle]
pub unsafe extern "C" fn trie_delete(trie: *mut Trie, key: *const AlphaChar) -> Bool {
    if trie.is_null() || key.is_null() {
        return DA_FALSE;
    }
    let trie = unsafe { &mut *trie };
    trie.delete(key)
}
#[no_mangle]
pub unsafe extern "C" fn trie_enumerate(
    trie: *const Trie,
    enum_func: TrieEnumFunc,
    user_data: *mut libc::c_void,
) -> Bool {
    if trie.is_null() {
        return DA_FALSE;
    }
    let trie = unsafe { &*trie };
    trie.enumerate(enum_func, user_data)
}
#[no_mangle]
pub unsafe extern "C" fn trie_root(trie: *const Trie) -> *mut TrieState {
    if trie.is_null() {
        return std::ptr::null_mut();
    }
    let trie = unsafe { &*trie };
    Trie::root(trie)
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_copy(dst: *mut TrieState, src: *const TrieState) {
    *dst = *src;
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_clone(s: *const TrieState) -> *mut TrieState {
    TrieState::trie_state_clone(s)
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_free(s: *mut TrieState) {
    TrieState::free(s)
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_rewind(s: *mut TrieState) {
    TrieState::rewind(s)
    // (*s).index = da_get_root((*(*s).trie).da);
    // (*s).is_suffix = DA_FALSE as libc::c_int as libc::c_short;
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_walk(s: *mut TrieState, c: AlphaChar) -> Bool {
    TrieState::walk(s, c)
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_is_walkable(s: *const TrieState, c: AlphaChar) -> Bool {
    TrieState::is_walkable(s, c)
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_walkable_chars(
    s: *const TrieState,
    chars: *mut AlphaChar,
    chars_nelm: libc::c_int,
) -> libc::c_int {
    TrieState::walkable_chars(s, chars, chars_nelm)
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_is_single(s: *const TrieState) -> Bool {
    (*s).is_suffix as Bool
}
#[no_mangle]
pub unsafe extern "C" fn trie_state_get_data(s: *const TrieState) -> TrieData {
    if s.is_null() {
        return -(1 as libc::c_int);
    }
    TrieState::get_data(s)
}
#[no_mangle]
pub unsafe extern "C" fn trie_iterator_new(s: *mut TrieState) -> *mut TrieIterator {
    TrieIterator::new(s)
}
#[no_mangle]
pub unsafe extern "C" fn trie_iterator_free(iter: *mut TrieIterator) {
    TrieIterator::free(iter)
}
#[no_mangle]
pub unsafe extern "C" fn trie_iterator_next(iter: *mut TrieIterator) -> Bool {
    TrieIterator::next(iter)
}
#[no_mangle]
pub unsafe extern "C" fn trie_iterator_get_key(iter: *const TrieIterator) -> *mut AlphaChar {
    TrieIterator::get_key(iter)
}
#[no_mangle]
pub unsafe extern "C" fn trie_iterator_get_data(iter: *const TrieIterator) -> TrieData {
    TrieIterator::get_data(iter)
}
