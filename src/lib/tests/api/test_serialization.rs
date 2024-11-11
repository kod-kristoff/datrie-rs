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

use datrie::{
    trie::{Trie, DA_TRUE},
    DatrieResult,
};
use tempfile::tempdir;

use crate::utils::{en_trie_new, get_dict_src, msg_step};

const TRIE_FILENAME: &str = "test_serialization.tri";

#[test]
fn test_serialization() -> DatrieResult<()> {
    msg_step("Preparing trie");
    let mut test_trie = en_trie_new()?;

    /* add/remove some words */
    let dict_src = get_dict_src();
    unsafe {
        for dict_p in &dict_src {
            assert_eq!(
                Trie::store(&mut test_trie, dict_p.key, dict_p.data),
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
            test_trie.save(trie_filename.as_ref()),
            Ok(()),
            "Failed to save trie to file '{}'.\n",
            TRIE_FILENAME
        );

        msg_step("Getting serialized trie size");
        let size = test_trie.get_serialized_size();
        println!("serialized trie size {}\n", size);
        msg_step("Allocating");
        let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
        msg_step("Serializing");
        // let mut buf = std::mem::ManuallyDrop::new(buf);
        test_trie.serialize_safe(&mut buf).unwrap();

        //     Vec::from_raw_parts(trie_serialized_data, size as usize, buf_cap);
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
    }
    Ok(())
}
#[test]
fn test_serialization_safe() -> DatrieResult<()> {
    let dir = tempdir().unwrap();
    msg_step("Preparing trie");
    let mut test_trie = en_trie_new()?;

    /* add/remove some words */
    let dict_src = get_dict_src();
    for dict_p in &dict_src {
        unsafe {
            assert_eq!(
                Trie::store(&mut test_trie, dict_p.key, dict_p.data),
                DA_TRUE,
                "Failed to add key '{:?}', data {}.\n",
                dict_p.key,
                dict_p.data
            );
        }
    }

    /* save & close */
    msg_step("Saving trie to file");
    let trie_filename = dir.path().join(TRIE_FILENAME);
    assert!(
        test_trie.save_safe(&trie_filename).is_ok(),
        "Failed to save trie to file '{}'.\n",
        trie_filename.display()
    );

    msg_step("Getting serialized trie size");
    let size = test_trie.get_serialized_size();
    println!("serialized trie size {}\n", size);
    msg_step("Allocating");
    let mut trie_serialized_data: Vec<u8> = Vec::with_capacity(size);
    msg_step("Serializing");
    // let mut buf = std::mem::ManuallyDrop::new(buf);
    // let trie_serialized_data = buf.as_mut_ptr();
    // let buf_cap = buf.capacity();
    test_trie.serialize_safe(&mut trie_serialized_data)?;
    // let trie_serialized_data = Vec::from_raw_parts(trie_serialized_data, size as usize, buf_cap);
    msg_step("Serialized");

    let mut f = fs::File::open(trie_filename)?;

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
    dir.close().unwrap();
    Ok(())
}
