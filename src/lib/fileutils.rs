use byteorder::{BigEndian, ReadBytesExt};
// use libc::fread;
use std::{io, ptr};

pub type FILE = libc::FILE;
pub type Bool = libc::c_uint;
pub const DA_TRUE: Bool = 1;
pub const DA_FALSE: Bool = 0;

pub struct CFile {
    file: *mut FILE,
    close_on_drop: bool,
}

impl CFile {
    pub fn new(file: *mut FILE, close_on_drop: bool) -> CFile {
        CFile {
            file,
            close_on_drop,
        }
    }
}
impl Drop for CFile {
    fn drop(&mut self) {
        if self.close_on_drop {
            let res = unsafe { libc::fclose(self.file) };
            if res < 0 {
                eprintln!("Call to libc::fclose returned '{res}'");
            }
            self.file = ptr::null_mut();
            self.close_on_drop = false;
        }
    }
}
impl io::Seek for CFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (offset, whence) = match pos {
            io::SeekFrom::Start(offset) => (offset as i64, libc::SEEK_SET),
            io::SeekFrom::Current(offset) => (offset, libc::SEEK_CUR),
            io::SeekFrom::End(offset) => (offset, libc::SEEK_END),
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
            libc::fread(
                buf.as_mut_ptr() as *mut libc::c_void,
                ::core::mem::size_of::<u8>(),
                buf.len(),
                self.file,
            )
        };
        Ok(len as usize)
    }
}

impl io::Write for CFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_written = unsafe {
            libc::fwrite(
                buf.as_ptr() as *mut libc::c_void,
                ::core::mem::size_of::<u8>(),
                buf.len(),
                self.file,
            )
        };
        Ok(bytes_written)
    }
    fn flush(&mut self) -> io::Result<()> {
        let res = unsafe { libc::fflush(self.file) };
        if res == -1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to flush with libc::fflush",
            ));
        }
        Ok(())
    }
}
unsafe extern "C" fn parse_int32_be(buff: *const u8) -> i32 {
    (*buff.offset(0 as libc::c_int as isize) as libc::c_int) << 24 as libc::c_int
        | (*buff.offset(1 as libc::c_int as isize) as libc::c_int) << 16 as libc::c_int
        | (*buff.offset(2 as libc::c_int as isize) as libc::c_int) << 8 as libc::c_int
        | *buff.offset(3 as libc::c_int as isize) as libc::c_int
}
#[no_mangle]
pub unsafe extern "C" fn file_read_int32(file: *mut FILE, o_val: *mut i32) -> Bool {
    let mut buff: [u8; 4] = [0; 4];
    if libc::fread(buff.as_mut_ptr() as *mut libc::c_void, 4, 1, file) == 1 {
        *o_val = parse_int32_be(buff.as_mut_ptr());
        return DA_TRUE;
    }
    DA_FALSE
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
        let slice = std::slice::from_raw_parts_mut(buf as *mut u8, len as usize);
        self.read_exact(slice)
    }
    fn read_chars_into(&mut self, _buf: &mut [u8]) -> io::Result<()> {
        // self.read_u32_into_into()
        todo!()
    }
    fn read_int16(&mut self, val: &mut i16) -> io::Result<()> {
        *val = self.read_i16::<BigEndian>()?;
        Ok(())
    }
}
impl<T: ReadExt + io::Seek> ReadSeekExt for T {}

unsafe extern "C" fn serialize_int32_be(buff: *mut u8, val: i32) {
    *buff.offset(0 as libc::c_int as isize) =
        (val >> 24 as libc::c_int & 0xff as libc::c_int) as u8;
    *buff.offset(1 as libc::c_int as isize) =
        (val >> 16 as libc::c_int & 0xff as libc::c_int) as u8;
    *buff.offset(2 as libc::c_int as isize) = (val >> 8 as libc::c_int & 0xff as libc::c_int) as u8;
    *buff.offset(3 as libc::c_int as isize) = (val & 0xff as libc::c_int) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn serialize_int32_be_incr(buff: *mut *mut u8, val: i32) {
    serialize_int32_be(*buff, val);
    *buff = (*buff).offset(4 as libc::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn file_write_int32(file: *mut FILE, val: i32) -> Bool {
    let mut buff: [u8; 4] = [0; 4];
    serialize_int32_be(buff.as_mut_ptr(), val);
    (libc::fwrite(buff.as_mut_ptr() as *const libc::c_void, 4, 1, file) == 1) as libc::c_int as Bool
}
unsafe extern "C" fn parse_int16_be(buff: *const u8) -> i16 {
    ((*buff.offset(0 as libc::c_int as isize) as libc::c_int) << 8 as libc::c_int
        | *buff.offset(1 as libc::c_int as isize) as libc::c_int) as i16
}
#[no_mangle]
pub unsafe extern "C" fn file_read_int16(file: *mut FILE, o_val: *mut i16) -> Bool {
    let mut buff: [u8; 2] = [0; 2];
    if libc::fread(buff.as_mut_ptr() as *mut libc::c_void, 2, 1, file) == 1 {
        *o_val = parse_int16_be(buff.as_mut_ptr());
        return DA_TRUE;
    }
    DA_FALSE
}
unsafe extern "C" fn serialize_int16_be(buff: *mut u8, val: i16) {
    *buff.offset(0 as libc::c_int as isize) = (val as libc::c_int >> 8 as libc::c_int) as u8;
    *buff.offset(1 as libc::c_int as isize) = (val as libc::c_int & 0xff as libc::c_int) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn serialize_int16_be_incr(buff: *mut *mut u8, val: i16) {
    serialize_int16_be(*buff, val);
    *buff = (*buff).offset(2 as libc::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn file_write_int16(file: *mut FILE, val: i16) -> Bool {
    let mut buff: [u8; 2] = [0; 2];
    serialize_int16_be(buff.as_mut_ptr(), val);
    (libc::fwrite(buff.as_mut_ptr() as *const libc::c_void, 2, 1, file) == 1) as libc::c_int as Bool
}
#[no_mangle]
pub unsafe extern "C" fn file_read_int8(file: *mut FILE, o_val: *mut i8) -> Bool {
    (libc::fread(
        o_val as *mut libc::c_void,
        ::core::mem::size_of::<i8>(),
        1,
        file,
    ) == 1) as libc::c_int as Bool
}
#[no_mangle]
pub unsafe extern "C" fn file_write_int8(file: *mut FILE, mut val: i8) -> Bool {
    (libc::fwrite(
        &mut val as *mut i8 as *const libc::c_void,
        ::core::mem::size_of::<i8>(),
        1,
        file,
    ) == 1) as libc::c_int as Bool
}
#[no_mangle]
pub unsafe extern "C" fn file_read_chars(
    file: *mut FILE,
    buff: *mut libc::c_char,
    len: libc::c_int,
) -> Bool {
    (libc::fread(
        buff as *mut libc::c_void,
        ::core::mem::size_of::<libc::c_char>(),
        len as usize,
        file,
    ) == len as usize) as libc::c_int as Bool
}
#[no_mangle]
pub unsafe extern "C" fn file_write_chars(
    file: *mut FILE,
    buff: *const libc::c_char,
    len: libc::c_int,
) -> Bool {
    (libc::fwrite(
        buff as *const libc::c_void,
        ::core::mem::size_of::<libc::c_char>(),
        len as usize,
        file,
    ) == len as usize) as libc::c_int as Bool
}
