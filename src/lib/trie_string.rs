use std::ptr::addr_of;

use ::libc;

use crate::dstring::*;

extern "C" {
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
}
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type TrieChar = libc::c_uchar;
#[derive(Copy, Clone)]
// #[repr(cC)]
pub struct TrieString {
    pub ds: DString,
}
/// # Safety
/// Caller must guarantee that `s` is a valid pointer
#[no_mangle]
pub unsafe extern "C" fn trie_char_strlen(mut s: *const TrieChar) -> usize {
    let mut len: usize = 0;
    loop {
        let fresh0 = s;
        s = unsafe { s.offset(1) };
        if unsafe { *fresh0 } as libc::c_int == '\0' as i32 {
            break;
        }
        len = len.wrapping_add(1);
    }
    len
}
#[no_mangle]
pub unsafe extern "C" fn trie_char_strsize(str: *const TrieChar) -> usize {
    (trie_char_strlen(str)).wrapping_mul(::core::mem::size_of::<TrieChar>())
}
#[no_mangle]
pub unsafe extern "C" fn trie_char_strdup(mut str: *const TrieChar) -> *mut TrieChar {
    let dup: *mut TrieChar = libc::malloc(
        (::core::mem::size_of::<TrieChar>()).wrapping_mul((trie_char_strlen(str)).wrapping_add(1)),
    ) as *mut TrieChar;
    let mut p: *mut TrieChar = dup;
    while *str as libc::c_int != '\0' as i32 {
        let fresh1 = str;
        str = str.offset(1);
        let fresh2 = p;
        p = p.offset(1);
        *fresh2 = *fresh1;
    }
    *p = '\0' as i32 as TrieChar;
    dup
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_new(n_elm: libc::c_int) -> *mut TrieString {
    dstring_new(
        ::core::mem::size_of::<TrieChar>() as libc::c_ulong as libc::c_int,
        n_elm,
    ) as *mut TrieString
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_free(ts: *mut TrieString) {
    dstring_free(ts as *mut DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_length(ts: *const TrieString) -> libc::c_int {
    dstring_length(ts as *mut DString)
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_get_val(ts: *const TrieString) -> *const libc::c_void {
    dstring_get_val(ts as *mut DString)
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_get_val_rw(ts: *mut TrieString) -> *mut libc::c_void {
    dstring_get_val_rw(ts as *mut DString)
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_clear(ts: *mut TrieString) {
    dstring_clear(ts as *mut DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_copy(dst: *mut TrieString, src: *const TrieString) -> Bool {
    dstring_copy(dst as *mut DString, src as *const DString)
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_append(dst: *mut TrieString, src: *const TrieString) -> Bool {
    dstring_append(dst as *mut DString, src as *const DString)
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_append_string(
    ts: *mut TrieString,
    str: *const TrieChar,
) -> Bool {
    dstring_append_string(
        ts as *mut DString,
        str as *const libc::c_void,
        strlen(str as *const libc::c_char) as libc::c_int,
    )
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_append_char(ts: *mut TrieString, mut tc: TrieChar) -> Bool {
    dstring_append_char(
        ts as *mut DString,
        &mut tc as *mut TrieChar as *const libc::c_void,
    )
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_terminate(ts: *mut TrieString) -> Bool {
    static mut term: TrieChar = '\0' as i32 as TrieChar;
    dstring_append_char(ts as *mut DString, addr_of!(term) as *const libc::c_void)
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_cut_last(ts: *mut TrieString) -> Bool {
    dstring_cut_last(ts as *mut DString)
}
