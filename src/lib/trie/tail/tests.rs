use crate::{trie::TrieCharStr, DatrieResult};

use super::*;

#[test]
fn get_serialized_size_works() -> DatrieResult<()> {
    let mut tail = Tail::new();
    assert_eq!(tail.get_serialized_size(), 12);
    tail.alloc_block();
    assert_eq!(tail.get_serialized_size(), 22);
    tail.add_suffix(TrieCharString::new(b"ap").unwrap());
    assert_eq!(tail.get_serialized_size(), 34);
    Ok(())
}

#[test]
fn walk_char() {
    let mut tail = Tail::new();

    tail.add_suffix(TrieCharString::new(&[b'a', b'p']).unwrap());
    assert!(!tail.get_suffix(1).is_none());
    // walk 'a'
    let mut suffix_idx = 0;
    assert!(tail.walk_char(1, &mut suffix_idx, b'a'));
    assert_eq!(suffix_idx, 1);

    // walk 'a' to 'p'
    assert!(tail.walk_char(1, &mut suffix_idx, b'p'));
    assert_eq!(suffix_idx, 2);

    // walk 'p' to '\0'
    assert!(tail.walk_char(1, &mut suffix_idx, b'\0'));
    assert_eq!(suffix_idx, 2);

    // try walk 'a' to 'b'
    let mut suffix_idx = 1;
    assert!(!tail.walk_char(1, &mut suffix_idx, b'b'));
    assert_eq!(suffix_idx, 1);
}
#[test]
fn walk_str() {
    let mut tail = Tail::new();
    tail.add_suffix(
        TrieCharStr::from_bytes_until_nul(b"apa\0")
            .unwrap()
            .to_owned(),
    );
    tail.add_suffix(TrieCharString::new(b"bad").unwrap());
    assert!(tail.get_suffix(1).is_some());
    // walk "apa" with (0,"a")
    let mut suffix_idx2 = 0;
    assert_eq!(tail.walk_str(1, &mut suffix_idx2, b"a"), 1);
    assert_eq!(1, suffix_idx2);
    // walk "apa" with (0,"ap")
    let mut suffix_idx2 = 0;
    assert_eq!(tail.walk_str(1, &mut suffix_idx2, b"ap"), 2);
    assert_eq!(2, suffix_idx2);
    // walk "apa" with (0,"al")
    let mut suffix_idx2 = 0;
    assert_eq!(tail.walk_str(1, &mut suffix_idx2, b"al"), 1);
    assert_eq!(1, suffix_idx2);
    // walk "apa" with (1,"pa")
    let mut suffix_idx2 = 1;
    println!("calling tail.walk_str2(1,1,'pa')");
    assert_eq!(tail.walk_str(1, &mut suffix_idx2, b"pa"), 2);
    assert_eq!(3, suffix_idx2);
    // walk "apa" with (1,"la")
    let mut suffix_idx2 = 1;
    assert_eq!(tail.walk_str(1, &mut suffix_idx2, b"la"), 0);
    assert_eq!(1, suffix_idx2);
    // walk "apa" with (1,"pap")
    let mut suffix_idx2 = 1;
    assert_eq!(tail.walk_str(1, &mut suffix_idx2, b"pap"), 2);
    assert_eq!(3, suffix_idx2);
    // walk "apa" with (5,"pap")
    let mut suffix_idx2 = 5;
    assert_eq!(tail.walk_str(1, &mut suffix_idx2, b"pap"), 0);
    assert_eq!(5, suffix_idx2);
}

#[test]
fn set_and_get_data() {
    let mut tail = Tail::new();

    assert!(!tail.set_data(1, 2));
    assert_eq!(tail.get_data(1), None);

    let idx = tail.add_suffix(TrieCharString::new(b"apa").unwrap());

    assert!(tail.set_data(idx, 2));
    assert_eq!(tail.get_data(idx), Some(2));
}
