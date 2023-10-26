use ::libc;
extern "C" {
    fn dstring_new(char_size: libc::c_int, n_elm: libc::c_int) -> *mut DString;
    fn dstring_free(ds: *mut DString);
    fn dstring_length(ds: *const DString) -> libc::c_int;
    fn dstring_get_val(ds: *const DString) -> *const libc::c_void;
    fn dstring_get_val_rw(ds: *mut DString) -> *mut libc::c_void;
    fn dstring_clear(ds: *mut DString);
    fn dstring_copy(dst: *mut DString, src: *const DString) -> Bool;
    fn dstring_append(dst: *mut DString, src: *const DString) -> Bool;
    fn dstring_append_string(
        ds: *mut DString,
        data: *const libc::c_void,
        len: libc::c_int,
    ) -> Bool;
    fn dstring_append_char(ds: *mut DString, data: *const libc::c_void) -> Bool;
    fn dstring_cut_last(ds: *mut DString) -> Bool;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
}
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _DString {
    pub char_size: libc::c_int,
    pub str_len: libc::c_int,
    pub alloc_size: libc::c_int,
    pub val: *mut libc::c_void,
}
pub type DString = _DString;
pub type TrieChar = libc::c_uchar;
pub type size_t = libc::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _TrieString {
    pub ds: DString,
}
pub type TrieString = _TrieString;
#[no_mangle]
pub unsafe extern "C" fn trie_char_strlen(mut str: *const TrieChar) -> size_t {
    let mut len: size_t = 0 as libc::c_int as size_t;
    loop {
        let fresh0 = str;
        str = str.offset(1);
        if !(*fresh0 as libc::c_int != '\0' as i32) {
            break;
        }
        len = len.wrapping_add(1);
        len;
    }
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn trie_char_strsize(mut str: *const TrieChar) -> size_t {
    return (trie_char_strlen(str))
        .wrapping_mul(::core::mem::size_of::<TrieChar>() as libc::c_ulong);
}
#[no_mangle]
pub unsafe extern "C" fn trie_char_strdup(mut str: *const TrieChar) -> *mut TrieChar {
    let mut dup: *mut TrieChar = malloc(
        (::core::mem::size_of::<TrieChar>() as libc::c_ulong)
            .wrapping_mul(
                (trie_char_strlen(str)).wrapping_add(1 as libc::c_int as libc::c_ulong),
            ),
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
    return dup;
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_new(mut n_elm: libc::c_int) -> *mut TrieString {
    return dstring_new(
        ::core::mem::size_of::<TrieChar>() as libc::c_ulong as libc::c_int,
        n_elm,
    ) as *mut TrieString;
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_free(mut ts: *mut TrieString) {
    dstring_free(ts as *mut DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_length(mut ts: *const TrieString) -> libc::c_int {
    return dstring_length(ts as *mut DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_get_val(
    mut ts: *const TrieString,
) -> *const libc::c_void {
    return dstring_get_val(ts as *mut DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_get_val_rw(
    mut ts: *mut TrieString,
) -> *mut libc::c_void {
    return dstring_get_val_rw(ts as *mut DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_clear(mut ts: *mut TrieString) {
    dstring_clear(ts as *mut DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_copy(
    mut dst: *mut TrieString,
    mut src: *const TrieString,
) -> Bool {
    return dstring_copy(dst as *mut DString, src as *const DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_append(
    mut dst: *mut TrieString,
    mut src: *const TrieString,
) -> Bool {
    return dstring_append(dst as *mut DString, src as *const DString);
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_append_string(
    mut ts: *mut TrieString,
    mut str: *const TrieChar,
) -> Bool {
    return dstring_append_string(
        ts as *mut DString,
        str as *const libc::c_void,
        strlen(str as *const libc::c_char) as libc::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_append_char(
    mut ts: *mut TrieString,
    mut tc: TrieChar,
) -> Bool {
    return dstring_append_char(
        ts as *mut DString,
        &mut tc as *mut TrieChar as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_terminate(mut ts: *mut TrieString) -> Bool {
    static mut term: TrieChar = '\0' as i32 as TrieChar;
    return dstring_append_char(
        ts as *mut DString,
        &term as *const TrieChar as *const libc::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn trie_string_cut_last(mut ts: *mut TrieString) -> Bool {
    return dstring_cut_last(ts as *mut DString);
}
