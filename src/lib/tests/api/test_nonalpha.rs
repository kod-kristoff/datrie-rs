// /*
//  * libdatrie - Double-Array Trie Library
//  * Copyright (C) 2014  Theppitak Karoonboonyanan <theppitak@gmail.com>
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
//  * test_nonalpha.c - Test for datrie behaviors on non-alphabet inputs
//  * Created: 2014-01-06
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  */
use datrie::{
    AlphaChar, AlphaStr, DatrieResult, {Trie, DA_TRUE},
};

use crate::utils::{en_trie_new, get_dict_src, msg_step, TRIE_DATA_UNREAD};

#[test]
fn test_nonalpha() -> DatrieResult<()> {
    unsafe {
        msg_step("Preparing trie");
        let mut test_trie = en_trie_new()?;

        /* store */
        msg_step("Adding data to trie");
        let dict_src = get_dict_src();
        for dict_p in &dict_src {
            assert!(
                Trie::store(&mut test_trie, dict_p.key, dict_p.data),
                "Failed to add key '{:?}', data {}.\n",
                dict_p.key,
                dict_p.data
            );
        }

        //     /* test storing keys with non-alphabet chars */
        let nonalpha_src = [
            &AlphaStr::from_slice_with_nul(&[
                'a' as AlphaChar,
                '6' as AlphaChar,
                'a' as AlphaChar,
                'c' as AlphaChar,
                'u' as AlphaChar,
                's' as AlphaChar,
                0x0000,
            ])
            .unwrap(),
            &AlphaStr::from_slice_with_nul(&[
                'a' as AlphaChar,
                '5' as AlphaChar,
                'a' as AlphaChar,
                'c' as AlphaChar,
                'u' as AlphaChar,
                's' as AlphaChar,
                0x0000,
            ])
            .unwrap(),
        ];

        let mut trie_data = 0;
        for nonalpha_key in nonalpha_src {
            assert!(
                !Trie::retrieve(&test_trie, nonalpha_key, &mut trie_data),
                "False duplication on key '{:?}', with existing data {}.\n",
                nonalpha_key,
                trie_data
            );
            assert!(
                !Trie::store(&mut test_trie, nonalpha_key, TRIE_DATA_UNREAD),
                "Wrongly added key '{:?}' containing non-alphanet char\n",
                nonalpha_key
            );
        }
    }
    Ok(())
}
