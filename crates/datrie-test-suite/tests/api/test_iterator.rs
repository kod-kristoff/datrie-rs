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
use datrie_test_suite::{AlphaChar, Bool, Trie, TrieData, TrieIterator, TrieState, DA_TRUE};

use crate::utils::{
    dict_src_get_data, dict_src_set_data, en_trie_new, get_dict_src, msg_step, TRIE_DATA_ERROR,
    TRIE_DATA_READ,
};

extern "C" {
    fn trie_store(trie: *mut Trie, key: *const AlphaChar, data: TrieData) -> Bool;
    fn trie_free(trie: *mut Trie);

    fn trie_root(trie: *const Trie) -> *mut TrieState;

    fn trie_state_free(s: *mut TrieState);

    fn trie_iterator_new(s: *mut TrieState) -> *mut TrieIterator;

    fn trie_iterator_free(iter: *mut TrieIterator);

    fn trie_iterator_next(iter: *mut TrieIterator) -> Bool;

    fn trie_iterator_get_key(iter: *const TrieIterator) -> *mut AlphaChar;

    fn trie_iterator_get_data(iter: *const TrieIterator) -> TrieData;

}

#[test]
fn test_iterator() -> anyhow::Result<()> {
    unsafe {
        msg_step("Preparing trie");
        let test_trie = en_trie_new()?;

        /* store */
        msg_step("Adding data to trie");
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

        /* iterate & check */
        msg_step("Iterating and checking trie contents");
        let trie_root_state = trie_root(test_trie);
        if trie_root_state.is_null() {
            panic!("Failed to get trie root state\n");
        }
        let trie_it = trie_iterator_new(trie_root_state);
        if trie_it.is_null() {
            trie_state_free(trie_root_state);
            anyhow::bail!("Failed to get trie iterator\n");
        }

        while trie_iterator_next(trie_it) == DA_TRUE {
            let key = trie_iterator_get_key(trie_it);
            if key.is_null() {
                trie_iterator_free(trie_it);
                trie_state_free(trie_root_state);
                panic!("Failed to get key from trie iterator");
            }
            let key_data = trie_iterator_get_data(trie_it);
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

        trie_iterator_free(trie_it);
        trie_state_free(trie_root_state);
        trie_free(test_trie);
    }
    Ok(())
}
