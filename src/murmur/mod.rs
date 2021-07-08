//! MurmurHash3 algorithm by Austin Appleby.
mod murmur3_128;
mod murmur3_128_64;
mod murmur3_32;

pub use murmur3_128::{hash128, hash128_with_seed, Murmur3Hash128, Murmur3Hasher128};
pub use murmur3_128_64::{
    hash128_x64, hash128_x64_with_seed, Murmur3Hash128x64, Murmur3Hasher128x64,
};
pub use murmur3_32::{hash32, hash32_with_seed, Murmur3Hash32, Murmur3Hasher32};
