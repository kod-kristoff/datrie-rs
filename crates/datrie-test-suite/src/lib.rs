#![allow(dead_code)]
#[link(name = "datrie")]
extern "C" {
    fn alpha_char_strlen(str: *const AlphaChar) -> libc::c_int;
    fn alpha_char_strcmp(str1: *const AlphaChar, str2: *const AlphaChar) -> libc::c_int;
    fn alpha_map_new() -> *mut AlphaMap;
    fn alpha_map_clone(a_map: *const AlphaMap) -> *mut AlphaMap;
    fn alpha_map_free(alpha_map: *mut AlphaMap);
    fn alpha_map_add_range(
        alpha_map: *mut AlphaMap,
        begin: AlphaChar,
        end: AlphaChar,
    ) -> libc::c_int;

    fn trie_new(alpha_map: *const AlphaMap) -> *mut Trie;
    fn trie_new_from_file(path: *const libc::c_char) -> *mut Trie;
    fn trie_retrieve(trie: *const Trie, key: *const AlphaChar, o_data: *mut TrieData) -> Bool;
    fn trie_store(trie: *mut Trie, key: *const AlphaChar, data: TrieData) -> Bool;
    fn trie_fread(file: *mut libc::FILE) -> *mut Trie;
    fn trie_free(trie: *mut Trie);
    fn trie_save(trie: *mut Trie, path: *const libc::c_char) -> libc::c_int;
    fn trie_get_serialized_size(trie: *const Trie) -> usize;
    fn trie_serialize(trie: *mut Trie, ptr: *mut u8);

    fn trie_fwrite(trie: *mut Trie, file: *mut libc::FILE) -> libc::c_int;

    fn trie_is_dirty(trie: *const Trie) -> Bool;

    fn trie_store_if_absent(key: *const AlphaChar, data: TrieData) -> Bool;

    fn trie_delete(trie: *mut Trie, key: *const AlphaChar) -> Bool;

    fn trie_enumerate(
        trie: *const Trie,
        enum_func: TrieEnumFunc,
        user_data: *mut libc::c_void,
    ) -> Bool;

    fn trie_root(trie: *const Trie) -> *mut TrieState;

    fn trie_state_copy(dst: *mut TrieState, src: *const TrieState);
    fn trie_state_clone(s: *const TrieState) -> *mut TrieState;
    fn trie_state_free(s: *mut TrieState);
    fn trie_state_rewind(s: *mut TrieState);
    fn trie_state_walk(s: *mut TrieState, c: AlphaChar) -> Bool;

    fn trie_state_is_walkable(s: *const TrieState, c: AlphaChar) -> Bool;

    fn trie_state_walkable_chars(
        s: *const TrieState,
        chars: *mut AlphaChar,
        chars_nelm: libc::c_int,
    ) -> libc::c_int;

    fn trie_state_is_single(s: *const TrieState) -> Bool;
    fn trie_state_is_terminal(s: *const TrieState) -> Bool;
    fn trie_state_get_data(s: *const TrieState) -> TrieData;

    fn trie_iterator_new(s: *mut TrieState) -> *mut TrieIterator;

    fn trie_iterator_free(iter: *mut TrieIterator);

    fn trie_iterator_next(iter: *mut TrieIterator) -> Bool;

    fn trie_iterator_get_key(iter: *const TrieIterator) -> *mut AlphaChar;

    fn trie_iterator_get_data(iter: *const TrieIterator) -> TrieData;

}
#[repr(C)]
pub struct TrieState {
    _private: [u8; 0],
}
#[repr(C)]
pub struct TrieIterator {
    _private: [u8; 0],
}
#[repr(C)]
pub struct Trie {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AlphaMap {
    _private: [u8; 0],
}
pub type TrieEnumFunc =
    Option<unsafe extern "C" fn(*const AlphaChar, TrieData, *mut libc::c_void) -> Bool>;
pub type AlphaChar = u32;
pub type TrieChar = u8;
pub type TrieIndex = i32;
pub type TrieData = i32;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;

pub const DA_OK: libc::c_int = 0;
pub const DA_ERR: libc::c_int = -1;
