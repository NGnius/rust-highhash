use std::hash::{BuildHasher, Hasher};

/// Hasher for MurmurHash3 32-bit implementation of the 32-bit hashing algorithm.
#[derive(Default)]
pub struct Murmur3Hasher32 {
    buffer: Vec<u8>,
}

impl Hasher for Murmur3Hasher32 {
    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes)
    }

    fn finish(&self) -> u64 {
        hash32(&self.buffer) as u64
    }
}

/// Hash builder for MurmurHash3 32-bit implementation of the 32-bit hashing algorithm.
#[derive(Default)]
pub struct Murmur3Hash32 {}

impl BuildHasher for Murmur3Hash32 {
    type Hasher = Murmur3Hasher32;

    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::default()
    }
}

// The code below is adapted from C++ code with the following disclaimer
//-----------------------------------------------------------------------------
// MurmurHash3 was written by Austin Appleby, and is placed in the public
// domain. The author hereby disclaims copyright to this source code.

/// MurmurHash3 32-bit implementation of the 32-bit hashing algorithm.
/// This version allows you to specify a seed.
pub fn hash32_with_seed<T: AsRef<[u8]>>(v: T, seed: u32) -> u32 {
    let data = v.as_ref().clone();
    let n_blocks = data.len() / 4;

    const C1: u32 = 0xcc9e2d51;
    const C2: u32 = 0x1b873593;
    const D: u32 = 0xe6546b64;

    let mut h1: u32 = seed;

    // body
    for i in 0..n_blocks {
        let mut k1 = get_u32(data, i * 4);

        k1 = k1.wrapping_mul(C1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(C2);

        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = (h1.wrapping_mul(5)).wrapping_add(D);
    }

    // tail
    let tail = data.clone();
    let tail_num = n_blocks * 4;
    let mut k1 = 0;
    for i in (1..=data.len() & 3).rev() {
        match i {
            3 => k1 ^= (tail[tail_num + 2] as u32) << 16,
            2 => k1 ^= (tail[tail_num + 1] as u32) << 8,
            1 => {
                k1 ^= tail[tail_num] as u32;
                k1 = k1.wrapping_mul(C1);
                k1 = k1.rotate_left(15);
                k1 = k1.wrapping_mul(C2);
                h1 ^= k1;
            }
            _ => {} // should never occur
        }
    }

    // finalization
    h1 ^= data.len() as u32;
    h1 = fmix32(h1);

    h1
}

/// MurmurHash3 32-bit implementation of the 32-bit hashing algorithm.
/// The seed is always 0 in this version.
pub fn hash32<T: AsRef<[u8]>>(v: T) -> u32 {
    hash32_with_seed(v, 0)
}

#[inline(always)]
fn get_u32(data: &[u8], i: usize) -> u32 {
    let buf = [data[i], data[i + 1], data[i + 2], data[i + 3]];
    u32::from_le_bytes(buf)
}

#[inline(always)]
fn fmix32(h: u32) -> u32 {
    let mut input = h;
    input ^= input >> 16;
    input = input.wrapping_mul(0x85ebca6b);
    input ^= input >> 13;
    input = input.wrapping_mul(0xc2b2ae35);
    input ^= input >> 16;
    input
}

#[cfg(test)]
mod test {
    #[test]
    fn compliance_test() {
        assert_eq!(
            crate::murmur::murmur3_32::hash32_with_seed("StandardBlockEntityDescriptorV4", 4919),
            1357220432
        );
    }
}
