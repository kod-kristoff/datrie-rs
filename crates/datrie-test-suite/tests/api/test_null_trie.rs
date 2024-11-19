use datrie_test_suite::{AlphaChar, Bool, Trie, TrieIterator, TrieState};

/*
 * libdatrie - Double-Array Trie Library
 * Copyright (C) 2015  Theppitak Karoonboonyanan <theppitak@gmail.com>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA
 */
/*
 * test_null_trie.c - Test for datrie iteration on empty trie
 * Created: 2015-04-21
 * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
 * Ported
 */
use crate::utils::{en_trie_new, msg_step};

extern "C" {

    fn trie_free(trie: *mut Trie);

    fn trie_root(trie: *const Trie) -> *mut TrieState;

    fn trie_state_free(s: *mut TrieState);

    fn trie_iterator_new(s: *mut TrieState) -> *mut TrieIterator;

    fn trie_iterator_free(iter: *mut TrieIterator);

    fn trie_iterator_next(iter: *mut TrieIterator) -> Bool;

    fn trie_iterator_get_key(iter: *const TrieIterator) -> *mut AlphaChar;

}

#[test]
fn test_null_trie() {
    unsafe {
        msg_step("Preparing empty trie");
        let test_trie = en_trie_new().expect("Fail to create test trie");

        /* iterate & check */
        msg_step("Iterating");
        let trie_root_state = trie_root(test_trie);
        // dbg!(&trie_root_state as *const TrieState);
        dbg!(trie_root_state);
        // dbg!(&trie_root_state.trie);
        // let trie: &Trie = &*trie_root_state.trie;
        // dbg!(&*trie_root_state.trie);
        if trie_root_state.is_null() {
            panic!("Failed to get trie root state\n");
        }
        let trie_it = trie_iterator_new(trie_root_state);
        if trie_it.is_null() {
            trie_state_free(trie_root_state);
            panic!("Failed to get trie iterator");
        }

        // // dbg!(*trie_it);
        let mut is_failed = false;
        while trie_iterator_next(trie_it) == 1 {
            println!("Got entry from empty trie, which is weird!\n");

            let key = trie_iterator_get_key(trie_it);
            if !key.is_null() {
                println!(
                    "Got key from empty trie, which is weird! (key='{}')\n",
                    *key
                );
                is_failed = true;
                libc::free(key as *mut libc::c_void);
            }
        }

        if is_failed {
            trie_iterator_free(trie_it);
            panic!("Errors found in empty trie iteration.\n");
        }

        trie_iterator_free(trie_it);
        trie_free(test_trie);
    }
}
