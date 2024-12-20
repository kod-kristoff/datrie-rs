use crate::DatrieResult;

use super::AlphaMap;
mod get_serialized_size {
    use super::*;

    #[test]
    fn single_range() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x00, 0xff)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }

    #[test]
    fn separated_ranges() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x00, 0x20)?;
        alpha_map.add_range(0x30, 0xff)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 24);
        Ok(())
    }
    #[test]
    fn overlapping_ranges() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x00, 0x40)?;
        alpha_map.add_range(0x30, 0xff)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }
    #[test]
    fn overlapping_ranges2() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x30, 0xff)?;
        alpha_map.add_range(0x00, 0x40)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }
    #[test]
    fn added_range_included_in_existent() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x00, 0x60)?;
        alpha_map.add_range(0x30, 0x50)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }
    #[test]
    fn existent_range_included_in_added() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x30, 0x50)?;
        alpha_map.add_range(0x00, 0x60)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }
    #[test]
    fn existent_range_adjacent_to_added_lower() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x30, 0x50)?;
        alpha_map.add_range(0x51, 0x60)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }
    #[test]
    fn existent_range_adjacent_to_added_higher() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x31, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }
    #[test]
    fn existent_ranges_is_connected_by_added() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x41, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        alpha_map.add_range(0x31, 0x40)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 16);
        Ok(())
    }
    #[test]
    fn existent_range_is_connected_by_added_1() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x41, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        alpha_map.add_range(0x31, 0x3f)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 24);
        Ok(())
    }
    #[test]
    fn existent_range_is_connected_by_added_2() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x41, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        alpha_map.add_range(0x32, 0x40)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 24);
        Ok(())
    }
    #[test]
    fn existent_ranges_is_not_connected_by_added() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x41, 0x50)?;
        alpha_map.add_range(0x21, 0x30)?;
        alpha_map.add_range(0x32, 0x3f)?;
        let size = alpha_map.get_serialized_size();
        assert_eq!(size, 32);
        Ok(())
    }
}
mod char_to_trie {
    use crate::{
        alpha_map::{AlphaChar, TrieIndex},
        AlphaStr,
    };

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, Some(0))]
    #[case(0x41, Some(1))]
    #[case(0x60, None)]
    fn char_to_trie_works(
        #[case] given: AlphaChar,
        #[case] expected: Option<TrieIndex>,
    ) -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x41, 0x50)?;
        let actual = alpha_map.char_to_trie(given);

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn char_to_trie_str() -> DatrieResult<()> {
        let mut alpha_map = AlphaMap::default();
        alpha_map.add_range(0x00, 0xff)?;
        let alpha_key = AlphaStr::from_slice_with_nul(&[97, 112, 97, 0]).unwrap();

        let key2 = alpha_map.char_to_trie_str(alpha_key).expect("a string");

        assert_eq!(key2.count_bytes(), 3);

        Ok(())
    }
}
#[test]
fn get_total_ranges_works() -> DatrieResult<()> {
    let mut alpha_map = AlphaMap::default();
    alpha_map.add_range(0x00, 0xff)?;
    let size = alpha_map.get_total_ranges();
    assert_eq!(size, 1);
    Ok(())
}

#[test]
// #[ignore = "fails because of double free"]
fn serialize_works() -> DatrieResult<()> {
    let mut alpha_map = AlphaMap::default();
    alpha_map.add_range(0x00, 0xff)?;
    let size = alpha_map.get_serialized_size();

    let mut serialized_self_data = Vec::with_capacity(size);
    alpha_map.serialize(&mut serialized_self_data)?;
    Ok(())
}
