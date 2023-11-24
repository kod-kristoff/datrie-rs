use std::io::{self, SeekFrom};

use ::libc;
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use core::mem::size_of;
use libc::c_void;

use crate::{
    fileutils::ReadExt,
    trie_char_string::TrieCharString,
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
#[derive(Clone, Debug)]
// #[repr(C)]
pub struct Tail {
    pub num_tails: TrieIndex,
    pub tails: *mut TailBlock,
    tails2: Vec<TailBlock2>,
    pub first_free: TrieIndex,
}
#[derive(Clone, Debug)]
// #[repr(C)]
pub struct TailBlock {
    pub next_free: TrieIndex,
    pub data: TrieData,
    pub suffix: *mut TrieChar,
}
#[derive(Clone, Debug)]
// #[repr(C)]
pub struct TailBlock2 {
    pub next_free: TrieIndex,
    pub data: TrieData,
    pub suffix: TrieCharString,
}

impl Default for TailBlock2 {
    fn default() -> Self {
        TailBlock2 {
            next_free: -1,
            data: -1,
            suffix: Default::default(),
        }
    }
}
impl TailBlock2 {
    fn reset(&mut self) {
        self.data = -1;
        self.suffix.clear();
    }
}

impl Tail {
    pub fn new() -> Tail {
        Tail {
            num_tails: 0,
            tails: std::ptr::null_mut(),
            first_free: 0,
            tails2: Vec::new(),
        }
    }
}
impl Drop for Tail {
    fn drop(&mut self) {
        unsafe {
            if !self.tails.is_null() {
                let mut i = 0 as libc::c_int;
                while i < self.num_tails {
                    if !((*(self.tails).offset(i as isize)).suffix).is_null() {
                        free((*(self.tails).offset(i as isize)).suffix as *mut libc::c_void);
                    }
                    i += 1;
                }
                free(self.tails as *mut libc::c_void);
            }
        }
    }
}
impl Drop for TailBlock {
    fn drop(&mut self) {
        unsafe {
            if !self.suffix.is_null() {
                free(self.suffix as *mut c_void);
            }
        }
    }
}
// #[no_mangle]
// pub unsafe extern "C" fn tail_new() -> *mut Tail {
//     let t: *mut Tail = malloc(::core::mem::size_of::<Tail>() as libc::c_ulong) as *mut Tail;
//     if t.is_null() as libc::c_int as libc::c_long != 0 {
//         return 0 as *mut Tail;
//     }
//     (*t).first_free = 0 as libc::c_int;
//     (*t).num_tails = 0 as libc::c_int;
//     (*t).tails = 0 as *mut TailBlock;
//     return t;
// }
impl Tail {
    pub fn fread_safe<R: ReadExt + io::Seek>(reader: &mut R) -> DatrieResult<Tail> {
        let save_pos = reader.seek(SeekFrom::Current(0))?;
        Tail::do_fread_safe(reader).map_err(|err| {
            if let Err(io_err) = reader.seek(SeekFrom::Start(save_pos)) {
                return io_err.into();
            }
            err
        })
    }
    fn do_fread_safe<R: ReadExt>(reader: &mut R) -> DatrieResult<Tail> {
        let current_block: u64;
        // let mut t: *mut Tail = 0 as *mut Tail;
        // let mut i: TrieIndex = 0;
        let mut sig: uint32 = 0;
        reader.read_uint32(&mut sig)?;
        if sig != Self::SIGNATURE {
            return Err(DatrieError::new(
                crate::ErrorKind::InvalidFileSignature,
                format!("tail: unexpected signature '{}'", sig),
            ));
        }
        // t = unsafe { malloc(::core::mem::size_of::<Tail>() as libc::c_ulong) as *mut Tail };
        // if !(t.is_null() as libc::c_int as libc::c_long != 0) {
        // unsafe {
        // if let Ok(first_free) = reader.read_int32() {
        //     if let Ok(num_tails) = reader.read_int32() {
        //         (*t).first_free = first_free;
        //         (*t).num_tails = num_tails;
        let mut first_free = 0;
        let mut num_tails = 0;
        reader.read_int32(&mut first_free)?;
        reader.read_int32(&mut num_tails)?;
        // {c
        // if !(file_read_int32(file, &mut (*t).first_free) as u64 == 0
        //     || file_read_int32(file, &mut (*t).num_tails) as u64 == 0)
        // {
        if num_tails as libc::c_ulong
            > (18446744073709551615 as libc::c_ulong)
                .wrapping_div(::core::mem::size_of::<TailBlock>() as libc::c_ulong)
        {
            return Err(DatrieError::new(
                crate::ErrorKind::Bug,
                "failed to read tail: num_tails too large".into(),
            ));
        }
        let mut tails2 = Vec::with_capacity(num_tails as usize);
        let tails = unsafe {
            malloc(
                (num_tails as libc::c_ulong)
                    .wrapping_mul(::core::mem::size_of::<TailBlock>() as libc::c_ulong),
            ) as *mut TailBlock
        };
        if !tails.is_null() {
            let mut i = 0 as libc::c_int;
            loop {
                if !(i < num_tails) {
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
                let mut next_free = 0;
                let mut data = 0;
                // reader.read_int32(&mut next_free)?;
                if reader.read_int32(&mut next_free).is_err()
                    || reader.read_int32(&mut data).is_err()
                    || reader.read_int16(&mut length).is_err()
                {
                    dbg!(next_free);
                    current_block = 1386273818809128762;
                    break;
                }
                unsafe {
                    (*tails.offset(i as isize)).next_free = next_free;
                    (*tails.offset(i as isize)).data = data;
                }
                unsafe {
                    (*tails.offset(i as isize)).suffix =
                        malloc((length as libc::c_int + 1 as libc::c_int) as libc::c_ulong)
                            as *mut TrieChar;
                }
                // let mut tail_block2 = TailBlock2 {
                //     next_free,
                //     data,
                let mut suffix_data = vec![0; length as usize];
                // };
                if unsafe {
                    (*tails.offset(i as isize)).suffix.is_null() // as libc::c_int as libc::c_long != 0
                } {
                    dbg!("suffix is null");
                    current_block = 1386273818809128762;
                    break;
                }
                if length as libc::c_int > 0 as libc::c_int {
                    if reader
                        .read_exact(&mut suffix_data)
                        // .read_chars(
                        //     (*tails.offset(i as isize)).suffix as *mut libc::c_char,
                        //     length as libc::c_int,
                        // )
                        .is_err()
                    {
                        unsafe {
                            free((*tails.offset(i as isize)).suffix as *mut libc::c_void);
                        }
                        current_block = 1386273818809128762;
                        break;
                    }
                    unsafe {
                        memcpy(
                            (*tails.offset(i as isize)).suffix as *mut libc::c_void,
                            suffix_data.as_ptr() as *const libc::c_void,
                            length as libc::c_ulong,
                        );
                        // for j in 0..length {
                        //     tail_block2.suffix[j as usize] =
                        //         *(*tails.offset(i as isize)).suffix.offset(j as isize);
                        // }
                    }
                }
                unsafe {
                    *((*tails.offset(i as isize)).suffix).offset(length as isize) =
                        '\0' as i32 as TrieChar;
                }
                // tail_block2.suffix[length as usize] = '\0' as TrieChar;
                tails2.push(TailBlock2 {
                    next_free,
                    data,
                    suffix: TrieCharString::new(suffix_data).unwrap(),
                });
                i += 1;
            }
            match current_block {
                15904375183555213903 => {
                    return Ok(Tail {
                        num_tails,
                        tails,
                        first_free,
                        tails2,
                    })
                }
                _ => unsafe {
                    while i > 0 as libc::c_int {
                        i -= 1;
                        free((*tails.offset(i as isize)).suffix as *mut libc::c_void);
                    }
                    free(tails as *mut libc::c_void);
                },
            }
        }
        // }
        // }
        // }
        //             free(t as *mut libc::c_void);
        //         }
        //     }
        // }
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
// #[no_mangle]
// pub unsafe extern "C" fn tail_free(mut t: *mut Tail) {
//     let mut i: TrieIndex = 0;
//     if !((*t).tails).is_null() {
//         i = 0 as libc::c_int;
//         while i < (*t).num_tails {
//             if !((*((*t).tails).offset(i as isize)).suffix).is_null() {
//                 free((*((*t).tails).offset(i as isize)).suffix as *mut libc::c_void);
//             }
//             i += 1;
//         }
//         free((*t).tails as *mut libc::c_void);
//     }
//     free(t as *mut libc::c_void);
// }
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
    pub fn num_tails(&self) -> usize {
        assert_eq!(self.num_tails as usize, self.tails2.len());
        return self.tails2.len();
    }
    pub fn get_serialized_size(&self) -> usize {
        let static_count =
            ::core::mem::size_of::<int32>() + 2 * ::core::mem::size_of::<TrieIndex>();
        // dbg!(&static_count);
        let mut dynamic_count: usize = 0;
        if self.num_tails() > 0 {
            dynamic_count += (size_of::<TrieIndex>() + size_of::<TrieData>() + size_of::<int16>())
                * self.num_tails();
            for i in 0..(self.num_tails() as isize) {
                let suffix = unsafe { (*self.tails.offset(i)).suffix };
                if !suffix.is_null() {
                    let suffix_len = trie_char_strsize(suffix);
                    assert_eq!(
                        suffix_len,
                        trie_char_strlen(self.tails2[i as usize].suffix.as_ptr())
                    );
                    dynamic_count += trie_char_strsize(suffix) as usize;
                }
            }
        }
        // dbg!(&dynamic_count);
        return static_count + dynamic_count;
    }

    pub fn get_serialized_size2(&self) -> usize {
        let static_count =
            ::core::mem::size_of::<int32>() + 2 * ::core::mem::size_of::<TrieIndex>();
        // dbg!(&static_count);
        let mut dynamic_count: usize = 0;
        if self.num_tails() > 0 {
            dynamic_count += (size_of::<TrieIndex>() + size_of::<TrieData>() + size_of::<int16>())
                * self.num_tails();
            for i in 0..(self.num_tails()) {
                // let suffix = unsafe { (*self.tails.offset(i)).suffix };
                dynamic_count += self.tails2[i].suffix.len();
                // if !suffix.is_null() {
                // dynamic_count += trie_char_strsize(suffix) as usize;
                // }
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
// #[no_mangle]
// pub unsafe extern "C" fn tail_get_suffix(t: *const Tail, mut index: TrieIndex) -> *const TrieChar {
//     index -= 1 as libc::c_int;
//     return if (index < (*t).num_tails) as libc::c_int as libc::c_long != 0 {
//         (*((*t).tails).offset(index as isize)).suffix
//     } else {
//         0 as *mut TrieChar
//     };
// }
// pub unsafe extern "C" fn tail_set_suffix(
//     t: *mut Tail,
//     mut index: TrieIndex,
//     suffix: *const TrieChar,
// ) -> Bool {
//     index -= 1 as libc::c_int;
//     if (index < (*t).num_tails) as libc::c_int as libc::c_long != 0 {
//         let mut tmp: *mut TrieChar = 0 as *mut TrieChar;
//         if !suffix.is_null() {
//             tmp = trie_char_strdup(suffix);
//             if tmp.is_null() as libc::c_int as libc::c_long != 0 {
//                 return DA_FALSE;
//             }
//         }
//         if !((*((*t).tails).offset(index as isize)).suffix).is_null() {
//             free((*((*t).tails).offset(index as isize)).suffix as *mut libc::c_void);
//         }
//         let ref mut fresh1 = (*((*t).tails).offset(index as isize)).suffix;
//         *fresh1 = tmp;
//         return DA_TRUE;
//     }
//     return DA_FALSE;
// }
impl Tail {
    pub unsafe fn get_suffix(&self, mut index: TrieIndex) -> *const TrieChar {
        index -= 1 as libc::c_int;
        return if (index < self.num_tails() as TrieIndex) as libc::c_int as libc::c_long != 0 {
            // (*(self.tails).offset(index as isize)).suffix
            self.tails2[index as usize].suffix.as_ptr()
        } else {
            0 as *mut TrieChar
        };
    }
    pub unsafe fn set_suffix(
        &mut self,
        // t: *mut Tail,
        mut index: TrieIndex,
        suffix: *const TrieChar,
    ) -> Bool {
        index -= 1 as libc::c_int;
        if ((index as usize) < self.num_tails()) as libc::c_int as libc::c_long != 0 {
            let mut tmp: *mut TrieChar = 0 as *mut TrieChar;
            if !suffix.is_null() {
                tmp = trie_char_strdup(suffix);
                if tmp.is_null() as libc::c_int as libc::c_long != 0 {
                    return DA_FALSE;
                }
            }
            if !((*(self.tails).offset(index as isize)).suffix).is_null() {
                free((*(self.tails).offset(index as isize)).suffix as *mut libc::c_void);
            }
            let ref mut fresh1 = (*(self.tails).offset(index as isize)).suffix;
            *fresh1 = tmp;
            if !suffix.is_null() {
                self.tails2[index as usize].suffix.replace_from_ptr(suffix);
            } else {
                self.tails2[index as usize].suffix.clear();
            }
            return DA_TRUE;
        }
        return DA_FALSE;
    }
}
// pub unsafe extern "C" fn tail_add_suffix(
//     mut t: *mut Tail,
//     mut suffix: *const TrieChar,
// ) -> TrieIndex {
//     let mut new_block: TrieIndex = 0;
//     new_block = tail_alloc_block(t);
//     if (0 as libc::c_int == new_block) as libc::c_int as libc::c_long != 0 {
//         return 0 as libc::c_int;
//     }
//     tail_set_suffix(t, new_block, suffix);
//     return new_block;
// }
impl Tail {
    pub unsafe fn add_suffix(
        &mut self,
        // mut t: *mut Tail,
        suffix: *const TrieChar,
    ) -> TrieIndex {
        let new_block: TrieIndex = self.alloc_block();
        if (0 as libc::c_int == new_block) as libc::c_int as libc::c_long != 0 {
            return 0 as libc::c_int;
        }
        self.set_suffix(new_block, suffix);
        return new_block;
    }
    unsafe fn alloc_block(&mut self) -> TrieIndex {
        let block: TrieIndex;
        if 0 as libc::c_int != self.first_free {
            block = self.first_free;
            self.first_free = (*(self.tails).offset(block as isize)).next_free;
        } else {
            // let mut new_block: *mut libc::c_void = 0 as *mut libc::c_void;
            block = self.num_tails;
            let new_block = realloc(
                self.tails as *mut libc::c_void,
                ((self.num_tails + 1 as libc::c_int) as libc::c_ulong)
                    .wrapping_mul(::core::mem::size_of::<TailBlock>() as libc::c_ulong),
            );
            if new_block.is_null() as libc::c_int as libc::c_long != 0 {
                return 0 as libc::c_int;
            }
            self.tails = new_block as *mut TailBlock;
            self.num_tails += 1;
            self.tails2.push(TailBlock2::default());
        }
        (*(self.tails).offset(block as isize)).next_free = -(1 as libc::c_int);
        (*(self.tails).offset(block as isize)).data = -(1 as libc::c_int);
        let ref mut fresh2 = (*(self.tails).offset(block as isize)).suffix;
        *fresh2 = 0 as *mut TrieChar;
        return block + 1 as libc::c_int;
    }
}
// unsafe extern "C" fn tail_alloc_block(mut t: *mut Tail) -> TrieIndex {
//     let mut block: TrieIndex = 0;
//     if 0 as libc::c_int != (*t).first_free {
//         block = (*t).first_free;
//         (*t).first_free = (*((*t).tails).offset(block as isize)).next_free;
//     } else {
//         let mut new_block: *mut libc::c_void = 0 as *mut libc::c_void;
//         block = (*t).num_tails;
//         new_block = realloc(
//             (*t).tails as *mut libc::c_void,
//             (((*t).num_tails + 1 as libc::c_int) as libc::c_ulong)
//                 .wrapping_mul(::core::mem::size_of::<TailBlock>() as libc::c_ulong),
//         );
//         if new_block.is_null() as libc::c_int as libc::c_long != 0 {
//             return 0 as libc::c_int;
//         }
//         (*t).tails = new_block as *mut TailBlock;
//         (*t).num_tails += 1;
//         (*t).num_tails;
//     }
//     (*((*t).tails).offset(block as isize)).next_free = -(1 as libc::c_int);
//     (*((*t).tails).offset(block as isize)).data = -(1 as libc::c_int);
//     let ref mut fresh2 = (*((*t).tails).offset(block as isize)).suffix;
//     *fresh2 = 0 as *mut TrieChar;
//     return block + 1 as libc::c_int;
// }
impl Tail {
    fn check_invariant(&self) {
        assert_eq!(self.num_tails as usize, self.tails2.len());
        for i in 0..self.num_tails() {
            let block = unsafe { &*self.tails.offset(i as isize) };
            assert_eq!(block.next_free, self.tails2[i].next_free);
            assert_eq!(block.data, self.tails2[i].data);
        }
    }
    unsafe fn free_block(&mut self, mut block: TrieIndex) {
        // let mut i: TrieIndex = 0;
        // let mut j: TrieIndex = 0;
        block -= 1 as libc::c_int;
        if block >= self.num_tails() as TrieIndex {
            return;
        }
        (*(self.tails).offset(block as isize)).data = -(1 as libc::c_int);
        if !((*(self.tails).offset(block as isize)).suffix).is_null() {
            free((*(self.tails).offset(block as isize)).suffix as *mut libc::c_void);
            let ref mut fresh3 = (*(self.tails).offset(block as isize)).suffix;
            *fresh3 = 0 as *mut TrieChar;
        }
        self.tails2[block as usize].reset();
        let mut j = 0 as libc::c_int;
        let mut i = self.first_free;
        while i != 0 as libc::c_int && i < block {
            j = i;
            let next_free = self.tails2[i as usize].next_free;
            i = (*(self.tails).offset(i as isize)).next_free;
            assert_eq!(i, next_free);
        }
        (*(self.tails).offset(block as isize)).next_free = i;
        self.tails2[block as usize].next_free = i;
        if 0 as libc::c_int != j {
            (*(self.tails).offset(j as isize)).next_free = block;
            self.tails2[j as usize].next_free = block;
        } else {
            self.first_free = block;
        };
    }
}
impl Tail {
    pub fn get_data(&self, mut index: TrieIndex) -> TrieData {
        index -= 1 as libc::c_int;
        return if ((index as usize) < self.num_tails()) as libc::c_int as libc::c_long != 0 {
            let data = unsafe { (*self.tails.offset(index as isize)).data };
            assert_eq!(data, self.tails2[index as usize].data);
            // data
            self.tails2[index as usize].data
        } else {
            -(1 as libc::c_int)
        };
    }
    pub fn set_data(
        &mut self,
        // mut t: *mut Tail,
        mut index: TrieIndex,
        data: TrieData,
    ) -> Bool {
        index -= 1 as libc::c_int;
        if ((index as usize) < self.num_tails()) as libc::c_int as libc::c_long != 0 {
            unsafe {
                (*self.tails.offset(index as isize)).data = data;
            }
            self.tails2[index as usize].data = data;
            return DA_TRUE;
        }
        return DA_FALSE;
    }
}
// #[no_mangle]
// pub unsafe extern "C" fn tail_get_data(mut t: *const Tail, mut index: TrieIndex) -> TrieData {
//     index -= 1 as libc::c_int;
//     return if (index < (*t).num_tails) as libc::c_int as libc::c_long != 0 {
//         (*((*t).tails).offset(index as isize)).data
//     } else {
//         -(1 as libc::c_int)
//     };
// }
// #[no_mangle]
// pub unsafe extern "C" fn tail_set_data(
//     mut t: *mut Tail,
//     mut index: TrieIndex,
//     mut data: TrieData,
// ) -> Bool {
//     index -= 1 as libc::c_int;
//     if (index < (*t).num_tails) as libc::c_int as libc::c_long != 0 {
//         (*((*t).tails).offset(index as isize)).data = data;
//         return DA_TRUE;
//     }
//     return DA_FALSE;
// }
impl Tail {
    pub unsafe fn delete(&mut self, index: TrieIndex) {
        self.free_block(index);
    }
    pub unsafe fn walk_str(
        &self,
        // mut t: *const Tail,
        mut s: TrieIndex,
        mut suffix_idx: *mut libc::c_short,
        mut str: *const TrieChar,
        mut len: libc::c_int,
    ) -> libc::c_int {
        let mut suffix: *const TrieChar = 0 as *const TrieChar;
        let mut i: libc::c_int = 0;
        let mut j: libc::c_short = 0;
        suffix = self.get_suffix(s);
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
            if '\0' as i32 == *suffix.offset(j as isize) as libc::c_int {
                break;
            }
            j += 1;
        }
        *suffix_idx = j;
        return i;
    }
    pub unsafe fn walk_char(
        &self,
        // mut t: *const Tail,
        s: TrieIndex,
        suffix_idx: *mut libc::c_short,
        c: TrieChar,
    ) -> Bool {
        // let mut suffix: *const TrieChar = 0 as *const TrieChar;
        // let mut suffix_char: TrieChar = 0;
        let suffix = self.get_suffix(s);
        if suffix.is_null() as libc::c_int as libc::c_long != 0 {
            return DA_FALSE;
        }
        let suffix_char = *suffix.offset(*suffix_idx as isize);
        if suffix_char as libc::c_int == c as libc::c_int {
            if '\0' as i32 != suffix_char as libc::c_int {
                *suffix_idx += 1;
            }
            return DA_TRUE;
        }
        return DA_FALSE;
    }
}

#[cfg(test)]
mod tests {
    use crate::DatrieResult;

    use super::Tail;

    #[test]
    fn get_serialized_size_works() -> DatrieResult<()> {
        let mut tail = Tail::new();
        assert_eq!(tail.get_serialized_size(), tail.get_serialized_size2());
        unsafe { tail.alloc_block() };
        assert_eq!(tail.get_serialized_size(), tail.get_serialized_size2());
        unsafe {
            tail.add_suffix(['a' as u8, 'p' as u8, '\0' as u8].as_ptr());
        }
        assert_eq!(tail.get_serialized_size(), tail.get_serialized_size2());
        Ok(())
    }

    #[test]
    fn next_free() {
        let mut tail = Tail::new();
        assert_eq!(tail.tails2.len(), 0);
        assert!(tail.tails.is_null());

        unsafe { tail.alloc_block() };
        assert_eq!(tail.tails2.len(), 1);
        assert_eq!(tail.num_tails, 1);
        assert!(!tail.tails.is_null());
        for i in 0..tail.num_tails() {
            let block = unsafe { &*tail.tails.offset(i as isize) };
            assert_eq!(block.next_free, tail.tails2[i].next_free);
            assert_eq!(block.data, tail.tails2[i].data);
        }
    }
}
