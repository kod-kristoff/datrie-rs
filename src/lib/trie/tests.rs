use crate::trie::AlphaChar;
use crate::AlphaStr;
use crate::{trie::Trie, DatrieResult};

use crate::alpha_map::AlphaMap;

#[test]
fn get_serialized_size_works() -> DatrieResult<()> {
    let mut alpha_map = AlphaMap::default();
    alpha_map.add_range(0x00, 0xff)?;
    let trie = Trie::new(&alpha_map)?;
    let size = trie.get_serialized_size();
    assert_eq!(size, 52);
    Ok(())
}
#[test]
fn serialize_to_slice_works() -> DatrieResult<()> {
    let mut alpha_map = AlphaMap::default();
    alpha_map.add_range(0x00, 0xff)?;
    let mut trie = Trie::new(&alpha_map)?;
    Trie::store(
        &mut trie,
        AlphaStr::from_slice_with_nul(&['a' as AlphaChar, 0x0000]).unwrap(),
        2,
    );
    let size = trie.get_serialized_size();
    let mut serialized_data = Vec::with_capacity(size);
    trie.serialize_safe(&mut serialized_data)?;
    let mut serialized_to_slice = vec![0; size];
    let serialized_size = trie
        .serialize_to_slice(serialized_to_slice.as_mut_slice())
        .unwrap();
    assert_eq!(serialized_size, size);
    for (i, (l, r)) in serialized_data
        .iter()
        .zip(serialized_to_slice.iter())
        .enumerate()
    {
        assert_eq!(l, r, "imdex {} fsilrd", i);
    }
    assert_eq!(serialized_data, serialized_to_slice);
    Ok(())
}

#[test]
fn cmp_store_and_store2() -> DatrieResult<()> {
    let mut alpha_map = AlphaMap::default();
    alpha_map.add_range(0x00, 0xff)?;
    let mut trie = Trie::new(&alpha_map)?;
    let mut trie2 = Trie::new(&alpha_map)?;

    for (alpha_str, data) in &[
        (AlphaStr::from_slice_with_nul(&[0xff, 0xff, 0]).unwrap(), 1),
        (AlphaStr::from_slice_with_nul(&[97, 112, 97, 0]).unwrap(), 2),
        (AlphaStr::from_slice_with_nul(&[97, 112, 98, 0]).unwrap(), 3),
        (AlphaStr::from_slice_with_nul(&[97, 113, 97, 0]).unwrap(), 4),
        (AlphaStr::from_slice_with_nul(&[97, 113, 98, 0]).unwrap(), 5),
    ] {
        dbg!(alpha_str);
        assert_eq!(trie.store(alpha_str, *data), trie2.store(alpha_str, *data));
        // dbg!(&trie.tail);
        // dbg!(&trie2.tail);
        dbg!(data);
        // assert_eq!(trie.da, trie2.da);
        assert_eq!(trie.tail, trie2.tail);
    }
    // assert!(false);

    Ok(())
}
