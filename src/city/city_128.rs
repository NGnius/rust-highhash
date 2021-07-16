use std::hash::{BuildHasher, Hasher};

/// Hasher for City hash implementation of the 64-bit hashing algorithm.
#[derive(Default)]
pub struct CityHasher128 {
    buffer: Vec<u8>,
}

impl Hasher for CityHasher128 {
    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes)
    }

    fn finish(&self) -> u64 {
        hash128(&self.buffer) as u64
    }
}

/// Hash builder for City hash implementation of the 64-bit hashing algorithm.
#[derive(Default)]
pub struct CityHash128 {}

impl BuildHasher for CityHash128 {
    type Hasher = CityHasher128;

    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::default()
    }
}

// The code below is adapted from C++ code with the following license
//-----------------------------------------------------------------------------
// Copyright (c) 2011 Google, Inc.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

// Some primes between 2^63 and 2^64 for various uses.

const K0: u64 = 0xc3a5c85c97cb3127;
const K1: u64 = 0xb492b66fbe98f273;
const K2: u64 = 0x9ae16a3b2f90404f;
const K3: u64 = 0xc949d7c7509e6557;

const K_MUL: u64 = 0x9ddfea08eb382d69;

/// City hash implementation of the 128-bit hashing algorithm.
/// This version allows you to specify a seed.
pub fn hash128_with_seed<T: AsRef<[u8]>>(v: T, seed: u128) -> u128 {
    let data = v.as_ref();
    if data.len() < 128 {
        return city_murmur(data, seed);
    }
    // >= 128 bytes
    // keep 56 bytes of state across loops
    let mut v = (0, 0);
    let mut w = (0, 0);
    let mut x = seed as u64;
    let mut y = (seed >> 64) as u64;
    let mut z = (data.len() as u64).wrapping_mul(K1);
    v.0 = (y ^ K1).rotate_right(47).wrapping_mul(K1).wrapping_add(fetch64(data, 0));
    v.1 = v.0.rotate_right(42).wrapping_mul(K1).wrapping_add(fetch64(data, 8));
    w.0 = y.wrapping_add(z).rotate_right(35).wrapping_mul(K1).wrapping_add(x);
    w.1 = x.wrapping_add(fetch64(data, 88)).rotate_right(53).wrapping_mul(K1);
    
    let mut len = data.len();
    let mut s = 0; // data index
    
    // similar to city hash64 loop
    loop {
        x = x.wrapping_add(y).wrapping_add(v.0).wrapping_add(fetch64(data, s + 8)).rotate_right(37).wrapping_mul(K1);
        y = y.wrapping_add(v.1).wrapping_add(fetch64(data, s + 48)).rotate_right(42).wrapping_mul(K1);
        x ^= w.1;
        y = y.wrapping_add(v.0).wrapping_add(fetch64(data, s + 40));
        z = z.wrapping_add(w.0).rotate_right(33).wrapping_mul(K1);
        v = weak_hash_len_32_with_seeds(data, v.1.wrapping_mul(K1), x.wrapping_add(w.0), s);
        w = weak_hash_len_32_with_seeds(data, z.wrapping_add(w.1), y.wrapping_add(fetch64(data, s + 16)), s + 32);
        std::mem::swap(&mut z, &mut x);
        s += 64;
        x = x.wrapping_add(y).wrapping_add(v.0).wrapping_add(fetch64(data, s + 8)).rotate_right(37).wrapping_mul(K1);
        y = y.wrapping_add(v.1).wrapping_add(fetch64(data, s + 48)).rotate_right(42).wrapping_mul(K1);
        x ^= w.1;
        y = y.wrapping_add(v.0).wrapping_add(fetch64(data, s + 40));
        z = z.wrapping_add(w.0).rotate_right(33).wrapping_mul(K1);
        v = weak_hash_len_32_with_seeds(data, v.1.wrapping_mul(K1), x.wrapping_add(w.0), s);
        w = weak_hash_len_32_with_seeds(data, z.wrapping_add(w.1), y.wrapping_add(fetch64(data, s + 16)), s + 32);
        std::mem::swap(&mut z, &mut x);
        len -= 128;
        if len < 128 { break; }
    }
    x = x.wrapping_add(v.0.wrapping_add(z).rotate_right(49).wrapping_mul(K0));
    z = x.wrapping_add(w.0.rotate_right(37).wrapping_mul(K0));
    // handle end of hash data
    // hash up to 4 32-bit chunks for tail section (up to 128 bits)
    let mut tail_done = 0;
    while tail_done < len {
        tail_done += 32;
        y = x.wrapping_add(y).rotate_right(42).wrapping_mul(K0).wrapping_add(v.1);
        w.0 = w.0.wrapping_add(fetch64(data, s + len - tail_done + 16));
        x = x.wrapping_mul(K0).wrapping_add(w.0);
        z = z.wrapping_add(w.1.wrapping_add(fetch64(data, s + len - tail_done)));
        w.1 = w.1.wrapping_add(v.0);
        v = weak_hash_len_32_with_seeds(data, v.0.wrapping_add(z), v.1, s + len - tail_done);
    }
    x = hash_len_16(x, v.0);
    y = hash_len_16(y.wrapping_add(z), w.0);
    
    let low: u64 = hash_len_16(x.wrapping_add(v.1), w.1).wrapping_add(y);
    let high: u64 = hash_len_16(x.wrapping_add(w.1), y.wrapping_add(v.1));
    //(high as u128) << 64 | (low as u128)
    to_u128(low, high)
}

/// City hash implementation of the 128-bit hashing algorithm.
/// This version generates a seed from the start of the data.
pub fn hash128<T: AsRef<[u8]>>(v: T) -> u128 {
    let data = v.as_ref();
    if data.len() >= 16 {
        let seed = to_u128(fetch64(data, 0) ^ K3, fetch64(data, 8));
        //let seed = (fetch64(data, 8) as u128) << 64 | ((fetch64(data, 0) ^ K3) as u128);
        return hash128_with_seed(&data[16..], seed);
    } else if data.len() >= 8 {
        let seed = to_u128(fetch64(data, 0) ^ (data.len() as u64).wrapping_mul(K0), (fetch64(data, data.len() - 8)) ^ K1);
        //let seed = ((fetch64(data, data.len() - 8) ^ K1) as u128) << 64 | ((fetch64(data, 0) ^ (data.len() as u64).wrapping_mul(K0)) as u128);
        return hash128_with_seed(&[], seed);
    }
    hash128_with_seed(data, to_u128(K0, K1))
}

#[inline(always)]
fn to_u128(a: u64, b: u64) -> u128 {
    (b as u128) << 64 | (a as u128)
}

#[inline(always)]
fn weak_hash_len_32_with_seeds(data: &[u8], mut a: u64, mut b: u64, index: usize) -> (u64, u64) {
    let w = fetch64(data, index);
    let x = fetch64(data, index + 8);
    let y = fetch64(data, index + 16);
    let z = fetch64(data, index + 24);
    a = a.wrapping_add(w);
    b = b.wrapping_add(a).wrapping_add(z).rotate_right(21);
    let c = a;
    a = a.wrapping_add(x).wrapping_add(y);
    b = b.wrapping_add(a.rotate_right(44));
    (a.wrapping_add(z), b.wrapping_add(c))
}

// like murmur3 get_u64()
#[inline(always)]
fn fetch64(data: &[u8], i: usize) -> u64 {
    let buf = [data[i], data[i + 1], data[i + 2], data[i + 3], data[i+4], data[i+5], data[i+6], data[i+7]];
    u64::from_le_bytes(buf)
}

// from murmur3 get_u32()
#[inline(always)]
fn fetch32(data: &[u8], i: usize) -> u32 {
    let buf = [data[i], data[i + 1], data[i + 2], data[i + 3]];
    u32::from_le_bytes(buf)
}

#[inline(always)]
fn shift_mix(val: u64) -> u64 {
    val ^ (val >> 47)
}

#[inline(always)]
fn hash_len_16(u: u64, v: u64) -> u64 {
    //let x = (v as u128) << 64 | (u as u128);
    let mut a = (v ^ u).wrapping_mul(K_MUL);
    a ^= a >> 47;
    let mut b = (v ^ a).wrapping_mul(K_MUL);
    b ^= b >> 47;
    b = b.wrapping_mul(K_MUL);
    b
}

#[inline(always)]
fn hash64_len_0_to_16(data: &[u8]) -> u64 {
    if data.len() > 8 {
        let a = fetch64(data, 0);
        let b = fetch64(data, data.len() - 8);
        return hash_len_16(a, b.wrapping_add(data.len() as u64).rotate_right(data.len() as u32)) ^ b;
    }
    if data.len() >= 4 {
        let a = fetch32(data, 0) as u64;
        return hash_len_16((data.len() as u64).wrapping_add(a << 3), fetch32(data, data.len() - 4) as u64);
    }
    if data.len() > 0 {
        let a = data[0];
        let b = data[data.len() >> 1];
        let c = data[data.len() - 1];
        let y = (a as u32).wrapping_add((b as u32) << 8);
        let z = (data.len() as u32).wrapping_add((c as u32) << 2);
        return shift_mix((y as u64).wrapping_mul(K2) ^ (z as u64).wrapping_mul(K3)).wrapping_mul(K2);
    }
    K2
}

#[inline(always)]
fn city_murmur(data: &[u8], seed: u128) -> u128 {
    let mut a = seed as u64;
    let mut b = (seed >> 64) as u64;
    let mut c: u64;
    let mut d: u64;
    let mut l = (data.len() as isize).wrapping_sub(16);
    if l <= 0 {
        a = shift_mix(a.wrapping_mul(K1)).wrapping_mul(K1);
        c = b.wrapping_mul(K1).wrapping_add(hash64_len_0_to_16(data));
        if data.len() >= 8 {
            d = shift_mix(a.wrapping_add(fetch64(data, 0)));
        } else {
            d = shift_mix(a.wrapping_add(c));
        }
    } else { // data.len() > 16
        c = hash_len_16(fetch64(data, data.len() - 8).wrapping_add(K1), a);
        d = hash_len_16(b.wrapping_add(data.len() as u64), c.wrapping_add(fetch64(data, data.len() - 16)));
        a = a.wrapping_add(d);
        let mut s = 0; // data index
        loop {
            a ^= shift_mix(fetch64(data, s).wrapping_mul(K1)).wrapping_mul(K1);
            a = a.wrapping_mul(K1);
            b ^= a;
            c ^= shift_mix(fetch64(data, s + 8).wrapping_mul(K1)).wrapping_mul(K1);
            c = c.wrapping_mul(K1);
            d ^= c;
            s += 16;
            l -= 16;
            if l <= 0 { break; }
        }
    }
    a = hash_len_16(a, c);
    b = hash_len_16(d, b);
    //(hash_len_16(b, a) as u128) << 64 | ((a ^ b) as u128)
    to_u128(a ^ b, hash_len_16(b, a))
}

#[cfg(test)]
mod test {
    #[test]
    fn compliance_test() {
        assert_eq!(crate::city::city_128::hash128("abc"), 26133304454536238711123707289922914558);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn fasthash_interop_test() {
        let input = "This is a very long test string to make sure this project produces the same results as fasthash";
        let seed = 0;
        assert_eq!(crate::city::city_128::hash128_with_seed(input, seed),
            fasthash::city::hash128_with_seed(input, seed));
    }
}


