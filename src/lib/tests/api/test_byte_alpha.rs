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
use datrie::{
    AlphaMap, AlphaStr, DatrieResult, {Trie, TrieData, DA_TRUE},
};

use crate::utils::msg_step;

const TEST_DATA: TrieData = 255;

#[test]
fn test_byte_alpha() -> DatrieResult<()> {
    unsafe {
        msg_step("Preparing alpha map");
        let mut alpha_map = AlphaMap::default();
        assert!(
            alpha_map.add_range(0x00, 0xff).is_ok(),
            "Fail to add full alpha map range\n"
        );

        msg_step("Preparing trie");
        let mut test_trie = Trie::new(&alpha_map)?;

        msg_step("Storing key to test trie");
        let key = AlphaStr::from_slice_with_nul(&[0xff, 0xff, 0]).unwrap();
        assert!(
            Trie::store(&mut test_trie, key, TEST_DATA),
            "Fail to store key to test trie\n"
        );

        msg_step("Retrieving data from test trie");
        let mut data = 0;
        assert!(
            Trie::retrieve(&test_trie, key, &mut data),
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
