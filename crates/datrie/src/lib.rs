#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(path_statements)]
#![allow(clippy::no_effect)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::needless_return)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::comparison_chain)]
pub(crate) mod alpha_map;
mod darray;
mod dstring;
mod error;
pub mod fileutils;
mod trie;
// pub mod trie_char_string;
pub mod alpha_str;
pub mod trie_str;
mod trie_string;

pub use crate::alpha_str::AlphaStr;
pub use crate::error::{DatrieError, ErrorKind};
pub use alpha_map::{alpha_char_strcmp, AlphaChar, AlphaMap};
pub use alpha_map::{Bool, DA_FALSE, DA_TRUE};
pub use trie::{Trie, TrieChar, TrieData, TrieEnumFunc, TrieIndex, TrieIterator, TrieState};
pub type DatrieResult<T> = Result<T, DatrieError>;
