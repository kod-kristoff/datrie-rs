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
//  * test_walk.c - Test for datrie walking operations
//  * Created: 2013-10-16
//  * Author:  Theppitak Karoonboonyanan <theppitak@gmail.com>
//  */
// #include <datrie/trie.h>
// #include "utils.h"
// #include <stdio.h>
// #include <wchar.h>

use crate::utils::{DictRec, TRIE_DATA_ERROR, TRIE_DATA_UNREAD};

// /*
//  * Sample trie in http://linux.thai.net/~thep/datrie/datrie.html
//  *
//  *           +---o-> (3) -o-> (4) -l-> [5]
//  *           |
//  *           |        +---i-> (7) -z-> (8) -e-> [9]
//  *           |        |
//  * (1) -p-> (2) -r-> (6) -e-> (10) -v-> (11) -i-> (12) -e-> (13) -w-> [14]
//  *                    |         |
//  *                    |         +---p-> (15) -a-> (16) -r-> (17) -e-> [18]
//  *                    |
//  *                    +---o-> (19) -d-> (20) -u-> (21) -c-> (22) -e-> [23]
//  *                              |
//  *                              +---g-> (24) -r-> (25) -e-> (26) -s-> (27) -s-> [28]
//  *
//  */
fn get_walk_dict() -> [DictRec; 6] {
    [
        DictRec {
            key: &AlphaStr::from_slice_with_nul(&[
                'p' as AlphaChar,
                'o' as AlphaChar,
                'o' as AlphaChar,
                'l' as AlphaChar,
                0x0000,
            ])
            .unwrap(),
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &AlphaStr::from_slice_with_nul(&[
                'p' as AlphaChar,
                'r' as AlphaChar,
                'i' as AlphaChar,
                'z' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ])
            .unwrap(),

            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &AlphaStr::from_slice_with_nul(&[
                'p' as AlphaChar,
                'r' as AlphaChar,
                'e' as AlphaChar,
                'v' as AlphaChar,
                'i' as AlphaChar,
                'e' as AlphaChar,
                'w' as AlphaChar,
                0x0000,
            ])
            .unwrap(),
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &AlphaStr::from_slice_with_nul(&[
                'p' as AlphaChar,
                'r' as AlphaChar,
                'e' as AlphaChar,
                'p' as AlphaChar,
                'a' as AlphaChar,
                'r' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ])
            .unwrap(),
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &AlphaStr::from_slice_with_nul(&[
                'p' as AlphaChar,
                'r' as AlphaChar,
                'o' as AlphaChar,
                'd' as AlphaChar,
                'u' as AlphaChar,
                'c' as AlphaChar,
                'e' as AlphaChar,
                0x0000,
            ])
            .unwrap(),
            data: TRIE_DATA_UNREAD,
        },
        DictRec {
            key: &AlphaStr::from_slice_with_nul(&[
                'p' as AlphaChar,
                'r' as AlphaChar,
                'o' as AlphaChar,
                'g' as AlphaChar,
                'r' as AlphaChar,
                'e' as AlphaChar,
                's' as AlphaChar,
                's' as AlphaChar,
                0x0000,
            ])
            .unwrap(),
            data: TRIE_DATA_UNREAD,
        },
        //     {(AlphaChar *)NULL,          TRIE_DATA_ERROR},
    ]
}

// static Bool
unsafe fn is_walkables_include(
    c: AlphaChar,
    walkables: *const AlphaChar,
    mut n_elm: usize,
) -> bool {
    let mut p = walkables;
    while n_elm > 0 {
        n_elm -= 1;
        if *p == c {
            return true;
        }
        p = p.offset(1);
    }
    false
}

// static void
unsafe fn print_walkables(walkables: *const AlphaChar, n_elm: usize) {
    // int i;
    let mut p = walkables;

    print!("{{");
    // for (i = 0; i < n_elm; i++) {
    for i in 0..n_elm {
        if i > 0 {
            print!(", ");
        }
        print!("'{}'", *p);
        p = p.offset(1);
    }
    print!("}}");
}

const ALPHABET_SIZE: usize = 256;

use datrie::{
    AlphaChar, AlphaStr, DatrieResult, {Trie, TrieState, DA_TRUE},
};

use crate::utils::{en_trie_new, msg_step};

#[test]
fn test_walk() -> DatrieResult<()> {
    unsafe {
        //     Trie       *test_trie;
        //     DictRec    *dict_p;
        //     TrieState  *s, *t, *u;
        let mut walkables = [0; ALPHABET_SIZE];
        //     int         n;
        //     Bool        is_failed;
        //     TrieData    data;

        msg_step("Preparing trie");
        let mut test_trie = en_trie_new()?;
        //     if (!test_trie) {
        //         fprint! (stderr, "Fail to create test trie\n");
        //         goto err_trie_not_created;
        //     }

        /* store */
        let walk_dict = get_walk_dict();
        for dict_p in &walk_dict {
            assert!(
                Trie::store(&mut test_trie, dict_p.key, dict_p.data),
                "Failed to add key '{:?}', data {}.\n",
                dict_p.key,
                dict_p.data
            );
            //             goto err_trie_created;
            //         }
        }

        println!("Now the trie structure is supposed to be:\n");
        //     print! (
        println!("          +---o-> (3) -o-> (4) -l-> [5]");
        println!("          |");
        println!("          |        +---i-> (7) -z-> (8) -e-> [9]");
        println!("          |        |");
        println!("(1) -p-> (2) -r-> (6) -e-> (10) -v-> (11) -i-> (12) -e-> (13) -w-> [14]");
        println!("                   |         |");
        println!("                   |         +---p-> (15) -a-> (16) -r-> (17) -e-> [18]");
        println!("                   |");
        println!("                   +---o-> (19) -d-> (20) -u-> (21) -c-> (22) -e-> [23]");
        println!("                             |");
        println!(
            "                             +---g-> (24) -r-> (25) -e-> (26) -s-> (27) -s-> [28]"
        );
        println!();
        //     );

        /* walk */
        msg_step("Test walking");
        let s = Trie::root(&test_trie);
        assert!(!s.is_null());
        //     if (!s) {
        //         print! ("Failed to get trie root state\n");
        //         goto err_trie_created;
        //     }

        msg_step("Test walking with 'p'");
        assert_eq!(
            TrieState::is_walkable(s, 'p' as AlphaChar),
            DA_TRUE,
            "Trie state is not walkable with 'p'\n"
        );
        //         goto err_TrieState::s_created;
        //     }
        assert_eq!(
            TrieState::walk(s, 'p' as AlphaChar),
            DA_TRUE,
            "Failed to walk with 'p'\n"
        );
        //         goto err_TrieState::s_created;
        //     }

        msg_step("Now at (2), walkable chars should be {'o', 'r'}");
        let mut is_failed = false;
        let n = TrieState::walkable_chars(s, walkables.as_mut_ptr(), ALPHABET_SIZE as i32);
        assert_eq!(2, n, "Walkable chars should be exactly 2, got {}\n", n);
        //         is_failed = TRUE;
        //     }
        if !is_walkables_include('o' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'o'");
            is_failed = true;
        }
        if !is_walkables_include('r' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'r'");
            is_failed = true;
        }
        if is_failed {
            print!("Walkables = ");
            print_walkables(walkables.as_ptr(), n as usize);
            println!();
            panic!("walkables failed");
            //         goto err_TrieState::s_created;
        }

        msg_step("Try walking from (2) with 'o' to (3)");
        let t = TrieState::trie_state_clone(s);
        assert!(!t.is_null(), "Failed to clone trie state\n");
        //         goto err_TrieState::s_created;
        //     }
        assert_eq!(
            TrieState::walk(t, 'o' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (2) with 'o' to (3)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        assert_eq!(
            TrieState::is_single(t),
            DA_TRUE,
            "(3) should be single, but isn't.\n"
        );
        //         goto err_TrieState::t_created;
        //     }

        msg_step("Try walking from (3) with 'o' to (4)");
        assert_eq!(
            TrieState::walk(t, 'o' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (3) with 'o' to (4)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        assert_eq!(
            TrieState::is_single(t),
            DA_TRUE,
            "(4) should be single, but isn't.\n"
        );
        //         goto err_TrieState::t_created;
        //     }

        msg_step("Try walking from (4) with 'l' to (5)");
        assert_eq!(
            TrieState::walk(t, 'l' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (4) with 'l' to (5)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        assert!(
            TrieState::is_terminal(t),
            "(5) should be terminal, but isn't.\n"
        );
        //         goto err_TrieState::t_created;
        //     }

        /* get key & data */
        msg_step("Try getting data from (5)");
        let data = TrieState::get_data(t);
        assert_ne!(TRIE_DATA_ERROR, data, "Failed to get data from (5)\n");
        //         goto err_TrieState::t_created;
        //     }
        assert_eq!(TRIE_DATA_UNREAD, data, "Mismatched data from (5),");
        //         goto err_TrieState::t_created;
        //     }

        /* walk s from (2) with 'r' to (6) */
        msg_step("Try walking from (2) with 'r' to (6)");
        assert_eq!(
            TrieState::walk(s, 'r' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (2) with 'r' to (6)\n"
        );
        //         goto err_TrieState::t_created;
        //     }

        msg_step("Now at (6), walkable chars should be {'e', 'i', 'o'}");
        //     is_failed = FALSE;
        let n = TrieState::walkable_chars(s, walkables.as_mut_ptr(), ALPHABET_SIZE as i32);
        assert_eq!(3, n, "Walkable chars should be exactly 3");
        //         is_failed = TRUE;
        //     }
        if !is_walkables_include('e' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'e'");
            is_failed = true;
        }
        if !is_walkables_include('i' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'i'");
            is_failed = true;
        }
        if !is_walkables_include('o' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'o'");
            is_failed = true;
        }
        if is_failed {
            print!("Walkables = ");
            print_walkables(walkables.as_ptr(), n as usize);
            println!();
            //         goto err_TrieState::t_created;
        }

        /* walk from s (6) with "ize" */
        msg_step("Try walking from (6) with 'i' to (7)");
        TrieState::trie_state_copy(t, s);
        assert_eq!(
            TrieState::walk(t, 'i' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (6) with 'i' to (7)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        msg_step("Try walking from (7) with 'z' to (8)");
        assert_eq!(
            TrieState::walk(t, 'z' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (7) with 'z' to (8)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        assert_eq!(
            TrieState::is_single(t),
            DA_TRUE,
            "(7) should be single, but isn't.\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        msg_step("Try walking from (8) with 'e' to (9)");
        assert_eq!(
            TrieState::walk(t, 'e' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (8) with 'e' to (9)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        assert!(
            TrieState::is_terminal(t),
            "(9) should be terminal, but isn't.\n"
        );
        //         goto err_TrieState::t_created;
        //     }

        msg_step("Try getting data from (9)");
        let data = TrieState::get_data(t);
        assert_ne!(TRIE_DATA_ERROR, data, "Failed to get data from (9)\n");
        //         goto err_TrieState::t_created;
        //     }
        assert_eq!(TRIE_DATA_UNREAD, data, "Mismatched data from (9)");
        //         goto err_TrieState::t_created;
        //     }

        /* walk from u = s (6) with 'e' to (10) */
        msg_step("Try walking from (6) with 'e' to (10)");
        let u = TrieState::trie_state_clone(s);
        assert!(!u.is_null(), "Failed to clone trie state\n");
        //         goto err_TrieState::t_created;
        //     }
        assert_eq!(
            TrieState::walk(u, 'e' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (6) with 'e' to (10)\n"
        );
        //         goto err_TrieState::u_created;
        //     }

        /* walkable chars from (10) should be {'p', 'v'} */
        msg_step("Now at (10), walkable chars should be {'p', 'v'}");
        is_failed = false;
        let n = TrieState::walkable_chars(u, walkables.as_mut_ptr(), ALPHABET_SIZE as i32);
        assert_eq!(2, n, "Walkable chars should be exactly 2");
        //         is_failed = TRUE;
        //     }
        if !is_walkables_include('p' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'p'");
            is_failed = true;
        }
        if !is_walkables_include('v' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'v'");
            is_failed = true;
        }
        if is_failed {
            print!("Walkables = ");
            print_walkables(walkables.as_ptr(), n as usize);
            println!();
            //         goto err_TrieState::u_created;
        }

        /* walk from u (10) with "view" */
        msg_step("Try walking from (10) with 'v' to (11)");
        TrieState::trie_state_copy(t, u);
        assert_eq!(
            TrieState::walk(t, 'v' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (10) with 'v' to (11)\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        assert_eq!(
            TrieState::is_single(t),
            DA_TRUE,
            "(11) should be single, but isn't.\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        msg_step("Try walking from (11) with 'i' to (12)");
        assert_eq!(
            TrieState::walk(t, 'i' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (11) with 'i' to (12)\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        msg_step("Try walking from (12) with 'e' to (13)");
        assert_eq!(
            TrieState::walk(t, 'e' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (12) with 'e' to (13)\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        msg_step("Try walking from (13) with 'w' to (14)");
        assert_eq!(
            TrieState::walk(t, 'w' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (13) with 'w' to (14)\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        assert!(
            TrieState::is_terminal(t),
            "(14) should be terminal, but isn't.\n"
        );
        //         goto err_TrieState::u_created;
        //     }

        msg_step("Try getting data from (14)");
        let data = TrieState::get_data(t);
        assert_ne!(TRIE_DATA_ERROR, data, "Failed to get data from (14)\n");
        //         goto err_TrieState::u_created;
        //     }
        assert_eq!(TRIE_DATA_UNREAD, data, "Mismatched data from (14)");
        //         goto err_TrieState::u_created;
        //     }

        /* walk from u (10) with "pare" */
        msg_step("Try walking from (10) with 'p' to (15)");
        TrieState::trie_state_copy(t, u);
        assert_eq!(
            TrieState::walk(t, 'p' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (10) with 'p' to (15)\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        assert_eq!(
            TrieState::is_single(t),
            DA_TRUE,
            "(15) should be single, but isn't.\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        msg_step("Try walking from (15) with 'a' to (16)");
        assert_eq!(
            TrieState::walk(t, 'a' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (15) with 'a' to (16)\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        msg_step("Try walking from (16) with 'r' to (17)");
        assert_eq!(
            TrieState::walk(t, 'r' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (16) with 'r' to (17)\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        msg_step("Try walking from (17) with 'e' to (18)");
        assert_eq!(
            TrieState::walk(t, 'e' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (17) with 'e' to (18)\n"
        );
        //         goto err_TrieState::u_created;
        //     }
        assert!(
            TrieState::is_terminal(t),
            "(18) should be terminal, but isn't.\n"
        );
        //         goto err_TrieState::u_created;
        //     }

        msg_step("Try getting data from (18)");
        let data = TrieState::get_data(t);
        assert_ne!(TRIE_DATA_ERROR, data, "Failed to get data from (18)\n");
        //         goto err_TrieState::u_created;
        //     }
        assert_eq!(TRIE_DATA_UNREAD, data, "Mismatched data from (18)");
        //         goto err_TrieState::u_created;
        //     }

        TrieState::free(u);

        /* walk s from (6) with 'o' to (19) */
        msg_step("Try walking from (6) with 'o' to (19)");
        assert_eq!(
            TrieState::walk(s, 'o' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (6) with 'o' to (19)\n"
        );
        //         goto err_TrieState::t_created;
        //     }

        msg_step("Now at (19), walkable chars should be {'d', 'g'}");
        is_failed = false;
        let n = TrieState::walkable_chars(s, walkables.as_mut_ptr(), ALPHABET_SIZE as i32);
        assert_eq!(2, n, "Walkable chars should be exactly 2");
        //         is_failed = TRUE;
        //     }
        if !is_walkables_include('d' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'd'");
            is_failed = true;
        }
        if !is_walkables_include('g' as AlphaChar, walkables.as_ptr(), n as usize) {
            println!("Walkable chars do not include 'g'");
            is_failed = true;
        }
        if is_failed {
            print!("Walkables = ");
            print_walkables(walkables.as_ptr(), n as usize);
            println!();
            //         goto err_TrieState::t_created;
        }

        /* walk from s (19) with "duce" */
        msg_step("Try walking from (19) with 'd' to (20)");
        TrieState::trie_state_copy(t, s);
        assert_eq!(
            TrieState::walk(t, 'd' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (19) with 'd' to (20)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        assert_eq!(
            TrieState::is_single(t),
            DA_TRUE,
            "(20) should be single, but isn't.\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        msg_step("Try walking from (20) with 'u' to (21)");
        assert_eq!(
            TrieState::walk(t, 'u' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (20) with 'u' to (21)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        msg_step("Try walking from (21) with 'c' to (22)");
        assert_eq!(
            TrieState::walk(t, 'c' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (21) with 'c' to (22)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        msg_step("Try walking from (22) with 'e' to (23)");
        assert_eq!(
            TrieState::walk(t, 'e' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (22) with 'e' to (23)\n"
        );
        //         goto err_TrieState::t_created;
        //     }
        assert!(
            TrieState::is_terminal(t),
            "(23) should be terminal, but isn't.\n"
        );
        //         goto err_TrieState::t_created;
        //     }

        msg_step("Try getting data from (23)");
        let data = TrieState::get_data(t);
        assert_ne!(TRIE_DATA_ERROR, data, "Failed to get data from (23)\n");
        //         goto err_TrieState::t_created;
        //     }
        assert_eq!(TRIE_DATA_UNREAD, data, "Mismatched data from (23)");
        //         goto err_TrieState::t_created;
        //     }

        TrieState::free(t);

        /* walk from s (19) with "gress" */
        msg_step("Try walking from (19) with 'g' to (24)");
        assert_eq!(
            TrieState::walk(s, 'g' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (19) with 'g' to (24)\n"
        );
        //         goto err_TrieState::s_created;
        //     }
        assert_eq!(
            TrieState::is_single(s),
            DA_TRUE,
            "(24) should be single, but isn't.\n"
        );
        //         goto err_TrieState::s_created;
        //     }
        msg_step("Try walking from (24) with 'r' to (25)");
        assert_eq!(
            TrieState::walk(s, 'r' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (24) with 'r' to (25)\n"
        );
        //         goto err_TrieState::s_created;
        //     }
        msg_step("Try walking from (25) with 'e' to (26)");
        assert_eq!(
            TrieState::walk(s, 'e' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (25) with 'e' to (26)\n"
        );
        //         goto err_TrieState::s_created;
        //     }
        msg_step("Try walking from (26) with 's' to (27)");
        assert_eq!(
            TrieState::walk(s, 's' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (26) with 's' to (27)\n"
        );
        //         goto err_TrieState::s_created;
        //     }
        msg_step("Try walking from (27) with 's' to (28)");
        assert_eq!(
            TrieState::walk(s, 's' as AlphaChar),
            DA_TRUE,
            "Failed to walk from (27) with 's' to (28)\n"
        );
        //         goto err_TrieState::s_created;
        //     }
        assert!(
            TrieState::is_terminal(s),
            "(28) should be terminal, but isn't.\n"
        );
        //         goto err_TrieState::s_created;
        //     }

        msg_step("Try getting data from (28)");
        let data = TrieState::get_data(s);
        assert_ne!(TRIE_DATA_ERROR, data, "Failed to get data from (28)\n");
        //         goto err_TrieState::s_created;
        //     }
        assert_eq!(TRIE_DATA_UNREAD, data, "Mismatched data from (28)");
        //         goto err_TrieState::s_created;
        //     }

        TrieState::free(s);
        //     trie_free (test_trie);
        //     return 0;

        // err_TrieState::u_created:
        //     TrieState::free (u);
        // err_TrieState::t_created:
        //     TrieState::free (t);
        // err_TrieState::s_created:
        //     TrieState::free (s);
        // err_trie_created:
        //     trie_free (test_trie);
        // err_trie_not_created:
        //     return 1;
    }
    Ok(())
}
