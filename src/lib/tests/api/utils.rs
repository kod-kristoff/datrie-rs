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
//  * utils.h - Utility functions for datrie test cases
//  * Created: 2013-10-16
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  */
// /*---------------------------*
//  *  Dict source for testing  *
//  *---------------------------*/
#[derive(Debug, Clone, Copy)]
pub struct DictRec {
    pub key: &'static [AlphaChar],
    pub data: TrieData,
}

pub const TRIE_DATA_ERROR: TrieData = -1;
pub const TRIE_DATA_UNREAD: TrieData = 1;
pub const TRIE_DATA_READ: TrieData = 2;

// extern DictRec dict_src[];

// int      dict_src_n_entries (void);
// TrieData dict_src_get_data (const AlphaChar *key);
// int      dict_src_set_data (const AlphaChar *key, TrieData data);

// /*
// vi:ts=4:ai:expandtab
// */
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
//  * utils.c - Utility functions for datrie test cases
//  * Created: 2013-10-16
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  */
// #include <datrie/trie.h>
// #include "utils.h"

use datrie::{
    alpha_map::{alpha_char_strcmp, AlphaChar, AlphaMap},
    trie::{Trie, TrieData},
    DatrieResult,
};

// /*---------------------*
//  *  Debugging helpers  *
//  *---------------------*/
pub fn msg_step(msg: &str) {
    println!("=> {}...", msg);
}

// /*-------------------------*
//  *  Trie creation helpers  *
//  *-------------------------*/
pub unsafe fn en_alpha_map_new() -> DatrieResult<AlphaMap> {
    let mut en_map = AlphaMap::new();
    en_map.add_range(0x0061, 0x007a)?;

    return Ok(en_map);
}

pub unsafe fn en_trie_new() -> DatrieResult<Trie> {
    let en_map = en_alpha_map_new()?;

    let en_trie = Trie::new(&en_map)?;
    dbg!(&en_trie);
    Ok(en_trie)
}

// /*---------------------------*
//  *  Dict source for testing  *
//  *---------------------------*/
pub fn get_dict_src() -> [DictRec; 39] {
    [
        DictRec {
            key: &['a' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'b' as AlphaChar,
                'a' as AlphaChar,
                'c' as AlphaChar,
                'u' as AlphaChar,
                's' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'b' as AlphaChar,
                'a' as AlphaChar,
                'n' as AlphaChar,
                'd' as AlphaChar,
                'o' as AlphaChar,
                'n' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'c' as AlphaChar,
                'c' as AlphaChar,
                'i' as AlphaChar,
                'd' as AlphaChar,
                'e' as AlphaChar,
                'n' as AlphaChar,
                't' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'c' as AlphaChar,
                'c' as AlphaChar,
                'r' as AlphaChar,
                'e' as AlphaChar,
                'd' as AlphaChar,
                'i' as AlphaChar,
                't' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'l' as AlphaChar,
                'g' as AlphaChar,
                'o' as AlphaChar,
                'r' as AlphaChar,
                'i' as AlphaChar,
                't' as AlphaChar,
                'h' as AlphaChar,
                'm' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'm' as AlphaChar,
                'm' as AlphaChar,
                'o' as AlphaChar,
                'n' as AlphaChar,
                'i' as AlphaChar,
                'a' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'n' as AlphaChar,
                'g' as AlphaChar,
                'e' as AlphaChar,
                'l' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'n' as AlphaChar,
                'g' as AlphaChar,
                'l' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'a' as AlphaChar,
                'z' as AlphaChar,
                'u' as AlphaChar,
                'r' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &['b' as AlphaChar, 'a' as AlphaChar, 't' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &['b' as AlphaChar, 'e' as AlphaChar, 't' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'b' as AlphaChar,
                'e' as AlphaChar,
                's' as AlphaChar,
                't' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'h' as AlphaChar,
                'o' as AlphaChar,
                'm' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'h' as AlphaChar,
                'o' as AlphaChar,
                'u' as AlphaChar,
                's' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &['h' as AlphaChar, 'u' as AlphaChar, 't' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'k' as AlphaChar,
                'i' as AlphaChar,
                'n' as AlphaChar,
                'g' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'k' as AlphaChar,
                'i' as AlphaChar,
                't' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'n' as AlphaChar,
                'a' as AlphaChar,
                'm' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &['n' as AlphaChar, 'e' as AlphaChar, 't' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'n' as AlphaChar,
                'e' as AlphaChar,
                't' as AlphaChar,
                'w' as AlphaChar,
                'o' as AlphaChar,
                'r' as AlphaChar,
                'k' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &['n' as AlphaChar, 'u' as AlphaChar, 't' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'n' as AlphaChar,
                'u' as AlphaChar,
                't' as AlphaChar,
                's' as AlphaChar,
                'h' as AlphaChar,
                'e' as AlphaChar,
                'l' as AlphaChar,
                'l' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'q' as AlphaChar,
                'u' as AlphaChar,
                'a' as AlphaChar,
                'l' as AlphaChar,
                'i' as AlphaChar,
                't' as AlphaChar,
                'y' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'q' as AlphaChar,
                'u' as AlphaChar,
                'a' as AlphaChar,
                'n' as AlphaChar,
                't' as AlphaChar,
                'u' as AlphaChar,
                'm' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'q' as AlphaChar,
                'u' as AlphaChar,
                'a' as AlphaChar,
                'n' as AlphaChar,
                't' as AlphaChar,
                'i' as AlphaChar,
                't' as AlphaChar,
                'y' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'q' as AlphaChar,
                'u' as AlphaChar,
                'a' as AlphaChar,
                'r' as AlphaChar,
                't' as AlphaChar,
                'z' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'q' as AlphaChar,
                'u' as AlphaChar,
                'i' as AlphaChar,
                'c' as AlphaChar,
                'k' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'q' as AlphaChar,
                'u' as AlphaChar,
                'i' as AlphaChar,
                'z' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &['r' as AlphaChar, 'u' as AlphaChar, 'n' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                't' as AlphaChar,
                'a' as AlphaChar,
                'p' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                't' as AlphaChar,
                'e' as AlphaChar,
                's' as AlphaChar,
                't' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'w' as AlphaChar,
                'h' as AlphaChar,
                'a' as AlphaChar,
                't' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'w' as AlphaChar,
                'h' as AlphaChar,
                'e' as AlphaChar,
                'n' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'w' as AlphaChar,
                'h' as AlphaChar,
                'e' as AlphaChar,
                'r' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'w' as AlphaChar,
                'h' as AlphaChar,
                'i' as AlphaChar,
                'c' as AlphaChar,
                'h' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &['w' as AlphaChar, 'h' as AlphaChar, 'o' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &['w' as AlphaChar, 'h' as AlphaChar, 'y' as AlphaChar, 0x0000],
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &[
                'z' as AlphaChar,
                'e' as AlphaChar,
                'b' as AlphaChar,
                'r' as AlphaChar,
                'a' as AlphaChar,
                0x0000,
            ],
            data: TRIE_DATA_UNREAD,
        },
        //     {(AlphaChar *)NULL,          TRIE_DATA_ERROR},
    ]
}

// int
// dict_src_n_entries (void)
// {
//     return sizeof (dict_src) / sizeof (dict_src[0]) - 1;
// }

// TrieData
pub unsafe fn dict_src_get_data(dict_src: &[DictRec], key: *const AlphaChar) -> TrieData {
    //     const DictRec *dict_p;

    //     for (dict_p = dict_src; dict_p->key; dict_p++) {
    for dict_p in dict_src {
        if alpha_char_strcmp(dict_p.key.as_ptr(), key) == 0 {
            return dict_p.data;
        }
    }

    return TRIE_DATA_ERROR;
}

// int
pub unsafe fn dict_src_set_data(
    dict_src: &mut [DictRec],
    key: *const AlphaChar,
    data: TrieData,
) -> bool {
    //     DictRec *dict_p;

    //     for (dict_p = dict_src; dict_p->key; dict_p++) {
    for dict_p in dict_src {
        if alpha_char_strcmp(dict_p.key.as_ptr(), key) == 0 {
            dict_p.data = data;
            return true;
        }
    }

    //     return -1;
    return false;
}

// /*
// vi:ts=4:ai:expandtab
// */
