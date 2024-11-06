#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(path_statements)]
#![allow(clippy::no_effect)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::needless_return)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::comparison_chain)]
pub mod alpha_map;
pub mod darray;
pub mod dstring;
mod error;
pub mod fileutils;
pub mod trie;
// pub mod trie_char_string;
pub mod alpha_str;
pub mod trie_string;

pub use crate::alpha_str::AlphaStr;
pub use crate::error::{DatrieError, ErrorKind};
pub use alpha_map::Bool;

pub type DatrieResult<T> = Result<T, DatrieError>;
