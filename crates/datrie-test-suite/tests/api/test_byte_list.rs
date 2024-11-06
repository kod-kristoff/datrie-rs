// /*
//  * libdatrie - Double-Array Trie Library
//  * Copyright (C) 2018  Theppitak Karoonboonyanan <theppitak@gmail.com>
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
//  * test_byte_list.c - Test byte trie enumeration
//  * Created: 2018-11-20
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  *          Based on test case in issue #9
//  *          https://github.com/tlwg/libdatrie/issues/9
//  */
extern "C" {
    // fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    // fn fclose(__stream: *mut FILE) -> libc::c_int;
    // fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
}
extern "C" {
    fn alpha_char_strcmp(str1: *const AlphaChar, str2: *const AlphaChar) -> libc::c_int;
    fn alpha_map_new() -> *mut AlphaMap;
    fn alpha_map_add_range(
        alpha_map: *mut AlphaMap,
        begin: AlphaChar,
        end: AlphaChar,
    ) -> libc::c_int;
    fn trie_new(alpha_map: *const AlphaMap) -> *mut Trie;
    fn trie_store(trie: *mut Trie, key: *const AlphaChar, data: TrieData) -> Bool;
    fn trie_root(trie: *const Trie) -> *mut TrieState;
    fn trie_state_free(s: *mut TrieState);
    fn trie_iterator_new(s: *mut TrieState) -> *mut TrieIterator;

    fn trie_iterator_free(iter: *mut TrieIterator);

    fn trie_iterator_next(iter: *mut TrieIterator) -> Bool;

    fn trie_iterator_get_key(iter: *const TrieIterator) -> *mut AlphaChar;

    fn trie_iterator_get_data(iter: *const TrieIterator) -> TrieData;
}

use datrie_test_suite::{
    AlphaChar, AlphaMap, Bool, Trie, TrieData, TrieIterator, TrieState, DA_FALSE,
};

#[derive(Debug, Clone, Copy)]
struct DictEntry {
    key: [AlphaChar; 4],
    data: TrieData,
    is_checked: bool,
}

// /* Dictionary source */
const SOURCE: [DictEntry; 2] = [
    DictEntry {
        key: ['1' as AlphaChar, '2' as AlphaChar, 0, 0],
        data: 1,
        is_checked: false,
    },
    DictEntry {
        key: ['1' as AlphaChar, '2' as AlphaChar, '3' as AlphaChar, 0],
        data: 2,
        is_checked: false,
    },
];

unsafe fn dump_key_data(key: *const AlphaChar, data: TrieData) {
    print!("[");
    let mut p = key;
    while *p != 0 {
        if p != key {
            print!(", ");
        }
        print!("{}", *p);
        p = p.offset(1);
    }
    println!("] : {}", data);
}

unsafe fn dump_entry(iter: *const TrieIterator) {
    let key = trie_iterator_get_key(iter);
    dump_key_data(key, trie_iterator_get_data(iter));
    free(key as *mut libc::c_void);
}

/*
 * Check if the trie entry referenced by iter match any Source[] element
 * and mark the matched element as checked.
 * Return: 1 if matched, 0 otherwise
 */
unsafe fn validate_entry(source: &mut [DictEntry], iter: *const TrieIterator) -> bool {
    let key = trie_iterator_get_key(iter);
    let data = trie_iterator_get_data(iter);

    for dict_p in source {
        if alpha_char_strcmp(dict_p.key.as_ptr(), key) == 0 && dict_p.data == data {
            dict_p.is_checked = true;
            free(key as *mut libc::c_void);
            return true;
        }
    }
    free(key as *mut libc::c_void);
    false
}

/*
 * Check if all Source[] elements are checked and reported unchecked one.
 * Return: 1 if all are checked, 0 otherwise.
 */
fn is_all_checked(source: &[DictEntry]) -> bool {
    let mut ret = true;
    for dict_p in source {
        if !dict_p.is_checked {
            print!("Not visited Source entry: ");
            unsafe {
                dump_key_data(dict_p.key.as_ptr(), dict_p.data);
            }
            ret = false;
        }
    }

    ret
}

use crate::utils::msg_step;

#[test]
fn test_byte_list() -> anyhow::Result<()> {
    unsafe {
        msg_step("Preparing alpha map");
        let alpha_map = alpha_map_new();
        assert_eq!(
            alpha_map_add_range(alpha_map, 0x00, 0xff),
            0,
            "Fail to add full alpha map range\n"
        );

        msg_step("Preparing trie");
        let test_trie = trie_new(alpha_map);
        if test_trie.is_null() {
            anyhow::bail!("failed to create test_trie");
        }

        msg_step("Storing entries to test trie");
        let mut source = SOURCE;
        for dict_p in &source {
            if trie_store(test_trie, dict_p.key.as_ptr(), dict_p.data) == 0 {
                panic!(
                    "Fail to store entry to test trie: {:?}->{}",
                    dict_p.key, dict_p.data
                );
            }
        }

        msg_step("Iterating trie");
        let root = trie_root(test_trie);
        let iter = trie_iterator_new(root);
        assert!(!iter.is_null());
        while trie_iterator_next(iter) != DA_FALSE {
            if !validate_entry(&mut source, iter) {
                println!("Fail to validate trie entry:");
                dump_entry(iter);
            }
        }
        assert!(is_all_checked(&source));
        trie_iterator_free(iter);
        trie_state_free(root);
    }
    Ok(())
}
