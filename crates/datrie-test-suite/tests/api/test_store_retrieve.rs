// /* -*- Mode: C; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
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
//  * test_store-retrieve.c - Test for datrie store/retrieve operations
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
use rand::Rng;

use crate::utils::{
    dict_src_get_data, dict_src_set_data, en_trie_new, get_dict_src, msg_step, TRIE_DATA_ERROR,
    TRIE_DATA_READ,
};

extern "C" {

    fn trie_retrieve(trie: *const Trie, key: *const AlphaChar, o_data: *mut TrieData) -> Bool;
    fn trie_store(trie: *mut Trie, key: *const AlphaChar, data: TrieData) -> Bool;
    fn trie_free(trie: *mut Trie);

    fn trie_delete(trie: *mut Trie, key: *const AlphaChar) -> Bool;

    fn trie_root(trie: *const Trie) -> *mut TrieState;

    fn trie_state_free(s: *mut TrieState);

    fn trie_iterator_new(s: *mut TrieState) -> *mut TrieIterator;

    fn trie_iterator_free(iter: *mut TrieIterator);

    fn trie_iterator_next(iter: *mut TrieIterator) -> Bool;

    fn trie_iterator_get_key(iter: *const TrieIterator) -> *mut AlphaChar;

    fn trie_iterator_get_data(iter: *const TrieIterator) -> TrieData;

}

// int
// main (void)
#[test]
fn test_store_retrieve() {
    //     Trie         *test_trie;
    //     DictRec      *dict_p;
    //     TrieData      trie_data;
    //     Bool          is_failed;
    //     int           n_entries, n_dels, i;
    //     TrieState    *trie_root_state;
    //     TrieIterator *trie_it;
    unsafe {
        msg_step("Preparing trie");
        let test_trie = en_trie_new().expect("Fail to create test trie");
        //         goto err_trie_not_created;
        //     }

        /* store */
        msg_step("Adding data to trie");
        let mut dict_src = get_dict_src();
        for dict_p in &dict_src {
            //     for (dict_p = dict_src; dict_p->key; dict_p++) {
            if trie_store(test_trie, dict_p.key.as_ptr(), dict_p.data) != DA_TRUE {
                panic!(
                    "Failed to add key '{:?}', data {}.\n",
                    dict_p.key, dict_p.data
                );
                //             goto err_trie_created;
            }
        }

        /* retrieve */
        msg_step("Retrieving data from trie");
        //     is_failed = FALSE;
        for dict_p in &dict_src {
            let mut trie_data = 0;
            if trie_retrieve(test_trie, dict_p.key.as_ptr(), &mut trie_data) != DA_TRUE {
                panic!("Failed to retrieve key '{:?}'.\n", dict_p.key);
            }
            assert_eq!(
                trie_data, dict_p.data,
                "Wrong data for key '{:?}';",
                dict_p.key
            );
        }
        //     if (is_failed) {
        //         printf ("Trie store/retrieval test failed.\n");
        //         goto err_trie_created;
        //     }

        /* delete */
        msg_step("Deleting some entries from trie");
        let n_entries = dict_src.len();
        //     srand (time (NULL));
        let mut rng = rand::thread_rng();
        for _ in 0..(n_entries / 3 + 1) {
            //     for (n_dels = n_entries/3 + 1; n_dels > 0; n_dels--) {
            /* pick an undeleted entry */
            let mut i;
            loop {
                i = rng.gen_range(0..n_entries);
                // i = rand () % n_entries;
                //         } while (TRIE_DATA_READ == dict_src[i].data);
                if dict_src[i].data != TRIE_DATA_READ {
                    break;
                }
            }
            println!("Deleting '{:?}'", dict_src[i].key);
            if trie_delete(test_trie, dict_src[i].key.as_ptr()) != DA_TRUE {
                panic!("Failed to delete '{:?}'", dict_src[i].key);
                //             is_failed = TRUE;
            }
            dict_src[i].data = TRIE_DATA_READ;
        }
        //     if (is_failed) {
        //         printf ("Trie deletion test failed.\n");
        //         goto err_trie_created;
        //     }

        /* retrieve */
        msg_step("Retrieving data from trie again after deletions");
        //     for (dict_p = dict_src; dict_p->key; dict_p++) {
        for dict_p in &dict_src {
            /* skip deleted entries */
            if TRIE_DATA_READ == dict_p.data {
                continue;
            }

            let mut trie_data = 0;
            if trie_retrieve(test_trie, dict_p.key.as_ptr(), &mut trie_data) != DA_TRUE {
                panic!("Failed to retrieve key {:?}'.\n", dict_p.key);
                //             is_failed = TRUE;
            }
            assert_eq!(
                trie_data, dict_p.data,
                "Wrong data for key '{:?}';",
                dict_p.key
            );
            //             is_failed = TRUE;
            //         }
        }
        //     if (is_failed) {
        //         printf ("Trie retrival-after-deletion test failed.\n");
        //         goto err_trie_created;
        //     }

        /* enumerate & check */
        msg_step("Iterating trie contents after deletions");
        let trie_root_state = trie_root(test_trie);
        if trie_root_state.is_null() {
            panic!("Failed to get trie root state\n");
            //         goto err_trie_created;
        }
        let trie_it = trie_iterator_new(trie_root_state);
        if trie_it.is_null() {
            trie_state_free(trie_root_state);
            panic!("Failed to get trie iterator\n");
            //         goto err_trie_root_created;
        }

        while trie_iterator_next(trie_it) == DA_TRUE {
            //         AlphaChar *key;
            //         TrieData   key_data, src_data;

            let key = trie_iterator_get_key(trie_it);
            if key.is_null() {
                trie_iterator_free(trie_it);
                trie_state_free(trie_root_state);
                panic!("Failed to get key from trie iterator");
                //             is_failed = TRUE;
                //             continue;
            }
            let key_data = trie_iterator_get_data(trie_it);
            assert_ne!(
                TRIE_DATA_ERROR, key_data,
                "Failed to get data from trie iterator for key '{:?}'",
                key
            );
            //             is_failed = TRUE;
            //         }
            /* mark entries found in trie */
            let src_data = dict_src_get_data(&dict_src, key);
            assert_ne!(
                TRIE_DATA_ERROR, src_data,
                "Extra entry in trie: key '{:?}', data {}.\n",
                key, key_data
            );
            //             is_failed = TRUE;
            assert_eq!(src_data, key_data, "Data mismatch for: key '{:?}'", key);
            //             is_failed = TRUE;
            //         } else {
            dict_src_set_data(&mut dict_src, key, TRIE_DATA_READ);
            //         }

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
}
