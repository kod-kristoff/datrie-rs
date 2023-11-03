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
//  * test_iterator.c - Test for datrie iterator operations
//  * Created: 2013-10-16
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  */
extern "C" {
    // fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    // fn fclose(__stream: *mut FILE) -> libc::c_int;
    // fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
}
use datrie::{
    trie::{Trie, TrieIterator, TrieState, DA_TRUE},
    DatrieResult,
};

use crate::utils::{
    dict_src_get_data, dict_src_set_data, en_trie_new, get_dict_src, msg_step, TRIE_DATA_ERROR,
    TRIE_DATA_READ,
};

#[test]
fn test_iterator() -> DatrieResult<()> {
    unsafe {
        msg_step("Preparing trie");
        let mut test_trie = en_trie_new()?;

        /* store */
        msg_step("Adding data to trie");
        let mut dict_src = get_dict_src();
        for dict_p in &dict_src {
            assert_eq!(
                Trie::store(&mut test_trie, dict_p.key.as_ptr(), dict_p.data),
                DA_TRUE,
                "Failed to add key '{:?}', data {}.\n",
                dict_p.key,
                dict_p.data
            );
        }

        /* iterate & check */
        msg_step("Iterating and checking trie contents");
        let trie_root_state = Trie::root(&test_trie);
        if trie_root_state.is_null() {
            panic!("Failed to get trie root state\n");
        }
        let trie_it = TrieIterator::new(trie_root_state);
        if trie_it.is_null() {
            TrieState::free(trie_root_state);
            panic!("Failed to get trie iterator\n");
        }

        while TrieIterator::next(trie_it) == DA_TRUE {
            let key = TrieIterator::get_key(trie_it);
            if key.is_null() {
                TrieIterator::free(trie_it);
                TrieState::free(trie_root_state);
                panic!("Failed to get key from trie iterator");
            }
            let key_data = TrieIterator::get_data(trie_it);
            assert_ne!(
                TRIE_DATA_ERROR, key_data,
                "Failed to get data from trie iterator for key '{:?}'",
                key
            );
            /* mark entries found in trie */
            let src_data = dict_src_get_data(&dict_src, key);
            assert_ne!(
                TRIE_DATA_ERROR, src_data,
                "Extra entry in trie: key '{:?}', data {}.\n",
                key, key_data
            );
            assert_eq!(src_data, key_data, "Data mismatch for: key '{:?}'", key);
            dict_src_set_data(&mut dict_src, key, TRIE_DATA_READ);

            free(key as *mut libc::c_void);
        }

        /* check for unmarked entries, (i.e. missed in trie) */
        for dict_p in &dict_src {
            assert_eq!(
                dict_p.data, TRIE_DATA_READ,
                "Entry missed in trie: key '{:?}'",
                dict_p.key
            );
        }

        TrieIterator::free(trie_it);
        TrieState::free(trie_root_state);
    }
    Ok(())
}
