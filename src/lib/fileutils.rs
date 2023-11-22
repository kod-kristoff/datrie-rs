use byteorder::{BigEndian, ReadBytesExt};
// use libc::fread;
use std::{fs, io};

use ::libc;
extern "C" {
    fn fread(
        _: *mut libc::c_void,
        _: libc::c_ulong,
        _: libc::c_ulong,
        _: *mut FILE,
    ) -> libc::c_ulong;
    fn fwrite(
        _: *const libc::c_void,
        _: libc::c_ulong,
        _: libc::c_ulong,
        _: *mut FILE,
    ) -> libc::c_ulong;
}
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
// #[derive(Copy, Clone)]
// #[repr(C)]
// pub struct _IO_FILE {
//     pub _flags: libc::c_int,
//     pub _IO_read_ptr: *mut libc::c_char,
//     pub _IO_read_end: *mut libc::c_char,
//     pub _IO_read_base: *mut libc::c_char,
//     pub _IO_write_base: *mut libc::c_char,
//     pub _IO_write_ptr: *mut libc::c_char,
//     pub _IO_write_end: *mut libc::c_char,
//     pub _IO_buf_base: *mut libc::c_char,
//     pub _IO_buf_end: *mut libc::c_char,
//     pub _IO_save_base: *mut libc::c_char,
//     pub _IO_backup_base: *mut libc::c_char,
//     pub _IO_save_end: *mut libc::c_char,
//     pub _markers: *mut _IO_marker,
//     pub _chain: *mut _IO_FILE,
//     pub _fileno: libc::c_int,
//     pub _flags2: libc::c_int,
//     pub _old_offset: __off_t,
//     pub _cur_column: libc::c_ushort,
//     pub _vtable_offset: libc::c_schar,
//     pub _shortbuf: [libc::c_char; 1],
//     pub _lock: *mut libc::c_void,
//     pub _offset: __off64_t,
//     pub _codecvt: *mut _IO_codecvt,
//     pub _wide_data: *mut _IO_wide_data,
//     pub _freeres_list: *mut _IO_FILE,
//     pub _freeres_buf: *mut libc::c_void,
//     pub __pad5: size_t,
//     pub _mode: libc::c_int,
//     pub _unused2: [libc::c_char; 20],
// }
// #[repr(C)]
// pub struct _IO_marker {
//     pub _next: *mut _IO_marker,
//     pub _sbuf: *mut _IO_FILE,
//     pub _pos: i32,
// }
// #[repr(C)]
// #[derive(Debug, Copy, Clone)]
// pub struct _IO_codecvt {
//     _unused: [u8; 0],
// }
// #[repr(C)]
// #[derive(Debug, Copy, Clone)]
// pub struct _IO_wide_data {
//     _unused: [u8; 0],
// }
// pub type _IO_lock_t = ();
pub type FILE = libc::FILE;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;
pub type uint8 = libc::c_uchar;
pub type int8 = libc::c_schar;
pub type int16 = libc::c_short;
pub type int32 = libc::c_int;

pub struct CFile {
    file: *mut FILE,
}

impl CFile {
    pub fn new(file: *mut FILE) -> CFile {
        CFile { file }
    }
}
impl io::Seek for CFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            io::SeekFrom::Start(offset) => (offset as i64, libc::SEEK_SET),
            io::SeekFrom::Current(offset) => (offset as i64, libc::SEEK_CUR),
            io::SeekFrom::End(offset) => (offset as i64, libc::SEEK_END),
            _ => todo!(),
        };
        let res = unsafe { libc::fseek(self.file, offset, whence) };
        if res != 0 {
            todo!()
        }
        let new_pos = unsafe { libc::ftell(self.file) };
        Ok(new_pos as u64)
    }
}
// impl io::Seek for FileDescriptor {
//     fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {

//     }
// }
impl io::Read for CFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = unsafe {
            fread(
                buf.as_mut_ptr() as *mut libc::c_void,
                ::core::mem::size_of::<u8>() as libc::c_ulong,
                buf.len() as u64,
                self.file,
            )
        };
        Ok(len as usize)
    }
}
unsafe extern "C" fn parse_int32_be(mut buff: *const uint8) -> int32 {
    return (*buff.offset(0 as libc::c_int as isize) as libc::c_int) << 24 as libc::c_int
        | (*buff.offset(1 as libc::c_int as isize) as libc::c_int) << 16 as libc::c_int
        | (*buff.offset(2 as libc::c_int as isize) as libc::c_int) << 8 as libc::c_int
        | *buff.offset(3 as libc::c_int as isize) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_int32(mut file: *mut FILE, mut o_val: *mut int32) -> Bool {
    let mut buff: [uint8; 4] = [0; 4];
    if fread(
        buff.as_mut_ptr() as *mut libc::c_void,
        4 as libc::c_int as libc::c_ulong,
        1 as libc::c_int as libc::c_ulong,
        file,
    ) == 1 as libc::c_int as libc::c_ulong
    {
        *o_val = parse_int32_be(buff.as_mut_ptr());
        return DA_TRUE;
    }
    return DA_FALSE;
}
pub trait ReadExt: io::Read + Sized {
    fn read_int32(&mut self, val: &mut i32) -> io::Result<()>;
    fn read_uint32(&mut self, val: &mut u32) -> io::Result<()>;
    fn read_int16(&mut self, val: &mut i16) -> io::Result<()>;
    unsafe fn read_chars(&mut self, buf: *mut libc::c_char, len: libc::c_int) -> io::Result<()>;
    fn read_chars_into(&mut self, buf: &mut [u8]) -> io::Result<()>;
}
pub trait ReadSeekExt: ReadExt + io::Seek
where
    Self: Sized,
{
}

impl<T: io::Read> ReadExt for T {
    fn read_int32(&mut self, val: &mut i32) -> io::Result<()> {
        *val = self.read_i32::<BigEndian>()?;
        Ok(())
    }
    fn read_uint32(&mut self, val: &mut u32) -> io::Result<()> {
        *val = self.read_u32::<BigEndian>()?;
        Ok(())
    }
    unsafe fn read_chars(&mut self, buf: *mut libc::c_char, len: libc::c_int) -> io::Result<()> {
        let slice = std::slice::from_raw_parts_mut(buf, len as usize);
        self.read_exact(slice)
    }
    fn read_chars_into(&mut self, buf: &mut [u8]) -> io::Result<()> {
        // self.read_u32_into_into()
        todo!()
    }
    fn read_int16(&mut self, val: &mut i16) -> io::Result<()> {
        *val = self.read_i16::<BigEndian>()?;
        Ok(())
    }
}
impl<T: ReadExt + io::Seek> ReadSeekExt for T {}

unsafe extern "C" fn serialize_int32_be(buff: *mut uint8, val: int32) {
    *buff.offset(0 as libc::c_int as isize) =
        (val >> 24 as libc::c_int & 0xff as libc::c_int) as uint8;
    *buff.offset(1 as libc::c_int as isize) =
        (val >> 16 as libc::c_int & 0xff as libc::c_int) as uint8;
    *buff.offset(2 as libc::c_int as isize) =
        (val >> 8 as libc::c_int & 0xff as libc::c_int) as uint8;
    *buff.offset(3 as libc::c_int as isize) = (val & 0xff as libc::c_int) as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn serialize_int32_be_incr(buff: *mut *mut uint8, val: int32) {
    serialize_int32_be(*buff, val);
    *buff = (*buff).offset(4 as libc::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn file_write_int32(mut file: *mut FILE, mut val: int32) -> Bool {
    let mut buff: [uint8; 4] = [0; 4];
    serialize_int32_be(buff.as_mut_ptr(), val);
    return (fwrite(
        buff.as_mut_ptr() as *const libc::c_void,
        4 as libc::c_int as libc::c_ulong,
        1 as libc::c_int as libc::c_ulong,
        file,
    ) == 1 as libc::c_int as libc::c_ulong) as libc::c_int as Bool;
}
unsafe extern "C" fn parse_int16_be(mut buff: *const uint8) -> int16 {
    return ((*buff.offset(0 as libc::c_int as isize) as libc::c_int) << 8 as libc::c_int
        | *buff.offset(1 as libc::c_int as isize) as libc::c_int) as int16;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_int16(mut file: *mut FILE, mut o_val: *mut int16) -> Bool {
    let mut buff: [uint8; 2] = [0; 2];
    if fread(
        buff.as_mut_ptr() as *mut libc::c_void,
        2 as libc::c_int as libc::c_ulong,
        1 as libc::c_int as libc::c_ulong,
        file,
    ) == 1 as libc::c_int as libc::c_ulong
    {
        *o_val = parse_int16_be(buff.as_mut_ptr());
        return DA_TRUE;
    }
    return DA_FALSE;
}
unsafe extern "C" fn serialize_int16_be(mut buff: *mut uint8, mut val: int16) {
    *buff.offset(0 as libc::c_int as isize) = (val as libc::c_int >> 8 as libc::c_int) as uint8;
    *buff.offset(1 as libc::c_int as isize) = (val as libc::c_int & 0xff as libc::c_int) as uint8;
}
#[no_mangle]
pub unsafe extern "C" fn serialize_int16_be_incr(mut buff: *mut *mut uint8, mut val: int16) {
    serialize_int16_be(*buff, val);
    *buff = (*buff).offset(2 as libc::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn file_write_int16(mut file: *mut FILE, mut val: int16) -> Bool {
    let mut buff: [uint8; 2] = [0; 2];
    serialize_int16_be(buff.as_mut_ptr(), val);
    return (fwrite(
        buff.as_mut_ptr() as *const libc::c_void,
        2 as libc::c_int as libc::c_ulong,
        1 as libc::c_int as libc::c_ulong,
        file,
    ) == 1 as libc::c_int as libc::c_ulong) as libc::c_int as Bool;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_int8(mut file: *mut FILE, mut o_val: *mut int8) -> Bool {
    return (fread(
        o_val as *mut libc::c_void,
        ::core::mem::size_of::<int8>() as libc::c_ulong,
        1 as libc::c_int as libc::c_ulong,
        file,
    ) == 1 as libc::c_int as libc::c_ulong) as libc::c_int as Bool;
}
#[no_mangle]
pub unsafe extern "C" fn file_write_int8(mut file: *mut FILE, mut val: int8) -> Bool {
    return (fwrite(
        &mut val as *mut int8 as *const libc::c_void,
        ::core::mem::size_of::<int8>() as libc::c_ulong,
        1 as libc::c_int as libc::c_ulong,
        file,
    ) == 1 as libc::c_int as libc::c_ulong) as libc::c_int as Bool;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_chars(
    mut file: *mut FILE,
    mut buff: *mut libc::c_char,
    mut len: libc::c_int,
) -> Bool {
    return (fread(
        buff as *mut libc::c_void,
        ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
        len as libc::c_ulong,
        file,
    ) == len as libc::c_ulong) as libc::c_int as Bool;
}
#[no_mangle]
pub unsafe extern "C" fn file_write_chars(
    mut file: *mut FILE,
    mut buff: *const libc::c_char,
    mut len: libc::c_int,
) -> Bool {
    return (fwrite(
        buff as *const libc::c_void,
        ::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
        len as libc::c_ulong,
        file,
    ) == len as libc::c_ulong) as libc::c_int as Bool;
}
