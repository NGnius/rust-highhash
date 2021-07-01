use std::hash::{BuildHasher, Hasher};

/// Hasher for MurmurHash3 64-bit implementation of the 128-bit hashing algorithm.
#[derive(Default)]
pub struct Murmur3Hasher128x64 {
    buffer: Vec<u8>,
}

impl Hasher for Murmur3Hasher128x64 {
    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes)
    }
    
    fn finish(&self) -> u64 {
        hash128_x64(&self.buffer) as u64
    }
}


/// Hash builder for MurmurHash3 64-bit implementation of the 128-bit hashing algorithm.
#[derive(Default)]
pub struct Murmur3Hash128x64 {}

impl BuildHasher for Murmur3Hash128x64 {
    type Hasher = Murmur3Hasher128x64;
    
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::default()
    }
}

// The code below is adapted from C++ code with the following disclaimer
//-----------------------------------------------------------------------------
// MurmurHash3 was written by Austin Appleby, and is placed in the public
// domain. The author hereby disclaims copyright to this source code.

/// MurmurHash3 64-bit implementation of the 128-bit hashing algorithm.
/// This version allows you to specify a seed.
pub fn hash128_x64_with_seed<T: AsRef<[u8]>>(v: T, seed: u32) -> u128 {
    let data = v.as_ref();
    let n_blocks = data.len() / 16;
    
    const C1: u64 = 0x87c37b91114253d5;
    const C2: u64 = 0x4cf5ad432745937f;
    
    const D1: u32 = 0x52dce729;
    const D2: u32 = 0x38495ab5;
    
    let mut h1: u64 = seed as u64;
    let mut h2: u64 = seed as u64;
    
    // body
    for i in 0..n_blocks {
        let mut k1 = get_u64(data, (i+0)*8);
        let mut k2 = get_u64(data, (i+1)*8);
        
        k1 = k1.wrapping_mul(C1);
        k1 = k1.rotate_left(31);
        k1 = k1.wrapping_mul(C2);
        h1 ^= k1;
        h1 = h1.rotate_left(27);
        h1 = h1.wrapping_add(h2);
        h1 = (h1.wrapping_mul(5)).wrapping_add(D1 as u64);

        k2 = k2.wrapping_mul(C2);
        k2 = k2.rotate_left(33);
        k2 = k2.wrapping_mul(C1);
        h2 ^= k2;
        h2 = h2.rotate_left(31);
        h2 = h2.wrapping_add(h1);
        h2 = (h2.wrapping_mul(5)).wrapping_add(D2 as u64);
    }
    
    // tail
    //let tail = data.clone();
    let tail_num = n_blocks * 16;
    let mut k1 = 0;
    let mut k2 = 0;
    println!("Tail size: {}", data.len() - tail_num);
    for i in (1..=data.len() & 15).rev() {
        println!("Doing tail {}/{} ({}/{})", i, data.len()&15, tail_num+i-1, data.len()-1);
        match i {
            15 => k2 ^= (data[tail_num+14] as u64) << 48,
            14 => k2 ^= (data[tail_num+13] as u64) << 40,
            13 => k2 ^= (data[tail_num+12] as u64) << 32,
            12 => k2 ^= (data[tail_num+11] as u64) << 24,
            11 => k2 ^= (data[tail_num+10] as u64) << 16,
            10 => k2 ^= (data[tail_num+9] as u64) << 8,
            9 => {
                k2 ^= (data[tail_num+8] as u64) << 0;
                k2 = k2.wrapping_mul(C2);
                k2 = k2.rotate_left(33);
                k2 = k2.wrapping_mul(C1);
                h2 ^= k2;
            },

            8 => k1 ^= (data[tail_num+7] as u64) << 56,
            7 => k1 ^= (data[tail_num+6] as u64) << 48,
            6 => k1 ^= (data[tail_num+5] as u64) << 40,
            5 => k1 ^= (data[tail_num+4] as u64) << 32,
            4 => k1 ^= (data[tail_num+3] as u64) << 24,
            3 => k1 ^= (data[tail_num+2] as u64) << 16,
            2 => k1 ^= (data[tail_num+1] as u64) << 8,
            1 => {
                k1 ^= data[tail_num] as u64;
                k1 = k1.wrapping_mul(C1);
                k1 = k1.rotate_left(31);
                k1 = k1.wrapping_mul(C2);
                h1 ^= k1;
            },
            _ => {}, // should never occur
        }
    }
    
    // finalization
    h1 ^= data.len() as u64;
    h2 ^= data.len() as u64;
    
    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);
    
    h1 = fmix64(h1);
    h2 = fmix64(h2);
    
    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);
    
    (h1 as u128) << 64
    | (h2 as u128) << 0
}

/// MurmurHash3 64-bit implementation of the 128-bit hashing algorithm.
/// The seed is always 0 in this version.
pub fn hash128_x64<T: AsRef<[u8]>>(v: T) -> u128 {
    hash128_x64_with_seed(v, 0)
}

#[inline(always)]
fn get_u64(data: &[u8], i: usize) -> u64 {
    let buf = [
            data[i],
            data[i+1],
            data[i+2],
            data[i+3],
            data[i+4],
            data[i+5],
            data[i+6],
            data[i+7],
            ];
    u64::from_le_bytes(buf)
}

#[inline(always)]
fn fmix64(h: u64) -> u64 {
    let mut input = h;
    input ^= input >> 33;
    input = input.wrapping_mul(0xff51afd7ed558ccd);
    input ^= input >> 33;
    input = input.wrapping_mul(0xc4ceb9fe1a85ec53);
    input ^= input >> 33;
    input
}

#[cfg(test)]
mod test {
    #[test]
    fn compliance_test() {
        assert_eq!(
        crate::murmur::murmur3_128_64::hash128_x64_with_seed("StandardBlockEntityDescriptorV4", 4919),
        0xb15ad2fb6e6b679225e57206d95bdb79);
    }
}
