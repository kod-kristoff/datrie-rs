use ::libc;
extern "C" {
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
}
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
#[derive(Copy, Clone)]
// #[repr(C)]
pub struct DString {
    pub char_size: libc::c_int,
    pub str_len: libc::c_int,
    pub alloc_size: libc::c_int,
    pub val: *mut libc::c_void,
}
#[no_mangle]
pub unsafe extern "C" fn dstring_new(char_size: libc::c_int, n_elm: libc::c_int) -> *mut DString {
    let mut ds: *mut DString = std::ptr::null_mut::<DString>();
    ds = malloc(::core::mem::size_of::<DString>() as libc::c_ulong) as *mut DString;
    if ds.is_null() as libc::c_int as libc::c_long != 0 {
        return std::ptr::null_mut::<DString>();
    }
    (*ds).alloc_size = char_size * n_elm;
    (*ds).val = malloc((*ds).alloc_size as libc::c_ulong);
    if ((*ds).val).is_null() {
        free(ds as *mut libc::c_void);
        return std::ptr::null_mut::<DString>();
    }
    (*ds).char_size = char_size;
    (*ds).str_len = 0 as libc::c_int;
    ds
}
#[no_mangle]
pub unsafe extern "C" fn dstring_free(ds: *mut DString) {
    free((*ds).val);
    free(ds as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn dstring_length(ds: *const DString) -> libc::c_int {
    (*ds).str_len
}
#[no_mangle]
pub unsafe extern "C" fn dstring_get_val(ds: *const DString) -> *const libc::c_void {
    (*ds).val
}
#[no_mangle]
pub unsafe extern "C" fn dstring_get_val_rw(ds: *mut DString) -> *mut libc::c_void {
    (*ds).val
}
#[no_mangle]
pub unsafe extern "C" fn dstring_clear(ds: *mut DString) {
    (*ds).str_len = 0 as libc::c_int;
}
unsafe extern "C" fn dstring_ensure_space(ds: *mut DString, size: libc::c_int) -> Bool {
    if (*ds).alloc_size < size {
        let re_size: libc::c_int = if (*ds).alloc_size * 2 as libc::c_int > size {
            (*ds).alloc_size * 2 as libc::c_int
        } else {
            size
        };
        let re_ptr: *mut libc::c_void = realloc((*ds).val, re_size as libc::c_ulong);
        if re_ptr.is_null() as libc::c_int as libc::c_long != 0 {
            return DA_FALSE;
        }
        (*ds).val = re_ptr;
        (*ds).alloc_size = re_size;
    }
    DA_TRUE
}
#[no_mangle]
pub unsafe extern "C" fn dstring_copy(dst: *mut DString, src: *const DString) -> Bool {
    if dstring_ensure_space(dst, ((*src).str_len + 1 as libc::c_int) * (*src).char_size) as u64 == 0
    {
        return DA_FALSE;
    }
    memcpy(
        (*dst).val,
        (*src).val,
        (((*src).str_len + 1 as libc::c_int) * (*src).char_size) as libc::c_ulong,
    );
    (*dst).char_size = (*src).char_size;
    (*dst).str_len = (*src).str_len;
    DA_TRUE
}
#[no_mangle]
pub unsafe extern "C" fn dstring_append(dst: *mut DString, src: *const DString) -> Bool {
    if (*dst).char_size != (*src).char_size {
        return DA_FALSE;
    }
    if dstring_ensure_space(
        dst,
        ((*dst).str_len + (*src).str_len + 1 as libc::c_int) * (*dst).char_size,
    ) as u64
        == 0
    {
        return DA_FALSE;
    }
    memcpy(
        ((*dst).val as *mut libc::c_char).offset(((*dst).char_size * (*dst).str_len) as isize)
            as *mut libc::c_void,
        (*src).val,
        (((*src).str_len + 1 as libc::c_int) * (*dst).char_size) as libc::c_ulong,
    );
    (*dst).str_len += (*src).str_len;
    DA_TRUE
}
#[no_mangle]
pub unsafe extern "C" fn dstring_append_string(
    ds: *mut DString,
    data: *const libc::c_void,
    len: libc::c_int,
) -> Bool {
    if dstring_ensure_space(
        ds,
        ((*ds).str_len + len + 1 as libc::c_int) * (*ds).char_size,
    ) as u64
        == 0
    {
        return DA_FALSE;
    }
    memcpy(
        ((*ds).val as *mut libc::c_char).offset(((*ds).char_size * (*ds).str_len) as isize)
            as *mut libc::c_void,
        data,
        ((*ds).char_size * len) as libc::c_ulong,
    );
    (*ds).str_len += len;
    DA_TRUE
}
#[no_mangle]
pub unsafe extern "C" fn dstring_append_char(ds: *mut DString, data: *const libc::c_void) -> Bool {
    if dstring_ensure_space(ds, ((*ds).str_len + 2 as libc::c_int) * (*ds).char_size) as u64 == 0 {
        return DA_FALSE;
    }
    memcpy(
        ((*ds).val as *mut libc::c_char).offset(((*ds).char_size * (*ds).str_len) as isize)
            as *mut libc::c_void,
        data,
        (*ds).char_size as libc::c_ulong,
    );
    (*ds).str_len += 1;
    (*ds).str_len;
    DA_TRUE
}
#[no_mangle]
pub unsafe extern "C" fn dstring_terminate(ds: *mut DString) -> Bool {
    if dstring_ensure_space(ds, ((*ds).str_len + 2 as libc::c_int) * (*ds).char_size) as u64 == 0 {
        return DA_FALSE;
    }
    memset(
        ((*ds).val as *mut libc::c_char).offset(((*ds).char_size * (*ds).str_len) as isize)
            as *mut libc::c_void,
        0 as libc::c_int,
        (*ds).char_size as libc::c_ulong,
    );
    DA_TRUE
}
#[no_mangle]
pub unsafe extern "C" fn dstring_cut_last(ds: *mut DString) -> Bool {
    if 0 as libc::c_int == (*ds).str_len {
        return DA_FALSE;
    }
    (*ds).str_len -= 1;
    (*ds).str_len;
    DA_TRUE
}
