pub mod alpha_map;
pub mod darray;
pub mod dstring;
mod error;
pub mod fileutils;
pub mod tail;
pub mod trie;
pub mod trie_string;

pub use crate::error::{DatrieError, ErrorKind};

pub type DatrieResult<T> = Result<T, DatrieError>;
