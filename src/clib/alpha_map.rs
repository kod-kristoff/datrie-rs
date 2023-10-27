use ::libc;
use datrie::alpha_map::{drop_boxed, AlphaChar, AlphaMap, AlphaRange};

// extern "C" {
//     fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
//     fn free(_: *mut libc::c_void);
//     fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: libc::c_int) -> libc::c_int;
//     fn ftell(__stream: *mut FILE) -> libc::c_long;
// }
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
// pub type AlphaChar = uint32;
pub type TrieChar = libc::c_uchar;
pub type TrieIndex = int32;
#[no_mangle]
pub unsafe extern "C" fn alpha_char_strlen(mut str: *const AlphaChar) -> libc::c_int {
    let mut p: *const AlphaChar = 0 as *const AlphaChar;
    p = str;
    while *p != 0 {
        p = p.offset(1);
        p;
    }
    return p.offset_from(str) as libc::c_long as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn alpha_char_strcmp(
    mut str1: *const AlphaChar,
    mut str2: *const AlphaChar,
) -> libc::c_int {
    while *str1 != 0 && *str1 == *str2 {
        str1 = str1.offset(1);
        str1;
        str2 = str2.offset(1);
        str2;
    }
    if *str1 < *str2 {
        return -(1 as libc::c_int);
    }
    if *str1 > *str2 {
        return 1 as libc::c_int;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn alpha_map_new() -> *mut AlphaMap {
    let alpha_map = AlphaMap::new_boxed();
    return Box::into_raw(alpha_map);
}
#[no_mangle]
pub unsafe extern "C" fn alpha_map_clone(mut a_map: *const AlphaMap) -> *mut AlphaMap {
    return datrie::alpha_map::alpha_map_clone(a_map);
}
#[no_mangle]
pub unsafe extern "C" fn alpha_map_free(mut alpha_map: *mut AlphaMap) {
    drop_boxed(Box::from_raw(alpha_map));
}
#[no_mangle]
pub unsafe extern "C" fn alpha_map_add_range(
    mut alpha_map: *mut AlphaMap,
    mut begin: AlphaChar,
    mut end: AlphaChar,
) -> libc::c_int {
    return datrie::alpha_map::alpha_map_add_range(alpha_map, begin, end);
}
