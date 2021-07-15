use std::hash::{BuildHasher, Hasher};

/// Hasher for City hash implementation of the 64-bit hashing algorithm.
#[derive(Default)]
pub struct CityHasher64 {
    buffer: Vec<u8>,
}

impl Hasher for CityHasher64 {
    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes)
    }

    fn finish(&self) -> u64 {
        hash64(&self.buffer)
    }
}

/// Hash builder for City hash implementation of the 64-bit hashing algorithm.
#[derive(Default)]
pub struct CityHash64 {}

impl BuildHasher for CityHash64 {
    type Hasher = CityHasher64;

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

/// City hash implementation of the 64-bit hashing algorithm.
/// This version allows you to specify one seed.
pub fn hash64_with_seed<T: AsRef<[u8]>>(v: T, seed: u64) -> u64 {
    hash64_with_seeds(v, K2, seed)
}

/// City hash implementation of the 64-bit hashing algorithm.
/// This version allows you to specify two seed.
pub fn hash64_with_seeds<T: AsRef<[u8]>>(v: T, seed0: u64, seed1: u64) -> u64 {
    hash_len_16(hash64(v).wrapping_sub(seed0), seed1)
}

/// City hash implementation of the 64-bit hashing algorithm.
/// This version has no seed
pub fn hash64<T: AsRef<[u8]>>(v: T) -> u64 {
    let data = v.as_ref();
    if data.len() <= 32 {
        if data.len() <= 16 {
            return hash64_len_0_to_16(data);
        }
        return hash64_len_17_to_32(data);
    } else if data.len() <= 64 {
        return hash64_len_33_to_64(data);
    }
    
    // >= 64 bytes
    // keep 56 bytes of state across loops
    let mut x = fetch64(data, data.len() - 40);
    let mut y = fetch64(data, data.len() - 16).wrapping_add(fetch64(data, data.len() - 56));
    let mut z = hash_len_16(fetch64(data, data.len() - 48).wrapping_add(data.len() as u64), fetch64(data, data.len() - 24));
    let mut v = weak_hash_len_32_with_seeds(data, data.len() as u64, z, data.len() - 64);
    let mut w = weak_hash_len_32_with_seeds(data, y.wrapping_add(K1), x, data.len() - 32);
    x = x.wrapping_mul(K1).wrapping_add(fetch64(data, 0));
    
    // Decrease len to the nearest multiple of 64, and operate on 64-byte chunks.
    let mut len = (data.len() - 1) & !63;
    let mut s = 0; // data index

    println!("x:{} y:{} z:{} v:({}, {}) w:({}, {})", x, y, z, v.0, v.1, w.0, w.1);
    
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
        len -= 64;
        println!("x:{} y:{} z:{} v:({}, {}) w:({}, {}) len:{}", x, y, z, v.0, v.1, w.0, w.1, len);
        if len == 0 { break; }
    }
    hash_len_16(
        hash_len_16(v.0, w.0).wrapping_add(shift_mix(y).wrapping_mul(K1)).wrapping_add(z),
        hash_len_16(v.1, w.1).wrapping_add(x))
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
    println!("val:{}", val);
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
        println!("a:{} b:{} c:{} y:{} z:{}", a, b, c, y, z);
        return shift_mix((y as u64).wrapping_mul(K2) ^ (z as u64).wrapping_mul(K3)).wrapping_mul(K2);
    }
    K2
}

#[inline(always)]
fn hash64_len_17_to_32(data: &[u8]) -> u64 {
    let a = fetch64(data, 0).wrapping_mul(K1);
    let b = fetch64(data, 8);
    let c = fetch64(data, data.len() - 8).wrapping_mul(K2);
    let d = fetch64(data, data.len() - 16).wrapping_mul(K0);
    let u = a.wrapping_sub(b).rotate_right(43).wrapping_add(c.rotate_right(30)).wrapping_add(d);
    let v = a.wrapping_add((b ^ K3).rotate_right(20)).wrapping_sub(c).wrapping_add(data.len() as u64);
    hash_len_16(u, v)
}

#[inline(always)]
fn hash64_len_33_to_64(data: &[u8]) -> u64 {
    let mut z = fetch64(data, 24);
    let mut a = fetch64(data, 0).wrapping_add((data.len() as u64).wrapping_add(fetch64(data, data.len() - 16)).wrapping_mul(K0));
    let mut b = a.wrapping_add(z).rotate_right(52);
    let mut c = a.rotate_right(37);
    a = a.wrapping_add(fetch64(data, 8));
    c = c.wrapping_add(a.rotate_right(7));
    a = a.wrapping_add(fetch64(data, 16));
    let vf = a.wrapping_add(z);
    let vs = b.wrapping_add(a.rotate_right(31)).wrapping_add(c);
    a = fetch64(data, 16).wrapping_add(fetch64(data, data.len() - 32));
    z = fetch64(data, data.len() - 8);
    b = a.wrapping_add(z).rotate_right(52);
    c = a.rotate_right(37);
    a = a.wrapping_add(fetch64(data, data.len() - 24));
    c = c.wrapping_add(a.rotate_right(7));
    a = a.wrapping_add(fetch64(data, data.len() - 16));
    let wf = a.wrapping_add(z);
    let ws = b.wrapping_add(a.rotate_right(31)).wrapping_add(c);
    let r = shift_mix(vf.wrapping_add(ws).wrapping_mul(K2).wrapping_add(wf.wrapping_add(vs).wrapping_mul(K0)));
    shift_mix(r.wrapping_mul(K0).wrapping_add(vs)).wrapping_mul(K2)
}

#[cfg(test)]
mod test {
    #[test]
    fn compliance_test() {
        assert_eq!(crate::city::city_64::hash64("abc"), 4220206313085259313);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn fasthash_interop_test() {
        let input = "This is a very long test string to make sure this project produces the same results as fasthash";
        let seed1 = 0;
        let seed2 = 0;
        //println!("Input: '{}' ({}) seed: {}", input, input.len(), seed);
        assert_eq!(crate::city::city_64::hash64_with_seeds(input, seed1, seed2),
            fasthash::city::hash64_with_seeds(input, seed1, seed2));
    }
}

