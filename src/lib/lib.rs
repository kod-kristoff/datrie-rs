#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
pub mod alpha_map;
pub mod darray;
pub mod dstring;
mod error;
pub mod fileutils;
pub mod tail;
pub mod trie;
// pub mod trie_char_string;
pub mod trie_string;

pub use crate::error::{DatrieError, ErrorKind};

pub type DatrieResult<T> = Result<T, DatrieError>;
