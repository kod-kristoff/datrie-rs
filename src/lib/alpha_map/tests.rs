use super::AlphaMap2;
use crate::DatrieResult;

use super::{alpha_map_serialize_bin, AlphaMap};
mod get_serialized_size {
    use super::*;

    #[test]
    fn single_range() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x00, 0xff)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x00, 0xff)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }

    #[test]
    fn separated_ranges() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x00, 0x20)?;
        alpha_map.add_range(0x30, 0xff)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 24);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x00, 0x20)?;
        alpha_map2.add_range(0x30, 0xff)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn overlapping_ranges() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x00, 0x40)?;
        alpha_map.add_range(0x30, 0xff)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x00, 0x40)?;
        alpha_map2.add_range(0x30, 0xff)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn overlapping_ranges2() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x30, 0xff)?;
        alpha_map.add_range(0x00, 0x40)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x30, 0xff)?;
        alpha_map2.add_range(0x00, 0x40)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn added_range_included_in_existent() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x00, 0x60)?;
        alpha_map.add_range(0x30, 0x50)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x00, 0x60)?;
        alpha_map2.add_range(0x30, 0x50)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn existent_range_included_in_added() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x30, 0x50)?;
        alpha_map.add_range(0x00, 0x60)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x30, 0x50)?;
        alpha_map2.add_range(0x00, 0x60)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn existent_range_adjacent_to_added_lower() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x30, 0x50)?;
        alpha_map.add_range(0x51, 0x60)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x30, 0x50)?;
        alpha_map2.add_range(0x51, 0x60)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn existent_range_adjacent_to_added_higher() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x31, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x31, 0x50)?;
        alpha_map2.add_range(0x21, 0x30)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn existent_ranges_is_connected_by_added() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x41, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        alpha_map.add_range(0x31, 0x40)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x41, 0x50)?;
        alpha_map2.add_range(0x21, 0x30)?;
        alpha_map2.add_range(0x31, 0x40)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn existent_range_is_connected_by_added_1() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x41, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        alpha_map.add_range(0x31, 0x3f)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 24);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x41, 0x50)?;
        alpha_map2.add_range(0x21, 0x30)?;
        alpha_map2.add_range(0x31, 0x3f)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn existent_range_is_connected_by_added_2() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x41, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        alpha_map.add_range(0x32, 0x40)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 24);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x41, 0x50)?;
        alpha_map2.add_range(0x21, 0x30)?;
        alpha_map2.add_range(0x32, 0x40)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
    #[test]
    fn existent_ranges_is_not_connected_by_added() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x41, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        alpha_map.add_range(0x32, 0x3f)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 32);
        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x41, 0x50)?;
        alpha_map2.add_range(0x21, 0x30)?;
        alpha_map2.add_range(0x32, 0x3f)?;
        let size2 = alpha_map2.get_serialized_size();
        assert_eq!(size, size2);
        Ok(())
    }
}
mod char_to_trie {
    use crate::alpha_map::{alpha_map_char_to_trie, AlphaChar, TrieIndex};

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0)]
    #[case(0x41, 1)]
    #[case(0x60, 0x7fffffff)]
    fn char_to_trie_works(
        #[case] given: AlphaChar,
        #[case] expected: TrieIndex,
    ) -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::new();
        alpha_map.add_range(0x41, 0x50)?;
        let actual = unsafe { alpha_map_char_to_trie(&alpha_map, given) };
        assert_eq!(actual, expected);

        let mut alpha_map2 = AlphaMap2::default();
        alpha_map2.add_range(0x41, 0x50)?;
        let actual = alpha_map2.char_to_trie(given);
        assert_eq!(actual, expected);
        Ok(())
    }
}
#[test]
fn get_total_ranges_works() -> DatrieResult<()> {
    let mut alpha_map = AlphaMap::new();
    alpha_map.add_range(0x00, 0xff)?;
    let size = alpha_map.get_total_ranges();
    assert_eq!(size, 1);
    Ok(())
}

#[test]
#[ignore = "fails because of double free"]
fn serialize_works() -> DatrieResult<()> {
    let mut alpha_map = AlphaMap::new();
    alpha_map.add_range(0x00, 0xff)?;
    let size = alpha_map.get_serialized_size();
    let buf: Vec<u8> = Vec::with_capacity(size);
    let mut buf = std::mem::ManuallyDrop::new(buf);
    let mut serialized_data = buf.as_mut_ptr();
    let buf_cap = buf.capacity();
    unsafe {
        alpha_map_serialize_bin(&alpha_map, &mut serialized_data);
    }
    let serialized_data = unsafe { Vec::from_raw_parts(serialized_data, size, buf_cap) };
    let mut serialized_self_data = Vec::with_capacity(size);
    alpha_map.serialize(&mut serialized_self_data)?;
    assert_eq!(serialized_data, serialized_self_data);
    Ok(())
}
