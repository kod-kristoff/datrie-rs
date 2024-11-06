// /*
//  * libdatrie - Double-Array Trie Library
//  * Copyright (C) 2013  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA
//  */
// /*
//  * test_serialization.c - Test for datrie file and in-memory blob operations
//  * Created: 2019-11-11
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com> and KOLANICH <KOLANICH@users.noreply.github.com>
//  */
use std::{ffi::CString, fs, io::Read};

use datrie_test_suite::{AlphaChar, Bool, Trie, TrieData, DA_OK, DA_TRUE};
use tempfile::tempdir;

use crate::utils::{en_trie_new, get_dict_src, msg_step};

extern "C" {

    fn trie_store(trie: *mut Trie, key: *const AlphaChar, data: TrieData) -> Bool;
    fn trie_free(trie: *mut Trie);
    fn trie_save(trie: *mut Trie, path: *const libc::c_char) -> libc::c_int;
    fn trie_get_serialized_size(trie: *const Trie) -> usize;
    fn trie_serialize(trie: *mut Trie, ptr: *mut u8);

}
const TRIE_FILENAME: &str = "test_serialization.tri";

#[test]
fn test_serialization() -> anyhow::Result<()> {
    msg_step("Preparing trie");
    let test_trie = unsafe { en_trie_new()? };

    /* add/remove some words */
    let dict_src = get_dict_src();
    unsafe {
        for dict_p in &dict_src {
            assert_eq!(
                trie_store(test_trie, dict_p.key.as_ptr(), dict_p.data),
                DA_TRUE,
                "Failed to add key '{:?}', data {}.\n",
                dict_p.key,
                dict_p.data
            );
        }

        /* save & close */
        msg_step("Saving trie to file");
        let _ = fs::remove_file(TRIE_FILENAME); /* error ignored */
        let trie_filename = CString::new(TRIE_FILENAME).unwrap();
        assert_eq!(
            trie_save(test_trie, trie_filename.as_ptr()),
            DA_OK,
            "Failed to save trie to file '{}'.\n",
            TRIE_FILENAME
        );

        msg_step("Getting serialized trie size");
        let size = trie_get_serialized_size(test_trie);
        println!("serialized trie size {}\n", size);
        msg_step("Allocating");
        let buf: Vec<u8> = Vec::with_capacity(size as usize);
        msg_step("Serializing");
        let buf_cap = buf.capacity();
        let mut buf = std::mem::ManuallyDrop::new(buf);
        let trie_serialized_data = buf.as_mut_ptr();
        trie_serialize(test_trie, trie_serialized_data);

        let buf = Vec::from_raw_parts(trie_serialized_data, size as usize, buf_cap);
        msg_step("Serialized");

        let mut f = fs::File::open(TRIE_FILENAME)?;

        let file_size = f.metadata().unwrap().len();

        assert_eq!(
            size, file_size as usize,
            "Trie serialized data doesn't match size of the file.\n"
        );

        let mut trie_file_data = Vec::new();
        assert_eq!(
            f.read_to_end(&mut trie_file_data)?,
            size as usize,
            "Failed to read back the serialized trie file.\n"
        );
        assert_eq!(
            buf, trie_file_data,
            "Trie serialized data doesn't match contents of the file.\n"
        );
        trie_free(test_trie);
    }
    Ok(())
}
#[test]
fn test_serialization_safe() -> anyhow::Result<()> {
    let dir = tempdir().unwrap();
    msg_step("Preparing trie");
    let test_trie = unsafe { en_trie_new()? };

    /* add/remove some words */
    let dict_src = get_dict_src();
    for dict_p in &dict_src {
        unsafe {
            assert_eq!(
                trie_store(test_trie, dict_p.key.as_ptr(), dict_p.data),
                DA_TRUE,
                "Failed to add key '{:?}', data {}.\n",
                dict_p.key,
                dict_p.data
            );
        }
    }

    /* save & close */
    msg_step("Saving trie to file");
    let trie_filename_orig = dir.path().join(TRIE_FILENAME);
    let trie_filename = CString::new(trie_filename_orig.to_str().unwrap()).unwrap();
    assert_eq!(
        unsafe { trie_save(test_trie, trie_filename.as_ptr()) },
        DA_OK,
        "Failed to save trie to file '{:?}'.\n",
        trie_filename
    );

    msg_step("Getting serialized trie size");
    let size = unsafe { trie_get_serialized_size(test_trie) };
    println!("serialized trie size {}\n", size);
    msg_step("Allocating");
    let trie_serialized_data: Vec<u8> = Vec::with_capacity(size);
    msg_step("Serializing");
    let mut buf = std::mem::ManuallyDrop::new(trie_serialized_data);
    let trie_serialized_data = buf.as_mut_ptr();
    let buf_cap = buf.capacity();
    unsafe { trie_serialize(test_trie, trie_serialized_data) };
    let trie_serialized_data =
        unsafe { Vec::from_raw_parts(trie_serialized_data, size as usize, buf_cap) };
    msg_step("Serialized");

    let mut f = fs::File::open(trie_filename_orig.as_path())?;

    let file_size = f.metadata().unwrap().len();

    assert_eq!(
        size as u64, file_size,
        "Trie serialized data doesn't match size of the file.\n"
    );

    let mut trie_file_data = Vec::new();
    assert_eq!(
        f.read_to_end(&mut trie_file_data)?,
        size as usize,
        "Failed to read back the serialized trie file.\n"
    );
    assert_eq!(
        trie_serialized_data, trie_file_data,
        "Trie serialized data doesn't match contents of the file.\n"
    );
    // }
    unsafe { trie_free(test_trie) };
    dir.close().unwrap();
    Ok(())
}
