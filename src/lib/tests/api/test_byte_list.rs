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
    let key = TrieIterator::get_key(iter);
    dump_key_data(key, TrieIterator::get_data(iter));
    free(key as *mut libc::c_void);
}

/*
 * Check if the trie entry referenced by iter match any Source[] element
 * and mark the matched element as checked.
 * Return: 1 if matched, 0 otherwise
 */
unsafe fn validate_entry(source: &mut [DictEntry], iter: *const TrieIterator) -> bool {
    let key = TrieIterator::get_key(iter);
    let data = TrieIterator::get_data(iter);

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

use datrie::{
    alpha_map::{alpha_char_strcmp, AlphaChar, AlphaMap},
    trie::{Trie, TrieData, TrieIterator, TrieState, DA_FALSE},
    DatrieResult,
};

use crate::utils::msg_step;

#[test]
fn test_byte_list() -> DatrieResult<()> {
    unsafe {
        msg_step("Preparing alpha map");
        let mut alpha_map = AlphaMap::default();
        assert!(
            alpha_map.add_range(0x00, 0xff).is_ok(),
            "Fail to add full alpha map range\n"
        );

        msg_step("Preparing trie");
        let mut test_trie = Trie::new(&alpha_map)?;

        msg_step("Storing entries to test trie");
        let mut source = SOURCE;
        for dict_p in &source {
            if Trie::store(&mut test_trie, dict_p.key.as_ptr(), dict_p.data) == 0 {
                panic!(
                    "Fail to store entry to test trie: {:?}->{}",
                    dict_p.key, dict_p.data
                );
            }
        }

        msg_step("Iterating trie");
        let root = Trie::root(&test_trie);
        let iter = TrieIterator::new(root);
        assert!(!iter.is_null());
        while TrieIterator::next(iter) != DA_FALSE {
            if !validate_entry(&mut source, iter) {
                println!("Fail to validate trie entry:");
                dump_entry(iter);
            }
        }
        assert!(is_all_checked(&source));
        TrieIterator::free(iter);
        TrieState::free(root);
    }
    Ok(())
}
