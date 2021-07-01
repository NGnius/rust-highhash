//! MurmurHash3 algorithm by Austin Appleby.
mod murmur3_32;
mod murmur3_128;
mod murmur3_128_64;

pub use murmur3_32::{Murmur3Hash32, Murmur3Hasher32,
hash32, hash32_with_seed};
pub use murmur3_128::{Murmur3Hash128, Murmur3Hasher128,
hash128, hash128_with_seed};
pub use murmur3_128_64::{Murmur3Hash128x64, Murmur3Hasher128x64,
hash128_x64, hash128_x64_with_seed};
