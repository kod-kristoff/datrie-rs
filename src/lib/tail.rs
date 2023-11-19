use std::io::{self, SeekFrom};

use ::libc;
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use core::mem::size_of;

use crate::{
    fileutils::ReadExt,
    trie_string::{trie_char_strdup, trie_char_strlen, trie_char_strsize},
    DatrieError, DatrieResult,
};

extern "C" {
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: libc::c_int) -> libc::c_int;
    fn ftell(__stream: *mut FILE) -> libc::c_long;
    fn serialize_int32_be_incr(buff: *mut *mut uint8, val: int32);
    fn file_read_int32(file: *mut FILE, o_val: *mut int32) -> Bool;
    fn file_write_int32(file: *mut FILE, val: int32) -> Bool;
    fn serialize_int16_be_incr(buff: *mut *mut uint8, val: int16);
    fn file_read_int16(file: *mut FILE, o_val: *mut int16) -> Bool;
    fn file_write_int16(file: *mut FILE, val: int16) -> Bool;
    fn file_read_chars(file: *mut FILE, buff: *mut libc::c_char, len: libc::c_int) -> Bool;
    fn file_write_chars(file: *mut FILE, buff: *const libc::c_char, len: libc::c_int) -> Bool;
}
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
pub type FILE = libc::FILE;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type uint8 = libc::c_uchar;
pub type int16 = libc::c_short;
pub type uint32 = libc::c_uint;
pub type int32 = libc::c_int;
pub type TrieChar = libc::c_uchar;
pub type TrieIndex = int32;
pub type TrieData = int32;
#[derive(Copy, Clone)]
// #[repr(C)]
pub struct Tail {
    pub num_tails: TrieIndex,
    pub tails: *mut TailBlock,
    pub first_free: TrieIndex,
}
#[derive(Copy, Clone)]
// #[repr(C)]
pub struct TailBlock {
    pub next_free: TrieIndex,
    pub data: TrieData,
    pub suffix: *mut TrieChar,
}

impl Tail {
    pub fn new() -> Tail {
        Tail {
            num_tails: 0,
            tails: std::ptr::null_mut(),
            first_free: 0,
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn tail_new() -> *mut Tail {
    let t: *mut Tail = malloc(::core::mem::size_of::<Tail>() as libc::c_ulong) as *mut Tail;
    if t.is_null() as libc::c_int as libc::c_long != 0 {
        return 0 as *mut Tail;
    }
    (*t).first_free = 0 as libc::c_int;
    (*t).num_tails = 0 as libc::c_int;
    (*t).tails = 0 as *mut TailBlock;
    return t;
}
impl Tail {
    pub fn fread_safe<R: ReadExt + io::Seek>(reader: &mut R) -> DatrieResult<*mut Tail> {
        let save_pos = reader.seek(SeekFrom::Current(0))?;
        Tail::do_fread_safe(reader).map_err(|err| {
            if let Err(io_err) = reader.seek(SeekFrom::Start(save_pos)) {
                return io_err.into();
            }
            err
        })
    }
    fn do_fread_safe<R: ReadExt>(reader: &mut R) -> DatrieResult<*mut Tail> {
        let mut current_block: u64;
        let mut t: *mut Tail = 0 as *mut Tail;
        let mut i: TrieIndex = 0;
        let mut sig: uint32 = 0;
        reader.read_uint32(&mut sig)?;
        if 0xdffcdffc == sig {
            t = unsafe { malloc(::core::mem::size_of::<Tail>() as libc::c_ulong) as *mut Tail };
            if !(t.is_null() as libc::c_int as libc::c_long != 0) {
                unsafe {
                    // if let Ok(first_free) = reader.read_int32() {
                    //     if let Ok(num_tails) = reader.read_int32() {
                    //         (*t).first_free = first_free;
                    //         (*t).num_tails = num_tails;
                    if reader.read_int32(&mut (*t).first_free).is_ok()
                        && reader.read_int32(&mut (*t).num_tails).is_ok()
                    {
                        // if !(file_read_int32(file, &mut (*t).first_free) as u64 == 0
                        //     || file_read_int32(file, &mut (*t).num_tails) as u64 == 0)
                        // {
                        if !((*t).num_tails as libc::c_ulong
                            > (18446744073709551615 as libc::c_ulong)
                                .wrapping_div(::core::mem::size_of::<TailBlock>() as libc::c_ulong))
                        {
                            (*t).tails =
                                malloc(((*t).num_tails as libc::c_ulong).wrapping_mul(
                                    ::core::mem::size_of::<TailBlock>() as libc::c_ulong,
                                )) as *mut TailBlock;
                            if !(((*t).tails).is_null() as libc::c_int as libc::c_long != 0) {
                                i = 0 as libc::c_int;
                                loop {
                                    if !(i < (*t).num_tails) {
                                        current_block = 15904375183555213903;
                                        break;
                                    }
                                    let mut length: int16 = 0;
                                    // if reader.read_int32(
                                    //    &mut (*((*t).tails).offset(i as isize)).next_free).is_err() ||
                                    // } else {
                                    //     current_block = 1386273818809128762;
                                    //     break;
                                    // }
                                    // if let Ok(data) = reader.read_int32() {
                                    //     (*((*t).tails).offset(i as isize)).data = data;
                                    // } else {
                                    //     current_block = 1386273818809128762;
                                    //     break;
                                    // }
                                    // if let Ok(v_length) = reader.read_int16() {
                                    //     length = v_length;
                                    // } else {
                                    //     current_block = 1386273818809128762;
                                    //     break;
                                    // }
                                    if reader
                                        .read_int32(
                                            &mut (*((*t).tails).offset(i as isize)).next_free,
                                        )
                                        .is_err()
                                        || reader
                                            .read_int32(
                                                &mut (*((*t).tails).offset(i as isize)).data,
                                            )
                                            .is_err()
                                        || reader.read_int16(&mut length).is_err()
                                    {
                                        current_block = 1386273818809128762;
                                        break;
                                    }
                                    let ref mut fresh0 = (*((*t).tails).offset(i as isize)).suffix;
                                    *fresh0 = malloc(
                                        (length as libc::c_int + 1 as libc::c_int) as libc::c_ulong,
                                    )
                                        as *mut TrieChar;
                                    if ((*((*t).tails).offset(i as isize)).suffix).is_null()
                                        as libc::c_int
                                        as libc::c_long
                                        != 0
                                    {
                                        current_block = 1386273818809128762;
                                        break;
                                    }
                                    if length as libc::c_int > 0 as libc::c_int {
                                        if reader
                                            .read_chars(
                                                (*((*t).tails).offset(i as isize)).suffix
                                                    as *mut libc::c_char,
                                                length as libc::c_int,
                                            )
                                            .is_err()
                                        {
                                            free(
                                                (*((*t).tails).offset(i as isize)).suffix
                                                    as *mut libc::c_void,
                                            );
                                            current_block = 1386273818809128762;
                                            break;
                                        }
                                    }
                                    *((*((*t).tails).offset(i as isize)).suffix)
                                        .offset(length as isize) = '\0' as i32 as TrieChar;
                                    i += 1;
                                    i;
                                }
                                match current_block {
                                    15904375183555213903 => return Ok(t),
                                    _ => {
                                        while i > 0 as libc::c_int {
                                            i -= 1;
                                            free(
                                                (*((*t).tails).offset(i as isize)).suffix
                                                    as *mut libc::c_void,
                                            );
                                        }
                                        free((*t).tails as *mut libc::c_void);
                                    }
                                }
                            }
                        }
                        // }
                    }
                    free(t as *mut libc::c_void);
                }
            }
        }
        // return 0 as *mut Tail;
        return Err(DatrieError::new(
            crate::ErrorKind::Bug,
            "failed to read tail".into(),
        ));
    }
}
#[no_mangle]
pub unsafe extern "C" fn tail_fread(file: *mut FILE) -> *mut Tail {
    let mut current_block: u64;
    let mut save_pos: libc::c_long = 0;
    let mut t: *mut Tail = 0 as *mut Tail;
    let mut i: TrieIndex = 0;
    let mut sig: uint32 = 0;
    save_pos = ftell(file);
    if !(file_read_int32(file, &mut sig as *mut uint32 as *mut int32) as u64 == 0
        || 0xdffcdffc as libc::c_uint != sig)
    {
        t = malloc(::core::mem::size_of::<Tail>() as libc::c_ulong) as *mut Tail;
        if !(t.is_null() as libc::c_int as libc::c_long != 0) {
            if !(file_read_int32(file, &mut (*t).first_free) as u64 == 0
                || file_read_int32(file, &mut (*t).num_tails) as u64 == 0)
            {
                if !((*t).num_tails as libc::c_ulong
                    > (18446744073709551615 as libc::c_ulong)
                        .wrapping_div(::core::mem::size_of::<TailBlock>() as libc::c_ulong))
                {
                    (*t).tails = malloc(
                        ((*t).num_tails as libc::c_ulong)
                            .wrapping_mul(::core::mem::size_of::<TailBlock>() as libc::c_ulong),
                    ) as *mut TailBlock;
                    if !(((*t).tails).is_null() as libc::c_int as libc::c_long != 0) {
                        i = 0 as libc::c_int;
                        loop {
                            if !(i < (*t).num_tails) {
                                current_block = 15904375183555213903;
                                break;
                            }
                            let mut length: int16 = 0;
                            if file_read_int32(
                                file,
                                &mut (*((*t).tails).offset(i as isize)).next_free,
                            ) as u64
                                == 0
                                || file_read_int32(
                                    file,
                                    &mut (*((*t).tails).offset(i as isize)).data,
                                ) as u64
                                    == 0
                                || file_read_int16(file, &mut length) as u64 == 0
                            {
                                current_block = 1386273818809128762;
                                break;
                            }
                            let ref mut fresh0 = (*((*t).tails).offset(i as isize)).suffix;
                            *fresh0 =
                                malloc((length as libc::c_int + 1 as libc::c_int) as libc::c_ulong)
                                    as *mut TrieChar;
                            if ((*((*t).tails).offset(i as isize)).suffix).is_null() as libc::c_int
                                as libc::c_long
                                != 0
                            {
                                current_block = 1386273818809128762;
                                break;
                            }
                            if length as libc::c_int > 0 as libc::c_int {
                                if file_read_chars(
                                    file,
                                    (*((*t).tails).offset(i as isize)).suffix as *mut libc::c_char,
                                    length as libc::c_int,
                                ) as u64
                                    == 0
                                {
                                    free(
                                        (*((*t).tails).offset(i as isize)).suffix
                                            as *mut libc::c_void,
                                    );
                                    current_block = 1386273818809128762;
                                    break;
                                }
                            }
                            *((*((*t).tails).offset(i as isize)).suffix).offset(length as isize) =
                                '\0' as i32 as TrieChar;
                            i += 1;
                            i;
                        }
                        match current_block {
                            15904375183555213903 => return t,
                            _ => {
                                while i > 0 as libc::c_int {
                                    i -= 1;
                                    free(
                                        (*((*t).tails).offset(i as isize)).suffix
                                            as *mut libc::c_void,
                                    );
                                }
                                free((*t).tails as *mut libc::c_void);
                            }
                        }
                    }
                }
            }
            free(t as *mut libc::c_void);
        }
    }
    fseek(file, save_pos, 0 as libc::c_int);
    return 0 as *mut Tail;
}
#[no_mangle]
pub unsafe extern "C" fn tail_free(mut t: *mut Tail) {
    let mut i: TrieIndex = 0;
    if !((*t).tails).is_null() {
        i = 0 as libc::c_int;
        while i < (*t).num_tails {
            if !((*((*t).tails).offset(i as isize)).suffix).is_null() {
                free((*((*t).tails).offset(i as isize)).suffix as *mut libc::c_void);
            }
            i += 1;
        }
        free((*t).tails as *mut libc::c_void);
    }
    free(t as *mut libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tail_fwrite(mut t: *const Tail, mut file: *mut FILE) -> libc::c_int {
    let mut i: TrieIndex = 0;
    if file_write_int32(file, 0xdffcdffc as libc::c_uint as int32) as u64 == 0
        || file_write_int32(file, (*t).first_free) as u64 == 0
        || file_write_int32(file, (*t).num_tails) as u64 == 0
    {
        return -(1 as libc::c_int);
    }
    i = 0 as libc::c_int;
    while i < (*t).num_tails {
        let mut length: int16 = 0;
        if file_write_int32(file, (*((*t).tails).offset(i as isize)).next_free) as u64 == 0
            || file_write_int32(file, (*((*t).tails).offset(i as isize)).data) as u64 == 0
        {
            return -(1 as libc::c_int);
        }
        length = (if !((*((*t).tails).offset(i as isize)).suffix).is_null() {
            trie_char_strlen((*((*t).tails).offset(i as isize)).suffix)
        } else {
            0 as libc::c_int as libc::c_ulong
        }) as int16;
        if file_write_int16(file, length) as u64 == 0 {
            return -(1 as libc::c_int);
        }
        if length as libc::c_int > 0 as libc::c_int
            && file_write_chars(
                file,
                (*((*t).tails).offset(i as isize)).suffix as *mut libc::c_char,
                length as libc::c_int,
            ) as u64
                == 0
        {
            return -(1 as libc::c_int);
        }
        i += 1;
    }
    return 0 as libc::c_int;
}

impl Tail {
    pub fn get_serialized_size(&self) -> usize {
        let static_count =
            ::core::mem::size_of::<int32>() + 2 * ::core::mem::size_of::<TrieIndex>();
        // dbg!(&static_count);
        let mut dynamic_count: usize = 0;
        if self.num_tails > 0 {
            dynamic_count += (size_of::<TrieIndex>() + size_of::<TrieData>() + size_of::<int16>())
                * self.num_tails as usize;
            for i in 0..(self.num_tails as isize) {
                let suffix = unsafe { (*self.tails.offset(i)).suffix };
                if !suffix.is_null() {
                    dynamic_count += trie_char_strsize(suffix) as usize;
                }
            }
        }
        // dbg!(&dynamic_count);
        return static_count + dynamic_count;
    }

    const SIGNATURE: u32 = 0xdffcdffc;
    pub fn serialize(&self, writer: &mut dyn std::io::Write) -> DatrieResult<()> {
        writer.write_i32::<BigEndian>(Self::SIGNATURE as i32)?;
        writer.write_i32::<BigEndian>(self.first_free)?;
        writer.write_i32::<BigEndian>(self.num_tails)?;
        for i in 0..self.num_tails {
            let next_free = unsafe { (*self.tails.offset(i as isize)).next_free };
            let data = unsafe { (*self.tails.offset(i as isize)).data };
            writer.write_i32::<BigEndian>(next_free)?;
            writer.write_i32::<BigEndian>(data)?;
            let suffix = unsafe { (*self.tails.offset(i as isize)).suffix };
            let length = if suffix.is_null() {
                0
            } else {
                trie_char_strsize(suffix)
            };
            writer.write_i16::<BigEndian>(length as i16)?;
            let suffix_slice = unsafe { std::slice::from_raw_parts(suffix, length as usize) };
            writer.write_all(suffix_slice)?;
        }
        Ok(())
    }
    pub fn serialize_to_slice(&self, mut buf: &mut [u8]) -> DatrieResult<usize> {
        buf.write_i32::<BigEndian>(Self::SIGNATURE as i32)?;
        buf.write_i32::<BigEndian>(self.first_free)?;
        buf.write_i32::<BigEndian>(self.num_tails)?;
        let mut written: usize = 12;
        for i in 0..self.num_tails {
            let next_free = unsafe { (*self.tails.offset(i as isize)).next_free };
            let data = unsafe { (*self.tails.offset(i as isize)).data };
            buf.write_i32::<BigEndian>(next_free)?;
            buf.write_i32::<BigEndian>(data)?;
            let suffix = unsafe { (*self.tails.offset(i as isize)).suffix };
            let length = if suffix.is_null() {
                0
            } else {
                trie_char_strsize(suffix)
            };
            buf.write_i16::<BigEndian>(length as i16)?;
            written += 10;
            let suffix_slice = unsafe { std::slice::from_raw_parts(suffix, length as usize) };
            for byte in suffix_slice {
                buf.write_u8(*byte)?;
            }
            written += length as usize;
        }
        Ok(written)
    }
}

#[no_mangle]
pub unsafe extern "C" fn tail_serialize(t: *const Tail, ptr: *mut *mut uint8) -> libc::c_int {
    let mut i: TrieIndex = 0;
    serialize_int32_be_incr(ptr, 0xdffcdffc as libc::c_uint as int32);
    serialize_int32_be_incr(ptr, (*t).first_free);
    serialize_int32_be_incr(ptr, (*t).num_tails);
    i = 0 as libc::c_int;
    while i < (*t).num_tails {
        let mut length: int16 = 0;
        serialize_int32_be_incr(ptr, (*((*t).tails).offset(i as isize)).next_free);
        serialize_int32_be_incr(ptr, (*((*t).tails).offset(i as isize)).data);
        length = (if !((*((*t).tails).offset(i as isize)).suffix).is_null() {
            trie_char_strsize((*((*t).tails).offset(i as isize)).suffix)
        } else {
            0 as libc::c_int as libc::c_ulong
        }) as int16;
        serialize_int16_be_incr(ptr, length);
        if length != 0 {
            memcpy(
                *ptr as *mut libc::c_void,
                (*((*t).tails).offset(i as isize)).suffix as *mut libc::c_char
                    as *const libc::c_void,
                length as libc::c_ulong,
            );
            *ptr = (*ptr).offset(length as libc::c_int as isize);
        }
        i += 1;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tail_get_suffix(t: *const Tail, mut index: TrieIndex) -> *const TrieChar {
    index -= 1 as libc::c_int;
    return if (index < (*t).num_tails) as libc::c_int as libc::c_long != 0 {
        (*((*t).tails).offset(index as isize)).suffix
    } else {
        0 as *mut TrieChar
    };
}
#[no_mangle]
pub unsafe extern "C" fn tail_set_suffix(
    t: *mut Tail,
    mut index: TrieIndex,
    suffix: *const TrieChar,
) -> Bool {
    index -= 1 as libc::c_int;
    if (index < (*t).num_tails) as libc::c_int as libc::c_long != 0 {
        let mut tmp: *mut TrieChar = 0 as *mut TrieChar;
        if !suffix.is_null() {
            tmp = trie_char_strdup(suffix);
            if tmp.is_null() as libc::c_int as libc::c_long != 0 {
                return DA_FALSE;
            }
        }
        if !((*((*t).tails).offset(index as isize)).suffix).is_null() {
            free((*((*t).tails).offset(index as isize)).suffix as *mut libc::c_void);
        }
        let ref mut fresh1 = (*((*t).tails).offset(index as isize)).suffix;
        *fresh1 = tmp;
        return DA_TRUE;
    }
    return DA_FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn tail_add_suffix(
    mut t: *mut Tail,
    mut suffix: *const TrieChar,
) -> TrieIndex {
    let mut new_block: TrieIndex = 0;
    new_block = tail_alloc_block(t);
    if (0 as libc::c_int == new_block) as libc::c_int as libc::c_long != 0 {
        return 0 as libc::c_int;
    }
    tail_set_suffix(t, new_block, suffix);
    return new_block;
}
unsafe extern "C" fn tail_alloc_block(mut t: *mut Tail) -> TrieIndex {
    let mut block: TrieIndex = 0;
    if 0 as libc::c_int != (*t).first_free {
        block = (*t).first_free;
        (*t).first_free = (*((*t).tails).offset(block as isize)).next_free;
    } else {
        let mut new_block: *mut libc::c_void = 0 as *mut libc::c_void;
        block = (*t).num_tails;
        new_block = realloc(
            (*t).tails as *mut libc::c_void,
            (((*t).num_tails + 1 as libc::c_int) as libc::c_ulong)
                .wrapping_mul(::core::mem::size_of::<TailBlock>() as libc::c_ulong),
        );
        if new_block.is_null() as libc::c_int as libc::c_long != 0 {
            return 0 as libc::c_int;
        }
        (*t).tails = new_block as *mut TailBlock;
        (*t).num_tails += 1;
        (*t).num_tails;
    }
    (*((*t).tails).offset(block as isize)).next_free = -(1 as libc::c_int);
    (*((*t).tails).offset(block as isize)).data = -(1 as libc::c_int);
    let ref mut fresh2 = (*((*t).tails).offset(block as isize)).suffix;
    *fresh2 = 0 as *mut TrieChar;
    return block + 1 as libc::c_int;
}
unsafe extern "C" fn tail_free_block(mut t: *mut Tail, mut block: TrieIndex) {
    let mut i: TrieIndex = 0;
    let mut j: TrieIndex = 0;
    block -= 1 as libc::c_int;
    if block >= (*t).num_tails {
        return;
    }
    (*((*t).tails).offset(block as isize)).data = -(1 as libc::c_int);
    if !((*((*t).tails).offset(block as isize)).suffix).is_null() {
        free((*((*t).tails).offset(block as isize)).suffix as *mut libc::c_void);
        let ref mut fresh3 = (*((*t).tails).offset(block as isize)).suffix;
        *fresh3 = 0 as *mut TrieChar;
    }
    j = 0 as libc::c_int;
    i = (*t).first_free;
    while i != 0 as libc::c_int && i < block {
        j = i;
        i = (*((*t).tails).offset(i as isize)).next_free;
    }
    (*((*t).tails).offset(block as isize)).next_free = i;
    if 0 as libc::c_int != j {
        (*((*t).tails).offset(j as isize)).next_free = block;
    } else {
        (*t).first_free = block;
    };
}
#[no_mangle]
pub unsafe extern "C" fn tail_get_data(mut t: *const Tail, mut index: TrieIndex) -> TrieData {
    index -= 1 as libc::c_int;
    return if (index < (*t).num_tails) as libc::c_int as libc::c_long != 0 {
        (*((*t).tails).offset(index as isize)).data
    } else {
        -(1 as libc::c_int)
    };
}
#[no_mangle]
pub unsafe extern "C" fn tail_set_data(
    mut t: *mut Tail,
    mut index: TrieIndex,
    mut data: TrieData,
) -> Bool {
    index -= 1 as libc::c_int;
    if (index < (*t).num_tails) as libc::c_int as libc::c_long != 0 {
        (*((*t).tails).offset(index as isize)).data = data;
        return DA_TRUE;
    }
    return DA_FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn tail_delete(mut t: *mut Tail, mut index: TrieIndex) {
    tail_free_block(t, index);
}
#[no_mangle]
pub unsafe extern "C" fn tail_walk_str(
    mut t: *const Tail,
    mut s: TrieIndex,
    mut suffix_idx: *mut libc::c_short,
    mut str: *const TrieChar,
    mut len: libc::c_int,
) -> libc::c_int {
    let mut suffix: *const TrieChar = 0 as *const TrieChar;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_short = 0;
    suffix = tail_get_suffix(t, s);
    if suffix.is_null() as libc::c_int as libc::c_long != 0 {
        return DA_FALSE as libc::c_int;
    }
    i = 0 as libc::c_int;
    j = *suffix_idx;
    while i < len {
        if *str.offset(i as isize) as libc::c_int != *suffix.offset(j as isize) as libc::c_int {
            break;
        }
        i += 1;
        i;
        if '\0' as i32 == *suffix.offset(j as isize) as libc::c_int {
            break;
        }
        j += 1;
        j;
    }
    *suffix_idx = j;
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn tail_walk_char(
    mut t: *const Tail,
    mut s: TrieIndex,
    mut suffix_idx: *mut libc::c_short,
    mut c: TrieChar,
) -> Bool {
    let mut suffix: *const TrieChar = 0 as *const TrieChar;
    let mut suffix_char: TrieChar = 0;
    suffix = tail_get_suffix(t, s);
    if suffix.is_null() as libc::c_int as libc::c_long != 0 {
        return DA_FALSE;
    }
    suffix_char = *suffix.offset(*suffix_idx as isize);
    if suffix_char as libc::c_int == c as libc::c_int {
        if '\0' as i32 != suffix_char as libc::c_int {
            *suffix_idx += 1;
        }
        return DA_TRUE;
    }
    return DA_FALSE;
}

#[cfg(test)]
mod tests {
    use crate::DatrieResult;

    use super::Tail;

    #[test]
    fn get_serialized_size_works() -> DatrieResult<()> {
        let tail = Tail::new();
        let size = tail.get_serialized_size();
        assert_eq!(size, 12);
        Ok(())
    }
}
