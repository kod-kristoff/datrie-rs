use ::libc;

use crate::datrie::{
    alpha_map::_AlphaMap,
    fileutils::{_IO_codecvt, _IO_marker, _IO_wide_data},
    trie::_Trie,
};
extern "C" {
    // pub type _IO_wide_data;
    // pub type _IO_codecvt;
    // pub type _IO_marker;
    // pub type _AlphaMap;
    // pub type _Trie;
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    fn strchr(_: *const libc::c_char, _: libc::c_int) -> *mut libc::c_char;
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn strtol(_: *const libc::c_char, _: *mut *mut libc::c_char, _: libc::c_int) -> libc::c_long;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn exit(_: libc::c_int) -> !;
    static mut stderr: *mut FILE;
    fn fclose(__stream: *mut FILE) -> libc::c_int;
    fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
    fn fprintf(_: *mut FILE, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn sscanf(_: *const libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn fgets(__s: *mut libc::c_char, __n: libc::c_int, __stream: *mut FILE) -> *mut libc::c_char;
    fn __ctype_b_loc() -> *mut *const libc::c_ushort;
    fn setlocale(__category: libc::c_int, __locale: *const libc::c_char) -> *mut libc::c_char;
    fn locale_charset() -> *const libc::c_char;
    fn iconv_close(__cd: iconv_t) -> libc::c_int;
    fn iconv_open(__tocode: *const libc::c_char, __fromcode: *const libc::c_char) -> iconv_t;
    fn iconv(
        __cd: iconv_t,
        __inbuf: *mut *mut libc::c_char,
        __inbytesleft: *mut size_t,
        __outbuf: *mut *mut libc::c_char,
        __outbytesleft: *mut size_t,
    ) -> size_t;
    // fn __assert_fail(
    //     __assertion: *const libc::c_char,
    //     __file: *const libc::c_char,
    //     __line: libc::c_uint,
    //     __function: *const libc::c_char,
    // ) -> !;
    fn alpha_map_free(alpha_map: *mut AlphaMap);
    fn alpha_map_add_range(
        alpha_map: *mut AlphaMap,
        begin: AlphaChar,
        end: AlphaChar,
    ) -> libc::c_int;
    fn alpha_char_strlen(str: *const AlphaChar) -> libc::c_int;
    fn trie_new(alpha_map: *const AlphaMap) -> *mut Trie;
    fn trie_new_from_file(path: *const libc::c_char) -> *mut Trie;
    fn trie_free(trie: *mut Trie);
    fn trie_save(trie: *mut Trie, path: *const libc::c_char) -> libc::c_int;
    fn trie_is_dirty(trie: *const Trie) -> Bool;
    fn trie_retrieve(trie: *const Trie, key: *const AlphaChar, o_data: *mut TrieData) -> Bool;
    fn trie_store(trie: *mut Trie, key: *const AlphaChar, data: TrieData) -> Bool;
    fn trie_delete(trie: *mut Trie, key: *const AlphaChar) -> Bool;
    fn trie_enumerate(
        trie: *const Trie,
        enum_func: TrieEnumFunc,
        user_data: *mut libc::c_void,
    ) -> Bool;
    fn alpha_map_new() -> *mut AlphaMap;
}
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: libc::c_int,
    pub _IO_read_ptr: *mut libc::c_char,
    pub _IO_read_end: *mut libc::c_char,
    pub _IO_read_base: *mut libc::c_char,
    pub _IO_write_base: *mut libc::c_char,
    pub _IO_write_ptr: *mut libc::c_char,
    pub _IO_write_end: *mut libc::c_char,
    pub _IO_buf_base: *mut libc::c_char,
    pub _IO_buf_end: *mut libc::c_char,
    pub _IO_save_base: *mut libc::c_char,
    pub _IO_backup_base: *mut libc::c_char,
    pub _IO_save_end: *mut libc::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: libc::c_int,
    pub _flags2: libc::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: libc::c_ushort,
    pub _vtable_offset: libc::c_schar,
    pub _shortbuf: [libc::c_char; 1],
    pub _lock: *mut libc::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut libc::c_void,
    pub __pad5: size_t,
    pub _mode: libc::c_int,
    pub _unused2: [libc::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type C2RustUnnamed = libc::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;
pub type iconv_t = *mut libc::c_void;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type uint32 = libc::c_uint;
pub type int32 = libc::c_int;
pub type AlphaChar = uint32;
pub type TrieData = int32;
pub type AlphaMap = _AlphaMap;
pub type Trie = _Trie;
pub type TrieEnumFunc =
    Option<unsafe extern "C" fn(*const AlphaChar, TrieData, *mut libc::c_void) -> Bool>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ProgEnv {
    pub path: *const libc::c_char,
    pub trie_name: *const libc::c_char,
    pub to_alpha_conv: iconv_t,
    pub from_alpha_conv: iconv_t,
    pub trie: *mut Trie,
}
#[inline]
unsafe extern "C" fn atoi(mut __nptr: *const libc::c_char) -> libc::c_int {
    return strtol(
        __nptr,
        0 as *mut libc::c_void as *mut *mut libc::c_char,
        10 as libc::c_int,
    ) as libc::c_int;
}
unsafe fn main_0(mut argc: libc::c_int, mut argv: *mut *mut libc::c_char) -> libc::c_int {
    let mut i: libc::c_int = 0;
    let mut env: ProgEnv = ProgEnv {
        path: 0 as *const libc::c_char,
        trie_name: 0 as *const libc::c_char,
        to_alpha_conv: 0 as *mut libc::c_void,
        from_alpha_conv: 0 as *mut libc::c_void,
        trie: 0 as *mut Trie,
    };
    let mut ret: libc::c_int = 0;
    env.path = b".\0" as *const u8 as *const libc::c_char;
    init_conv(&mut env);
    i = decode_switch(argc, argv, &mut env);
    if i == argc {
        usage(*argv.offset(0 as libc::c_int as isize), 1 as libc::c_int);
    }
    let fresh0 = i;
    i = i + 1;
    env.trie_name = *argv.offset(fresh0 as isize);
    if prepare_trie(&mut env) != 0 as libc::c_int {
        exit(1 as libc::c_int);
    }
    ret = decode_command(argc - i, argv.offset(i as isize), &mut env);
    if close_trie(&mut env) != 0 as libc::c_int {
        exit(1 as libc::c_int);
    }
    close_conv(&mut env);
    return ret;
}
unsafe extern "C" fn init_conv(mut env: *mut ProgEnv) {
    let mut prev_locale: *const libc::c_char = 0 as *const libc::c_char;
    let mut locale_codeset: *const libc::c_char = 0 as *const libc::c_char;
    prev_locale = setlocale(0 as libc::c_int, b"\0" as *const u8 as *const libc::c_char);
    locale_codeset = locale_charset();
    setlocale(0 as libc::c_int, prev_locale);
    (*env).to_alpha_conv = iconv_open(
        b"UCS-4LE\0" as *const u8 as *const libc::c_char,
        locale_codeset,
    );
    (*env).from_alpha_conv = iconv_open(
        locale_codeset,
        b"UCS-4LE\0" as *const u8 as *const libc::c_char,
    );
}
unsafe extern "C" fn conv_to_alpha(
    mut env: *mut ProgEnv,
    mut in_0: *const libc::c_char,
    mut out: *mut AlphaChar,
    mut out_size: size_t,
) -> size_t {
    let mut in_p: *mut libc::c_char = in_0 as *mut libc::c_char;
    let mut out_p: *mut libc::c_char = out as *mut libc::c_char;
    let mut in_left: size_t = strlen(in_0);
    let mut out_left: size_t =
        out_size.wrapping_mul(::core::mem::size_of::<AlphaChar>() as libc::c_ulong);
    let mut res: size_t = 0;
    let mut byte_p: *const libc::c_uchar = 0 as *const libc::c_uchar;
    if ::core::mem::size_of::<AlphaChar>() as libc::c_ulong == 4 as libc::c_int as libc::c_ulong {
    } else {
        panic!("sizeof (AlphaChar) == 4");
        // __assert_fail(
        //     b"sizeof (AlphaChar) == 4\0" as *const u8 as *const libc::c_char,
        //     b"trietool.c\0" as *const u8 as *const libc::c_char,
        //     121 as libc::c_int as libc::c_uint,
        //     (*::core::mem::transmute::<&[u8; 67], &[libc::c_char; 67]>(
        //         b"size_t conv_to_alpha(ProgEnv *, const char *, AlphaChar *, size_t)\0",
        //     ))
        //     .as_ptr(),
        // );
    }
    'c_2942: {
        if ::core::mem::size_of::<AlphaChar>() as libc::c_ulong == 4 as libc::c_int as libc::c_ulong
        {
        } else {
            panic!("sizeof (AlphaChar) == 4");
            //             __assert_fail(
            //     b"sizeof (AlphaChar) == 4\0" as *const u8 as *const libc::c_char,
            //     b"trietool.c\0" as *const u8 as *const libc::c_char,
            //     121 as libc::c_int as libc::c_uint,
            //     (*::core::mem::transmute::<&[u8; 67], &[libc::c_char; 67]>(
            //         b"size_t conv_to_alpha(ProgEnv *, const char *, AlphaChar *, size_t)\0",
            //     ))
            //     .as_ptr(),
            // );
        }
    };
    res = iconv(
        (*env).to_alpha_conv,
        &mut in_p as *mut *mut libc::c_char,
        &mut in_left,
        &mut out_p,
        &mut out_left,
    );
    if res == -(1 as libc::c_int) as size_t {
        return res;
    }
    res = 0 as libc::c_int as size_t;
    byte_p = out as *const libc::c_uchar;
    while res < out_size
        && byte_p.offset(3 as libc::c_int as isize)
            < out_p as *mut libc::c_uchar as *const libc::c_uchar
    {
        let fresh1 = res;
        res = res.wrapping_add(1);
        *out.offset(fresh1 as isize) = (*byte_p.offset(0 as libc::c_int as isize) as libc::c_int
            | (*byte_p.offset(1 as libc::c_int as isize) as libc::c_int) << 8 as libc::c_int
            | (*byte_p.offset(2 as libc::c_int as isize) as libc::c_int) << 16 as libc::c_int
            | (*byte_p.offset(3 as libc::c_int as isize) as libc::c_int) << 24 as libc::c_int)
            as AlphaChar;
        byte_p = byte_p.offset(4 as libc::c_int as isize);
    }
    if res < out_size {
        *out.offset(res as isize) = 0 as libc::c_int as AlphaChar;
    }
    return res;
}
unsafe extern "C" fn conv_from_alpha(
    mut env: *mut ProgEnv,
    mut in_0: *const AlphaChar,
    mut out: *mut libc::c_char,
    mut out_size: size_t,
) -> size_t {
    let mut in_left: size_t = (alpha_char_strlen(in_0) as libc::c_ulong)
        .wrapping_mul(::core::mem::size_of::<AlphaChar>() as libc::c_ulong);
    let mut res: size_t = 0;
    if ::core::mem::size_of::<AlphaChar>() as libc::c_ulong == 4 as libc::c_int as libc::c_ulong {
    } else {
        panic!("sizeof (AlphaChar) == 4");
        // __assert_fail(
        //     b"sizeof (AlphaChar) == 4\0" as *const u8 as *const libc::c_char,
        //     b"trietool.c\0" as *const u8 as *const libc::c_char,
        //     154 as libc::c_int as libc::c_uint,
        //     (*::core::mem::transmute::<&[u8; 69], &[libc::c_char; 69]>(
        //         b"size_t conv_from_alpha(ProgEnv *, const AlphaChar *, char *, size_t)\0",
        //     ))
        //     .as_ptr(),
        // );
    }
    'c_3143: {
        if ::core::mem::size_of::<AlphaChar>() as libc::c_ulong == 4 as libc::c_int as libc::c_ulong
        {
        } else {
            panic!("sizeof (AlphaChar) == 4");
            // __assert_fail(
            //     b"sizeof (AlphaChar) == 4\0" as *const u8 as *const libc::c_char,
            //     b"trietool.c\0" as *const u8 as *const libc::c_char,
            //     154 as libc::c_int as libc::c_uint,
            //     (*::core::mem::transmute::<&[u8; 69], &[libc::c_char; 69]>(
            //         b"size_t conv_from_alpha(ProgEnv *, const AlphaChar *, char *, size_t)\0",
            //     ))
            //     .as_ptr(),
            // );
        }
    };
    res = 0 as libc::c_int as size_t;
    while *in_0.offset(res as isize) != 0 {
        let mut b: [libc::c_uchar; 4] = [0; 4];
        b[0 as libc::c_int as usize] =
            (*in_0.offset(res as isize) & 0xff as libc::c_int as libc::c_uint) as libc::c_uchar;
        b[1 as libc::c_int as usize] = (*in_0.offset(res as isize) >> 8 as libc::c_int
            & 0xff as libc::c_int as libc::c_uint)
            as libc::c_uchar;
        b[2 as libc::c_int as usize] = (*in_0.offset(res as isize) >> 16 as libc::c_int
            & 0xff as libc::c_int as libc::c_uint)
            as libc::c_uchar;
        b[3 as libc::c_int as usize] = (*in_0.offset(res as isize) >> 24 as libc::c_int
            & 0xff as libc::c_int as libc::c_uint)
            as libc::c_uchar;
        memcpy(
            &*in_0.offset(res as isize) as *const AlphaChar as *mut libc::c_char
                as *mut libc::c_void,
            b.as_mut_ptr() as *const libc::c_void,
            4 as libc::c_int as libc::c_ulong,
        );
        res = res.wrapping_add(1);
        res;
    }
    res = iconv(
        (*env).from_alpha_conv,
        &mut in_0 as *mut *const AlphaChar as *mut *mut libc::c_char,
        &mut in_left,
        &mut out,
        &mut out_size,
    );
    *out = 0 as libc::c_int as libc::c_char;
    return res;
}
unsafe extern "C" fn close_conv(mut env: *mut ProgEnv) {
    iconv_close((*env).to_alpha_conv);
    iconv_close((*env).from_alpha_conv);
}
unsafe extern "C" fn full_path(
    mut path: *const libc::c_char,
    mut name: *const libc::c_char,
    mut ext: *const libc::c_char,
) -> *mut libc::c_char {
    let mut full_size: libc::c_int = (strlen(path))
        .wrapping_add(strlen(name))
        .wrapping_add(strlen(ext))
        .wrapping_add(2 as libc::c_int as libc::c_ulong)
        as libc::c_int;
    let mut full_path_buff: *mut libc::c_char =
        malloc(full_size as libc::c_ulong) as *mut libc::c_char;
    sprintf(
        full_path_buff,
        b"%s/%s%s\0" as *const u8 as *const libc::c_char,
        path,
        name,
        ext,
    );
    return full_path_buff;
}
unsafe extern "C" fn prepare_trie(mut env: *mut ProgEnv) -> libc::c_int {
    let mut buff: [libc::c_char; 256] = [0; 256];
    let mut path_name: *mut libc::c_char = 0 as *mut libc::c_char;
    path_name = full_path(
        (*env).path,
        (*env).trie_name,
        b".tri\0" as *const u8 as *const libc::c_char,
    );
    (*env).trie = trie_new_from_file(path_name);
    free(path_name as *mut libc::c_void);
    if ((*env).trie).is_null() {
        let mut sbm: *mut FILE = 0 as *mut FILE;
        let mut alpha_map: *mut AlphaMap = 0 as *mut AlphaMap;
        path_name = full_path(
            (*env).path,
            (*env).trie_name,
            b".abm\0" as *const u8 as *const libc::c_char,
        );
        sbm = fopen(path_name, b"r\0" as *const u8 as *const libc::c_char);
        if sbm.is_null() {
            fprintf(
                stderr,
                b"Cannot open alphabet map file %s\n\0" as *const u8 as *const libc::c_char,
                path_name,
            );
            free(path_name as *mut libc::c_void);
            return -(1 as libc::c_int);
        }
        free(path_name as *mut libc::c_void);
        alpha_map = alpha_map_new();
        while !(fgets(
            buff.as_mut_ptr(),
            ::core::mem::size_of::<[libc::c_char; 256]>() as libc::c_ulong as libc::c_int,
            sbm,
        ))
        .is_null()
        {
            let mut b: libc::c_uint = 0;
            let mut e: libc::c_uint = 0;
            if sscanf(
                buff.as_mut_ptr(),
                b" [ %x , %x ] \0" as *const u8 as *const libc::c_char,
                &mut b as *mut libc::c_uint,
                &mut e as *mut libc::c_uint,
            ) != 2 as libc::c_int
            {
                continue;
            }
            if b > e {
                fprintf(
                    stderr,
                    b"Range begin (%x) > range end (%x)\n\0" as *const u8 as *const libc::c_char,
                    b,
                    e,
                );
            } else {
                alpha_map_add_range(alpha_map, b, e);
            }
        }
        (*env).trie = trie_new(alpha_map);
        alpha_map_free(alpha_map);
        fclose(sbm);
    }
    return 0 as libc::c_int;
}
unsafe extern "C" fn close_trie(mut env: *mut ProgEnv) -> libc::c_int {
    if trie_is_dirty((*env).trie) as u64 != 0 {
        let mut path: *mut libc::c_char = full_path(
            (*env).path,
            (*env).trie_name,
            b".tri\0" as *const u8 as *const libc::c_char,
        );
        if trie_save((*env).trie, path) != 0 as libc::c_int {
            fprintf(
                stderr,
                b"Cannot save trie to %s\n\0" as *const u8 as *const libc::c_char,
                path,
            );
            free(path as *mut libc::c_void);
            return -(1 as libc::c_int);
        }
        free(path as *mut libc::c_void);
    }
    trie_free((*env).trie);
    return 0 as libc::c_int;
}
unsafe extern "C" fn decode_switch(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
    mut env: *mut ProgEnv,
) -> libc::c_int {
    let mut opt_idx: libc::c_int = 0;
    opt_idx = 1 as libc::c_int;
    while opt_idx < argc && **argv.offset(opt_idx as isize) as libc::c_int == '-' as i32 {
        if strcmp(
            *argv.offset(opt_idx as isize),
            b"-h\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
            || strcmp(
                *argv.offset(opt_idx as isize),
                b"--help\0" as *const u8 as *const libc::c_char,
            ) == 0 as libc::c_int
        {
            usage(*argv.offset(0 as libc::c_int as isize), 1 as libc::c_int);
        } else if strcmp(
            *argv.offset(opt_idx as isize),
            b"-V\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
            || strcmp(
                *argv.offset(opt_idx as isize),
                b"--version\0" as *const u8 as *const libc::c_char,
            ) == 0 as libc::c_int
        {
            printf(
                b"%s\n\0" as *const u8 as *const libc::c_char,
                b"0.2.13-6-gb174e65\0" as *const u8 as *const libc::c_char,
            );
            exit(1 as libc::c_int);
        } else if strcmp(
            *argv.offset(opt_idx as isize),
            b"-p\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
            || strcmp(
                *argv.offset(opt_idx as isize),
                b"--path\0" as *const u8 as *const libc::c_char,
            ) == 0 as libc::c_int
        {
            opt_idx += 1;
            (*env).path = *argv.offset(opt_idx as isize);
        } else if strcmp(
            *argv.offset(opt_idx as isize),
            b"--\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opt_idx += 1;
            opt_idx;
            break;
        } else {
            fprintf(
                stderr,
                b"Unknown option: %s\n\0" as *const u8 as *const libc::c_char,
                *argv.offset(opt_idx as isize),
            );
            exit(1 as libc::c_int);
        }
        opt_idx += 1;
        opt_idx;
    }
    return opt_idx;
}
unsafe extern "C" fn decode_command(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
    mut env: *mut ProgEnv,
) -> libc::c_int {
    let mut opt_idx: libc::c_int = 0;
    opt_idx = 0 as libc::c_int;
    while opt_idx < argc {
        if strcmp(
            *argv.offset(opt_idx as isize),
            b"add\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opt_idx += 1;
            opt_idx;
            opt_idx += command_add(argc - opt_idx, argv.offset(opt_idx as isize), env);
        } else if strcmp(
            *argv.offset(opt_idx as isize),
            b"add-list\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opt_idx += 1;
            opt_idx;
            opt_idx += command_add_list(argc - opt_idx, argv.offset(opt_idx as isize), env);
        } else if strcmp(
            *argv.offset(opt_idx as isize),
            b"delete\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opt_idx += 1;
            opt_idx;
            opt_idx += command_delete(argc - opt_idx, argv.offset(opt_idx as isize), env);
        } else if strcmp(
            *argv.offset(opt_idx as isize),
            b"delete-list\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opt_idx += 1;
            opt_idx;
            opt_idx += command_delete_list(argc - opt_idx, argv.offset(opt_idx as isize), env);
        } else if strcmp(
            *argv.offset(opt_idx as isize),
            b"query\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opt_idx += 1;
            opt_idx;
            opt_idx += command_query(argc - opt_idx, argv.offset(opt_idx as isize), env);
        } else if strcmp(
            *argv.offset(opt_idx as isize),
            b"list\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opt_idx += 1;
            opt_idx;
            opt_idx += command_list(argc - opt_idx, argv.offset(opt_idx as isize), env);
        } else {
            fprintf(
                stderr,
                b"Unknown command: %s\n\0" as *const u8 as *const libc::c_char,
                *argv.offset(opt_idx as isize),
            );
            return 1 as libc::c_int;
        }
        opt_idx += 1;
        opt_idx;
    }
    return 0 as libc::c_int;
}
unsafe extern "C" fn command_add(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
    mut env: *mut ProgEnv,
) -> libc::c_int {
    let mut opt_idx: libc::c_int = 0;
    opt_idx = 0 as libc::c_int;
    while opt_idx < argc {
        let mut key: *const libc::c_char = 0 as *const libc::c_char;
        let mut key_alpha: [AlphaChar; 256] = [0; 256];
        let mut data: TrieData = 0;
        let fresh2 = opt_idx;
        opt_idx = opt_idx + 1;
        key = *argv.offset(fresh2 as isize);
        data = if opt_idx < argc {
            let fresh3 = opt_idx;
            opt_idx = opt_idx + 1;
            atoi(*argv.offset(fresh3 as isize))
        } else {
            -(1 as libc::c_int)
        };
        conv_to_alpha(
            env,
            key,
            key_alpha.as_mut_ptr(),
            (::core::mem::size_of::<[AlphaChar; 256]>() as libc::c_ulong)
                .wrapping_div(::core::mem::size_of::<AlphaChar>() as libc::c_ulong),
        );
        if trie_store((*env).trie, key_alpha.as_mut_ptr(), data) as u64 == 0 {
            fprintf(
                stderr,
                b"Failed to add entry '%s' with data %d\n\0" as *const u8 as *const libc::c_char,
                key,
                data,
            );
        }
    }
    return opt_idx;
}
unsafe extern "C" fn command_add_list(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
    mut env: *mut ProgEnv,
) -> libc::c_int {
    let mut enc_name: *const libc::c_char = 0 as *const libc::c_char;
    let mut input_name: *const libc::c_char = 0 as *const libc::c_char;
    let mut opt_idx: libc::c_int = 0;
    let mut saved_conv: iconv_t = 0 as *mut libc::c_void;
    let mut input: *mut FILE = 0 as *mut FILE;
    let mut line: [libc::c_char; 256] = [0; 256];
    enc_name = 0 as *const libc::c_char;
    opt_idx = 0 as libc::c_int;
    saved_conv = (*env).to_alpha_conv;
    if strcmp(
        *argv.offset(0 as libc::c_int as isize),
        b"-e\0" as *const u8 as *const libc::c_char,
    ) == 0 as libc::c_int
        || strcmp(
            *argv.offset(0 as libc::c_int as isize),
            b"--encoding\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
    {
        opt_idx += 1;
        if opt_idx >= argc {
            fprintf(
                stderr,
                b"add-list option \"%s\" requires encoding name\0" as *const u8
                    as *const libc::c_char,
                *argv.offset(0 as libc::c_int as isize),
            );
            return opt_idx;
        }
        let fresh4 = opt_idx;
        opt_idx = opt_idx + 1;
        enc_name = *argv.offset(fresh4 as isize);
    }
    if opt_idx >= argc {
        fprintf(
            stderr,
            b"add-list requires input word list file name\n\0" as *const u8 as *const libc::c_char,
        );
        return opt_idx;
    }
    let fresh5 = opt_idx;
    opt_idx = opt_idx + 1;
    input_name = *argv.offset(fresh5 as isize);
    if !enc_name.is_null() {
        let mut conv: iconv_t =
            iconv_open(b"UCS-4LE\0" as *const u8 as *const libc::c_char, enc_name);
        if -(1 as libc::c_int) as iconv_t == conv {
            fprintf(
                stderr,
                b"Conversion from \"%s\" to \"%s\" is not supported.\n\0" as *const u8
                    as *const libc::c_char,
                enc_name,
                b"UCS-4LE\0" as *const u8 as *const libc::c_char,
            );
            return opt_idx;
        }
        (*env).to_alpha_conv = conv;
    }
    input = fopen(input_name, b"r\0" as *const u8 as *const libc::c_char);
    if input.is_null() {
        fprintf(
            stderr,
            b"add-list: Cannot open input file \"%s\"\n\0" as *const u8 as *const libc::c_char,
            input_name,
        );
    } else {
        while !(fgets(
            line.as_mut_ptr(),
            ::core::mem::size_of::<[libc::c_char; 256]>() as libc::c_ulong as libc::c_int,
            input,
        ))
        .is_null()
        {
            let mut key: *mut libc::c_char = 0 as *mut libc::c_char;
            let mut data: *mut libc::c_char = 0 as *mut libc::c_char;
            let mut key_alpha: [AlphaChar; 256] = [0; 256];
            let mut data_val: TrieData = 0;
            key = string_trim(line.as_mut_ptr());
            if '\0' as i32 != *key as libc::c_int {
                data = key;
                while *data as libc::c_int != 0
                    && (strchr(
                        b"\t,\0" as *const u8 as *const libc::c_char,
                        *data as libc::c_int,
                    ))
                    .is_null()
                {
                    data = data.offset(1);
                    data;
                }
                if '\0' as i32 != *data as libc::c_int {
                    let fresh6 = data;
                    data = data.offset(1);
                    *fresh6 = '\0' as i32 as libc::c_char;
                    while *(*__ctype_b_loc()).offset(*data as libc::c_uchar as libc::c_int as isize)
                        as libc::c_int
                        & _ISspace as libc::c_int as libc::c_ushort as libc::c_int
                        != 0
                    {
                        data = data.offset(1);
                        data;
                    }
                }
                data_val = if '\0' as i32 != *data as libc::c_int {
                    atoi(data)
                } else {
                    -(1 as libc::c_int)
                };
                conv_to_alpha(
                    env,
                    key,
                    key_alpha.as_mut_ptr(),
                    (::core::mem::size_of::<[AlphaChar; 256]>() as libc::c_ulong)
                        .wrapping_div(::core::mem::size_of::<AlphaChar>() as libc::c_ulong),
                );
                if trie_store((*env).trie, key_alpha.as_mut_ptr(), data_val) as u64 == 0 {
                    fprintf(
                        stderr,
                        b"Failed to add key '%s' with data %d.\n\0" as *const u8
                            as *const libc::c_char,
                        key,
                        data_val,
                    );
                }
            }
        }
        fclose(input);
    }
    if !enc_name.is_null() {
        iconv_close((*env).to_alpha_conv);
        (*env).to_alpha_conv = saved_conv;
    }
    return opt_idx;
}
unsafe extern "C" fn command_delete(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
    mut env: *mut ProgEnv,
) -> libc::c_int {
    let mut opt_idx: libc::c_int = 0;
    opt_idx = 0 as libc::c_int;
    while opt_idx < argc {
        let mut key_alpha: [AlphaChar; 256] = [0; 256];
        conv_to_alpha(
            env,
            *argv.offset(opt_idx as isize),
            key_alpha.as_mut_ptr(),
            (::core::mem::size_of::<[AlphaChar; 256]>() as libc::c_ulong)
                .wrapping_div(::core::mem::size_of::<AlphaChar>() as libc::c_ulong),
        );
        if trie_delete((*env).trie, key_alpha.as_mut_ptr()) as u64 == 0 {
            fprintf(
                stderr,
                b"No entry '%s'. Not deleted.\n\0" as *const u8 as *const libc::c_char,
                *argv.offset(opt_idx as isize),
            );
        }
        opt_idx += 1;
        opt_idx;
    }
    return opt_idx;
}
unsafe extern "C" fn command_delete_list(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
    mut env: *mut ProgEnv,
) -> libc::c_int {
    let mut enc_name: *const libc::c_char = 0 as *const libc::c_char;
    let mut input_name: *const libc::c_char = 0 as *const libc::c_char;
    let mut opt_idx: libc::c_int = 0;
    let mut saved_conv: iconv_t = 0 as *mut libc::c_void;
    let mut input: *mut FILE = 0 as *mut FILE;
    let mut line: [libc::c_char; 256] = [0; 256];
    enc_name = 0 as *const libc::c_char;
    opt_idx = 0 as libc::c_int;
    saved_conv = (*env).to_alpha_conv;
    if strcmp(
        *argv.offset(0 as libc::c_int as isize),
        b"-e\0" as *const u8 as *const libc::c_char,
    ) == 0 as libc::c_int
        || strcmp(
            *argv.offset(0 as libc::c_int as isize),
            b"--encoding\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
    {
        opt_idx += 1;
        if opt_idx >= argc {
            fprintf(
                stderr,
                b"delete-list option \"%s\" requires encoding name\0" as *const u8
                    as *const libc::c_char,
                *argv.offset(0 as libc::c_int as isize),
            );
            return opt_idx;
        }
        let fresh7 = opt_idx;
        opt_idx = opt_idx + 1;
        enc_name = *argv.offset(fresh7 as isize);
    }
    if opt_idx >= argc {
        fprintf(
            stderr,
            b"delete-list requires input word list file name\n\0" as *const u8
                as *const libc::c_char,
        );
        return opt_idx;
    }
    let fresh8 = opt_idx;
    opt_idx = opt_idx + 1;
    input_name = *argv.offset(fresh8 as isize);
    if !enc_name.is_null() {
        let mut conv: iconv_t =
            iconv_open(b"UCS-4LE\0" as *const u8 as *const libc::c_char, enc_name);
        if -(1 as libc::c_int) as iconv_t == conv {
            fprintf(
                stderr,
                b"Conversion from \"%s\" to \"%s\" is not supported.\n\0" as *const u8
                    as *const libc::c_char,
                enc_name,
                b"UCS-4LE\0" as *const u8 as *const libc::c_char,
            );
            return opt_idx;
        }
        (*env).to_alpha_conv = conv;
    }
    input = fopen(input_name, b"r\0" as *const u8 as *const libc::c_char);
    if input.is_null() {
        fprintf(
            stderr,
            b"delete-list: Cannot open input file \"%s\"\n\0" as *const u8 as *const libc::c_char,
            input_name,
        );
    } else {
        while !(fgets(
            line.as_mut_ptr(),
            ::core::mem::size_of::<[libc::c_char; 256]>() as libc::c_ulong as libc::c_int,
            input,
        ))
        .is_null()
        {
            let mut p: *mut libc::c_char = 0 as *mut libc::c_char;
            p = string_trim(line.as_mut_ptr());
            if '\0' as i32 != *p as libc::c_int {
                let mut key_alpha: [AlphaChar; 256] = [0; 256];
                conv_to_alpha(
                    env,
                    p,
                    key_alpha.as_mut_ptr(),
                    (::core::mem::size_of::<[AlphaChar; 256]>() as libc::c_ulong)
                        .wrapping_div(::core::mem::size_of::<AlphaChar>() as libc::c_ulong),
                );
                if trie_delete((*env).trie, key_alpha.as_mut_ptr()) as u64 == 0 {
                    fprintf(
                        stderr,
                        b"No entry '%s'. Not deleted.\n\0" as *const u8 as *const libc::c_char,
                        p,
                    );
                }
            }
        }
        fclose(input);
    }
    if !enc_name.is_null() {
        iconv_close((*env).to_alpha_conv);
        (*env).to_alpha_conv = saved_conv;
    }
    return opt_idx;
}
unsafe extern "C" fn command_query(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
    mut env: *mut ProgEnv,
) -> libc::c_int {
    let mut key_alpha: [AlphaChar; 256] = [0; 256];
    let mut data: TrieData = 0;
    if argc == 0 as libc::c_int {
        fprintf(
            stderr,
            b"query: No key specified.\n\0" as *const u8 as *const libc::c_char,
        );
        return 0 as libc::c_int;
    }
    conv_to_alpha(
        env,
        *argv.offset(0 as libc::c_int as isize),
        key_alpha.as_mut_ptr(),
        (::core::mem::size_of::<[AlphaChar; 256]>() as libc::c_ulong)
            .wrapping_div(::core::mem::size_of::<AlphaChar>() as libc::c_ulong),
    );
    if trie_retrieve((*env).trie, key_alpha.as_mut_ptr(), &mut data) as u64 != 0 {
        printf(b"%d\n\0" as *const u8 as *const libc::c_char, data);
    } else {
        fprintf(
            stderr,
            b"query: Key '%s' not found.\n\0" as *const u8 as *const libc::c_char,
            *argv.offset(0 as libc::c_int as isize),
        );
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn list_enum_func(
    mut key: *const AlphaChar,
    mut key_data: TrieData,
    mut user_data: *mut libc::c_void,
) -> Bool {
    let mut env: *mut ProgEnv = user_data as *mut ProgEnv;
    let mut key_locale: [libc::c_char; 1024] = [0; 1024];
    conv_from_alpha(
        env,
        key,
        key_locale.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 1024]>() as libc::c_ulong)
            .wrapping_div(::core::mem::size_of::<libc::c_char>() as libc::c_ulong),
    );
    printf(
        b"%s\t%d\n\0" as *const u8 as *const libc::c_char,
        key_locale.as_mut_ptr(),
        key_data,
    );
    return DA_TRUE;
}
unsafe extern "C" fn command_list(
    mut argc: libc::c_int,
    mut argv: *mut *mut libc::c_char,
    mut env: *mut ProgEnv,
) -> libc::c_int {
    trie_enumerate(
        (*env).trie,
        Some(
            list_enum_func
                as unsafe extern "C" fn(*const AlphaChar, TrieData, *mut libc::c_void) -> Bool,
        ),
        env as *mut libc::c_void,
    );
    return 0 as libc::c_int;
}
unsafe extern "C" fn usage(mut prog_name: *const libc::c_char, mut exit_status: libc::c_int) {
    printf(
        b"%s - double-array trie manipulator\n\0" as *const u8 as *const libc::c_char,
        prog_name,
    );
    printf(
        b"Usage: %s [OPTION]... TRIE CMD ARG ...\n\0" as *const u8 as *const libc::c_char,
        prog_name,
    );
    printf(b"Options:\n\0" as *const u8 as *const libc::c_char);
    printf(
        b"  -p, --path DIR           set trie directory to DIR [default=.]\n\0" as *const u8
            as *const libc::c_char,
    );
    printf(
        b"  -h, --help               display this help and exit\n\0" as *const u8
            as *const libc::c_char,
    );
    printf(
        b"  -V, --version            output version information and exit\n\0" as *const u8
            as *const libc::c_char,
    );
    printf(b"\n\0" as *const u8 as *const libc::c_char);
    printf(b"Commands:\n\0" as *const u8 as *const libc::c_char);
    printf(
        b"  add  WORD DATA ...\n      Add WORD with DATA to trie\n\0" as *const u8
            as *const libc::c_char,
    );
    printf(
        b"  add-list [OPTION] LISTFILE\n      Add words and data listed in LISTFILE to trie\n      Options:\n          -e, --encoding ENC    specify character encoding of LISTFILE\n\0"
            as *const u8 as *const libc::c_char,
    );
    printf(
        b"  delete WORD ...\n      Delete WORD from trie\n\0" as *const u8 as *const libc::c_char,
    );
    printf(
        b"  delete-list [OPTION] LISTFILE\n      Delete words listed in LISTFILE from trie\n      Options:\n          -e, --encoding ENC    specify character encoding of LISTFILE\n\0"
            as *const u8 as *const libc::c_char,
    );
    printf(
        b"  query WORD\n      Query WORD data from trie\n\0" as *const u8 as *const libc::c_char,
    );
    printf(b"  list\n      List all words in trie\n\0" as *const u8 as *const libc::c_char);
    exit(exit_status);
}
unsafe extern "C" fn string_trim(mut s: *mut libc::c_char) -> *mut libc::c_char {
    let mut p: *mut libc::c_char = 0 as *mut libc::c_char;
    while *s as libc::c_int != 0
        && *(*__ctype_b_loc()).offset(*s as libc::c_uchar as libc::c_int as isize) as libc::c_int
            & _ISspace as libc::c_int as libc::c_ushort as libc::c_int
            != 0
    {
        s = s.offset(1);
        s;
    }
    p = s
        .offset(strlen(s) as isize)
        .offset(-(1 as libc::c_int as isize));
    while *(*__ctype_b_loc()).offset(*p as libc::c_uchar as libc::c_int as isize) as libc::c_int
        & _ISspace as libc::c_int as libc::c_ushort as libc::c_int
        != 0
    {
        p = p.offset(-1);
        p;
    }
    p = p.offset(1);
    *p = '\0' as i32 as libc::c_char;
    return s;
}
pub fn main() {
    let mut args: Vec<*mut libc::c_char> = Vec::new();
    for arg in ::std::env::args() {
        args.push(
            (::std::ffi::CString::new(arg))
                .expect("Failed to convert argument into CString.")
                .into_raw(),
        );
    }
    args.push(::core::ptr::null_mut());
    unsafe {
        ::std::process::exit(main_0(
            (args.len() - 1) as libc::c_int,
            args.as_mut_ptr() as *mut *mut libc::c_char,
        ) as i32)
    }
}
