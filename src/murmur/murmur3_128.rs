use std::hash::{BuildHasher, Hasher};

/// Hasher for MurmurHash3 32-bit implementation of the 128-bit hashing algorithm.
#[derive(Default)]
pub struct Murmur3Hasher128 {
    buffer: Vec<u8>,
}

impl Hasher for Murmur3Hasher128 {
    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes)
    }

    fn finish(&self) -> u64 {
        hash128(&self.buffer) as u64
    }
}

/// Hash builder for MurmurHash3 32-bit implementation of the 128-bit hashing algorithm.
#[derive(Default)]
pub struct Murmur3Hash128 {}

impl BuildHasher for Murmur3Hash128 {
    type Hasher = Murmur3Hasher128;

    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::default()
    }
}

// The code below is adapted from C++ code with the following disclaimer
//-----------------------------------------------------------------------------
// MurmurHash3 was written by Austin Appleby, and is placed in the public
// domain. The author hereby disclaims copyright to this source code.

/// MurmurHash3 32-bit implementation of the 128-bit hashing algorithm.
/// This version allows you to specify a seed.
pub fn hash128_with_seed<T: AsRef<[u8]>>(v: T, seed: u32) -> u128 {
    let data = v.as_ref();
    let n_blocks = data.len() / 16;

    const C1: u32 = 0x239b961b;
    const C2: u32 = 0xab0e9789;
    const C3: u32 = 0x38b34ae5;
    const C4: u32 = 0xa1e38b93;

    const D1: u32 = 0x561ccd1b;
    const D2: u32 = 0x0bcaa747;
    const D3: u32 = 0x96cd1c35;
    const D4: u32 = 0x32ac3b17;

    let mut h1: u32 = seed;
    let mut h2: u32 = seed;
    let mut h3: u32 = seed;
    let mut h4: u32 = seed;

    // body
    for i in 0..n_blocks {
        let mut k1 = get_u32(data, (i + 0) * 4);
        let mut k2 = get_u32(data, (i + 1) * 4);
        let mut k3 = get_u32(data, (i + 2) * 4);
        let mut k4 = get_u32(data, (i + 3) * 4);

        k1 = k1.wrapping_mul(C1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(C2);
        h1 ^= k1;
        h1 = h1.rotate_left(19);
        h1 = h1.wrapping_add(h2);
        h1 = (h1.wrapping_mul(5)).wrapping_add(D1);

        k2 = k2.wrapping_mul(C2);
        k2 = k2.rotate_left(16);
        k2 = k2.wrapping_mul(C3);
        h2 ^= k2;
        h2 = h2.rotate_left(17);
        h2 = h2.wrapping_add(h3);
        h2 = (h2.wrapping_mul(5)).wrapping_add(D2);

        k3 = k3.wrapping_mul(C3);
        k3 = k3.rotate_left(17);
        k3 = k3.wrapping_mul(C4);
        h3 ^= k3;
        h3 = h3.rotate_left(15);
        h3 = h3.wrapping_add(h4);
        h3 = (h3.wrapping_mul(5)).wrapping_add(D3);

        k4 = k4.wrapping_mul(C4);
        k4 = k4.rotate_left(18);
        k4 = k4.wrapping_mul(C1);
        h4 ^= k4;
        h4 = h4.rotate_left(13);
        h4 = h4.wrapping_add(h1);
        h4 = (h4.wrapping_mul(5)).wrapping_add(D4);
    }

    // tail
    //let tail = data.clone();
    let tail_num = n_blocks * 16;
    let mut k1 = 0;
    let mut k2 = 0;
    let mut k3 = 0;
    let mut k4 = 0;
    for i in (1..=data.len() & 15).rev() {
        match i {
            15 => k4 ^= (data[tail_num + 14] as u32) << 16,
            14 => k4 ^= (data[tail_num + 13] as u32) << 8,
            13 => {
                k4 ^= (data[tail_num + 12] as u32) << 0;
                k4 = k4.wrapping_mul(C4);
                k4 = k4.rotate_left(18);
                k4 = k4.wrapping_mul(C1);
                h4 ^= k4;
            }

            12 => k3 ^= (data[tail_num + 11] as u32) << 24,
            11 => k3 ^= (data[tail_num + 10] as u32) << 16,
            10 => k3 ^= (data[tail_num + 9] as u32) << 8,
            9 => {
                k3 ^= (data[tail_num + 8] as u32) << 0;
                k3 = k3.wrapping_mul(C3);
                k3 = k3.rotate_left(17);
                k3 = k3.wrapping_mul(C4);
                h3 ^= k3;
            }

            8 => k2 ^= (data[tail_num + 7] as u32) << 24,
            7 => k2 ^= (data[tail_num + 6] as u32) << 16,
            6 => k2 ^= (data[tail_num + 5] as u32) << 8,
            5 => {
                k2 ^= (data[tail_num + 4] as u32) << 0;
                k2 = k2.wrapping_mul(C2);
                k2 = k2.rotate_left(16);
                k2 = k2.wrapping_mul(C3);
                h2 ^= k2;
            }

            4 => k1 ^= (data[tail_num + 3] as u32) << 24,
            3 => k1 ^= (data[tail_num + 2] as u32) << 16,
            2 => k1 ^= (data[tail_num + 1] as u32) << 8,
            1 => {
                k1 ^= data[tail_num] as u32;
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
    h2 ^= data.len() as u32;
    h3 ^= data.len() as u32;
    h4 ^= data.len() as u32;

    h1 = h1.wrapping_add(h2).wrapping_add(h3).wrapping_add(h4);
    h2 = h2.wrapping_add(h1);
    h3 = h3.wrapping_add(h1);
    h4 = h4.wrapping_add(h1);

    h1 = fmix32(h1);
    h2 = fmix32(h2);
    h3 = fmix32(h3);
    h4 = fmix32(h4);

    h1 = h1.wrapping_add(h2).wrapping_add(h3).wrapping_add(h4);
    h2 = h2.wrapping_add(h1);
    h3 = h3.wrapping_add(h1);
    h4 = h4.wrapping_add(h1);

    (h1 as u128) << 96 | (h2 as u128) << 64 | (h3 as u128) << 32 | (h4 as u128)
}

/// MurmurHash3 32-bit implementation of the 128-bit hashing algorithm.
/// The seed is always 0 in this version.
pub fn hash128<T: AsRef<[u8]>>(v: T) -> u128 {
    hash128_with_seed(v, 0)
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
            crate::murmur::murmur3_128::hash128_with_seed("StandardBlockEntityDescriptorV4", 4919),
            0x4da5b4125adab9dc7d30c1c10bb975f7
        );
    }
}
