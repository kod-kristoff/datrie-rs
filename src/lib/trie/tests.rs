use crate::trie::AlphaChar;
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
    unsafe {
        Trie::store(&mut trie, ['a' as AlphaChar, 0x0000].as_ptr(), 2);
    }
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
