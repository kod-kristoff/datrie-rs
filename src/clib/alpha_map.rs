use ::libc;
use datrie::{AlphaChar, AlphaMap};

pub type FILE = libc::FILE;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;

pub const DA_OK: libc::c_int = 0;
pub const DA_ERR: libc::c_int = -1;

#[no_mangle]
pub unsafe extern "C" fn alpha_char_strlen(str: *const AlphaChar) -> libc::c_int {
    let mut p: *const AlphaChar = std::ptr::null::<AlphaChar>();
    p = str;
    while *p != 0 {
        p = p.offset(1);
    }
    p.offset_from(str) as libc::c_long as libc::c_int
}
#[no_mangle]
pub unsafe extern "C" fn alpha_char_strcmp(
    mut str1: *const AlphaChar,
    mut str2: *const AlphaChar,
) -> libc::c_int {
    while *str1 != 0 && *str1 == *str2 {
        str1 = str1.offset(1);
        str2 = str2.offset(1);
    }
    if *str1 < *str2 {
        return -(1 as libc::c_int);
    }
    if *str1 > *str2 {
        return 1 as libc::c_int;
    }
    0 as libc::c_int
}
#[no_mangle]
pub unsafe extern "C" fn alpha_map_new() -> *mut AlphaMap {
    let alpha_map = AlphaMap::default();
    Box::into_raw(Box::new(alpha_map))
}
#[no_mangle]
pub unsafe extern "C" fn alpha_map_clone(a_map: *const AlphaMap) -> *mut AlphaMap {
    if a_map.is_null() {
        return std::ptr::null_mut();
    }
    let a_map = unsafe { &*a_map };
    let alpha_map = a_map.clone();
    Box::into_raw(Box::new(alpha_map))
}
#[no_mangle]
pub unsafe extern "C" fn alpha_map_free(alpha_map: *mut AlphaMap) {
    if alpha_map.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(alpha_map)) };
}
#[no_mangle]
pub unsafe extern "C" fn alpha_map_add_range(
    alpha_map: *mut AlphaMap,
    begin: AlphaChar,
    end: AlphaChar,
) -> libc::c_int {
    if alpha_map.is_null() {
        return DA_OK;
    }
    let alpha_map = unsafe { &mut *alpha_map };
    match alpha_map.add_range(begin, end) {
        Ok(_) => DA_OK,
        Err(_) => DA_ERR,
    }
}
