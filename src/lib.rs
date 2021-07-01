//! High speed hashing algorithms.
#![warn(missing_docs)]

pub mod murmur;

pub use murmur::Murmur3Hasher32 as Murmur3Hasher;
