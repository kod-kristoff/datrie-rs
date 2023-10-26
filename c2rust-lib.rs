#![allow(dead_code)]
#![allow(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![feature(extern_types)]
#![feature(label_break_value)]


extern crate libc;
pub mod datrie {
pub mod alpha_map;
pub mod darray;
pub mod dstring;
pub mod fileutils;
pub mod tail;
pub mod trie;
pub mod trie_string;
} // mod datrie
pub mod tools {
pub mod trietool;
} // mod tools
