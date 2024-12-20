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
//  * test_file.c - Test for datrie file operations
//  * Created: 2013-10-16
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  */
use datrie_test_suite::{AlphaChar, Bool, Trie, TrieData, TrieEnumFunc, DA_OK, DA_TRUE};
use std::{ffi::CString, fs, os::unix::ffi::OsStrExt};
use tempfile::tempdir;

use crate::utils::{
    dict_src_get_data, dict_src_set_data, en_trie_new, get_dict_src, msg_step, DictRec,
    TRIE_DATA_ERROR, TRIE_DATA_READ,
};

extern "C" {
    fn trie_new_from_file(path: *const libc::c_char) -> *mut Trie;
    fn trie_store(trie: *mut Trie, key: *const AlphaChar, data: TrieData) -> Bool;
    fn trie_save(trie: *mut Trie, path: *const libc::c_char) -> libc::c_int;
    fn trie_enumerate(
        trie: *const Trie,
        enum_func: TrieEnumFunc,
        user_data: *mut libc::c_void,
    ) -> Bool;
}

const TRIE_FILENAME: &str = "test_file.tri";

struct EnumData<'a> {
    dict_src: &'a mut [DictRec],
    is_failed: &'a mut bool,
}
// static
extern "C" fn trie_enum_mark_rec(
    key: *const AlphaChar,
    key_data: TrieData,
    user_data: *mut libc::c_void,
) -> Bool {
    // Bool *is_failed = (Bool *)user_data;
    // TrieData src_data;
    let enum_data = user_data as *mut EnumData;

    let src_data = unsafe { dict_src_get_data((*enum_data).dict_src, key) };
    if TRIE_DATA_ERROR == src_data {
        println!("Extra entry in file: key '{:?}', data {}.\n", key, key_data);
        unsafe {
            *(*enum_data).is_failed = true;
        }
    } else if src_data != key_data {
        println!(
            "Data mismatch for: key '{:?}', expected {}, got {}.\n",
            key, src_data, key_data
        );
        unsafe {
            *(*enum_data).is_failed = true;
        }
    } else {
        unsafe {
            dict_src_set_data((*enum_data).dict_src, key, TRIE_DATA_READ);
        }
    }

    DA_TRUE
}

#[test]
fn test_file() -> anyhow::Result<()> {
    unsafe {
        msg_step("Preparing trie");
        let test_trie = en_trie_new()?;

        /* add/remove some words */
        let mut dict_src = get_dict_src();
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

        /* reload from file */
        msg_step("Reloading trie from the saved file");
        let test_trie = trie_new_from_file(trie_filename.as_ptr());
        if test_trie.is_null() {
            anyhow::bail!("failed to load trie");
        }

        /* enumerate & check */
        msg_step("Checking trie contents");
        let mut is_failed = false;
        let mut enum_data = EnumData {
            dict_src: &mut dict_src,
            is_failed: &mut is_failed,
        };
        /* mark entries found in file */
        if trie_enumerate(
            test_trie,
            Some(trie_enum_mark_rec),
            &mut enum_data as *mut EnumData as *mut libc::c_void,
        ) != DA_TRUE
        {
            panic!("Failed to enumerate trie file contents.\n");
            //         goto err_trie_saved;
        }
        /* check for unmarked entries, (i.e. missed in file) */
        for dict_p in dict_src {
            if dict_p.data != TRIE_DATA_READ {
                println!(
                    "Entry missed in file: key '{:?}', data {}.\n",
                    dict_p.key, dict_p.data
                );
                is_failed = true;
            }
        }
        if is_failed {
            panic!("Errors found in trie saved contents.\n");
        }

        //     remove (TRIE_FILENAME);
    }
    Ok(())
}

#[test]
fn test_save_file_and_reload() -> anyhow::Result<()> {
    let dir = tempdir().unwrap();

    msg_step("Preparing trie");
    let test_trie = unsafe { en_trie_new()? };

    /* add/remove some words */
    let mut dict_src = get_dict_src();
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
    let trie_filename = dir.path().join(TRIE_FILENAME);
    // let _ = fs::remove_file(TRIE_FILENAME); /* error ignored */
    // let trie_filename = CString::new(TRIE_FILENAME).unwrap();
    let trie_filename = CString::new(trie_filename.as_os_str().as_bytes()).unwrap();

    assert_eq!(
        unsafe { trie_save(test_trie, trie_filename.as_ptr()) },
        DA_OK,
        "Failed to save trie to file '{}'.\n",
        TRIE_FILENAME
    );

    /* reload from file */
    msg_step("Reloading trie from the saved file");
    let test_trie = unsafe { trie_new_from_file(trie_filename.as_ptr()) };
    if test_trie.is_null() {
        anyhow::bail!("failed to load trie");
    }
    /* enumerate & check */
    msg_step("Checking trie contents");
    let mut is_failed = false;
    let mut enum_data = EnumData {
        dict_src: &mut dict_src,
        is_failed: &mut is_failed,
    };
    /* mark entries found in file */
    unsafe {
        if trie_enumerate(
            test_trie,
            Some(trie_enum_mark_rec),
            &mut enum_data as *mut EnumData as *mut libc::c_void,
        ) != DA_TRUE
        {
            panic!("Failed to enumerate trie file contents.\n");
            //         goto err_trie_saved;
        }
        /* check for unmarked entries, (i.e. missed in file) */
        for dict_p in dict_src {
            if dict_p.data != TRIE_DATA_READ {
                println!(
                    "Entry missed in file: key '{:?}', data {}.\n",
                    dict_p.key, dict_p.data
                );
                is_failed = true;
            }
        }
        if is_failed {
            panic!("Errors found in trie saved contents.\n");
        }

        //     remove (TRIE_FILENAME);
    }
    dir.close().unwrap();
    Ok(())
}
