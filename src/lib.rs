//! High speed hashing algorithms.
//! Algorithms translated into Rust from C++ source found here: https://github.com/rurban/smhasher
#![warn(missing_docs)]

pub mod city;
pub mod murmur;

pub use murmur::Murmur3Hasher32 as Murmur3Hasher;
