#[macro_use]
extern crate error_chain;

extern crate flate2;
extern crate byteorder;

pub mod tag;
pub mod read;

pub use tag::Tag;
pub use read::*;

pub mod errors;