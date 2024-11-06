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
//  * test_byte_alpha.c - Test byte stream (full-range 0..255) alpha map
//  * Created: 2018-04-21
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  *          Based on test case in issue #6
//  *          https://github.com/tlwg/libdatrie/issues/6
//  */
extern "C" {
    fn alpha_map_new() -> *mut AlphaMap;
    fn alpha_map_add_range(
        alpha_map: *mut AlphaMap,
        begin: AlphaChar,
        end: AlphaChar,
    ) -> libc::c_int;
    fn trie_new(alpha_map: *const AlphaMap) -> *mut Trie;
    fn trie_retrieve(trie: *const Trie, key: *const AlphaChar, o_data: *mut TrieData) -> Bool;
    fn trie_store(trie: *mut Trie, key: *const AlphaChar, data: TrieData) -> Bool;
}
use datrie_test_suite::{AlphaChar, AlphaMap, Bool, Trie, TrieData};

use crate::utils::msg_step;

const TEST_DATA: TrieData = 255;

#[test]
fn test_byte_alpha() -> anyhow::Result<()> {
    unsafe {
        msg_step("Preparing alpha map");
        let alpha_map = alpha_map_new();
        if alpha_map.is_null() {
            anyhow::bail!("failed to create alpha_map");
        }
        assert_eq!(
            alpha_map_add_range(alpha_map, 0x00, 0xff),
            0,
            "Fail to add full alpha map range\n"
        );

        msg_step("Preparing trie");

        let test_trie = trie_new(alpha_map);
        if test_trie.is_null() {
            anyhow::bail!("failed to create trie");
        }

        msg_step("Storing key to test trie");
        let key = &[0xff, 0xff, 0];
        assert_eq!(
            trie_store(test_trie, key.as_ptr(), TEST_DATA),
            1,
            "Fail to store key to test trie\n"
        );

        msg_step("Retrieving data from test trie");
        let mut data = 0;
        assert_eq!(
            trie_retrieve(test_trie, key.as_ptr(), &mut data),
            1,
            "Fail to retrieve key from test trie\n"
        );
        assert_eq!(
            TEST_DATA, data,
            "Retrieved data = {}, not {}\n",
            data, TEST_DATA
        );
    }
    Ok(())
}
