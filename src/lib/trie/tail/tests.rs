use crate::{trie::TrieCharStr, DatrieResult};

use super::*;

#[test]
fn get_serialized_size_works() -> DatrieResult<()> {
    let mut tail = Tail::new();
    assert_eq!(tail.get_serialized_size(), 12);
    tail.alloc_block();
    assert_eq!(tail.get_serialized_size(), 22);
    tail.add_suffix2(TrieCharString::new(b"ap").unwrap());
    assert_eq!(tail.get_serialized_size(), 34);
    Ok(())
}

#[test]
fn walk_char() {
    let mut tail = Tail::new();

    unsafe {
        tail.add_suffix([b'a', b'p', b'\0'].as_ptr());
        assert!(!tail.get_suffix(1).is_null());
    }
    // walk 'a'
    let mut suffix_idx = 0;
    let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'a') };

    assert_eq!(res, DA_TRUE);
    assert_eq!(suffix_idx, 1);

    let mut suffix_idx2 = 0;
    assert!(tail.walk_char2(1, &mut suffix_idx2, b'a'));
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk 'a' to 'p'
    let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'p') };

    assert_eq!(res, DA_TRUE);
    assert_eq!(suffix_idx, 2);

    assert!(tail.walk_char2(1, &mut suffix_idx2, b'p'));
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk 'p' to '\0'
    let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'\0') };

    assert_eq!(res, DA_TRUE);
    assert_eq!(suffix_idx, 2);

    assert!(tail.walk_char2(1, &mut suffix_idx2, b'\0'));
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // try walk 'a' to 'b'
    let mut suffix_idx = 1;
    let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'b') };

    assert_eq!(res, DA_FALSE);
    assert_eq!(suffix_idx, 1);

    let mut suffix_idx2 = 1;
    assert!(!tail.walk_char2(1, &mut suffix_idx2, b'b'));
    assert_eq!(suffix_idx as usize, suffix_idx2);
}
#[test]
fn walk_char_2() {
    let mut tail = Tail::new();

    tail.add_suffix2(TrieCharString::new(&[b'a', b'p']).unwrap());
    assert!(!tail.get_suffix2(1).is_none());
    // walk 'a'
    let mut suffix_idx = 0;
    let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'a') };

    assert_eq!(res, DA_TRUE);
    assert_eq!(suffix_idx, 1);

    let mut suffix_idx2 = 0;
    assert!(tail.walk_char2(1, &mut suffix_idx2, b'a'));
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk 'a' to 'p'
    let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'p') };

    assert_eq!(res, DA_TRUE);
    assert_eq!(suffix_idx, 2);

    assert!(tail.walk_char2(1, &mut suffix_idx2, b'p'));
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk 'p' to '\0'
    let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'\0') };

    assert_eq!(res, DA_TRUE);
    assert_eq!(suffix_idx, 2);

    assert!(tail.walk_char2(1, &mut suffix_idx2, b'\0'));
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // try walk 'a' to 'b'
    let mut suffix_idx = 1;
    let res = unsafe { tail.walk_char(1, &mut suffix_idx, b'b') };

    assert_eq!(res, DA_FALSE);
    assert_eq!(suffix_idx, 1);

    let mut suffix_idx2 = 1;
    assert!(!tail.walk_char2(1, &mut suffix_idx2, b'b'));
    assert_eq!(suffix_idx as usize, suffix_idx2);
}
#[test]
fn walk_str() {
    let mut tail = Tail::new();
    unsafe {
        tail.add_suffix(
            TrieCharStr::from_bytes_until_nul(b"apa\0")
                .unwrap()
                .as_ptr(),
        );
        tail.add_suffix(b"bad\0".as_ptr());
        assert!(!tail.get_suffix(1).is_null());
    }
    // walk "apa" with (0,"a")
    let mut suffix_idx = 0;
    let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"a".as_ptr(), 1) };

    assert_eq!(res, 1);
    assert_eq!(suffix_idx, 1);
    let mut suffix_idx2 = 0;
    assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"a"), 1);
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk "apa" with (0,"ap")
    let mut suffix_idx = 0;
    let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"ap".as_ptr(), 2) };

    assert_eq!(res, 2);
    assert_eq!(suffix_idx, 2);
    let mut suffix_idx2 = 0;
    assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"ap"), 2);
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk "apa" with (0,"al")
    let mut suffix_idx = 0;
    let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"al".as_ptr(), 2) };

    assert_eq!(res, 1);
    assert_eq!(suffix_idx, 1);
    let mut suffix_idx2 = 0;
    assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"al"), 1);
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk "apa" with (1,"pa")
    let mut suffix_idx = 1;
    println!("calling tail.walk_str(1,1,'pa',2)");

    let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"pa".as_ptr(), 2) };

    assert_eq!(res, 2);
    assert_eq!(suffix_idx, 3);
    let mut suffix_idx2 = 1;
    println!("calling tail.walk_str2(1,1,'pa')");
    assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"pa"), 2);
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk "apa" with (1,"la")
    let mut suffix_idx = 1;
    let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"la".as_ptr(), 2) };

    assert_eq!(res, 0);
    assert_eq!(suffix_idx, 1);
    let mut suffix_idx2 = 1;
    assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"la"), 0);
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk "apa" with (1,"pap")
    let mut suffix_idx = 1;
    let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"pap".as_ptr(), 2) };

    assert_eq!(res, 2);
    assert_eq!(suffix_idx, 3);
    let mut suffix_idx2 = 1;
    assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"pap"), 2);
    assert_eq!(suffix_idx as usize, suffix_idx2);
    // walk "apa" with (5,"pap")
    let mut suffix_idx = 5;
    let res = unsafe { tail.walk_str(1, &mut suffix_idx, b"pap".as_ptr(), 2) };

    assert_eq!(res, 0);
    assert_eq!(suffix_idx, 5);
    let mut suffix_idx2 = 5;
    assert_eq!(tail.walk_str2(1, &mut suffix_idx2, b"pap"), 0);
    assert_eq!(suffix_idx as usize, suffix_idx2);
}

#[test]
fn set_and_get_data() {
    let mut tail = Tail::new();

    assert!(!tail.set_data(1, 2));
    assert_eq!(tail.get_data(1), -1);
    assert_eq!(tail.get_data2(1), None);

    let idx = tail.add_suffix2(TrieCharString::new(b"apa").unwrap());

    assert!(tail.set_data(idx, 2));

    assert_eq!(tail.get_data(idx), 2);
    assert_eq!(tail.get_data2(idx), Some(2));
}
