use std::hash::{BuildHasher, Hasher};

/// Hasher for City hash implementation of the 32-bit hashing algorithm.
#[derive(Default)]
pub struct CityHasher32 {
    buffer: Vec<u8>,
}

impl Hasher for CityHasher32 {
    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes)
    }

    fn finish(&self) -> u64 {
        hash32(&self.buffer) as u64
    }
}

/// Hash builder for City hash implementation of the 32-bit hashing algorithm.
#[derive(Default)]
pub struct CityHash32 {}

impl BuildHasher for CityHash32 {
    type Hasher = CityHasher32;

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
/*
const K0: u64 = 0xc3a5c85c97cb3127;
const K1: u64 = 0xb492b66fbe98f273;
const K2: u64 = 0x9ae16a3b2f90404f;
const K3: u64 = 0xc949d7c7509e6557;*/

// Magic numbers for 32-bit hashing.  Copied from Murmur3.
const C1: u32 = 0xcc9e2d51;
const C2: u32 = 0x1b873593;

const D0: u32 = 0xe6546b64;

/// City hash implementation of the 32-bit hashing algorithm.
/// This version allows you to specify a seed.
pub fn hash32_with_seed<T: AsRef<[u8]>>(v: T, seed: u32) -> u32 {
    let data = v.as_ref();
    if data.len() <= 24 {
        if data.len() <= 12 {
            if data.len() <= 4 {
                return hash32_len_0_to_4(data, seed);
            }
            return hash32_len_5_to_12(data, seed);
        }
        return hash32_len_13_to_24(data, seed);
    }
    let mut h = (data.len() as u32).wrapping_add(seed);
    let mut g = C1.wrapping_mul(data.len() as u32);
    let mut f = g;

    let mut a0 = fetch32(data, data.len() - 4)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let mut a1 = fetch32(data, data.len() - 8)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let mut a2 = fetch32(data, data.len() - 16)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let mut a3 = fetch32(data, data.len() - 12)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let mut a4 = fetch32(data, data.len() - 20)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);

    h ^= a0;
    h = h.rotate_right(19);
    h = h.wrapping_mul(5).wrapping_add(D0);

    h ^= a2;
    h = h.rotate_right(19);
    h = h.wrapping_mul(5).wrapping_add(D0);

    g ^= a1;
    g = g.rotate_right(19);
    g = g.wrapping_mul(5).wrapping_add(D0);

    g ^= a3;
    g = g.rotate_right(19);
    g = g.wrapping_mul(5).wrapping_add(D0);

    f = f.wrapping_add(a4);
    f = f.rotate_right(19);
    f = f.wrapping_mul(5).wrapping_add(D0);

    for i in 0..(data.len() - 1) / 20 {
        a0 = fetch32(data, (i * 20) + 0)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        a1 = fetch32(data, (i * 20) + 4);
        a2 = fetch32(data, (i * 20) + 8)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        a3 = fetch32(data, (i * 20) + 12)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        a4 = fetch32(data, (i * 20) + 16);

        h ^= a0;
        h = h.rotate_right(18).wrapping_mul(5).wrapping_add(D0);

        f = f.wrapping_add(a1).rotate_right(19).wrapping_mul(C1);

        g = g
            .wrapping_add(a2)
            .rotate_right(18)
            .wrapping_mul(5)
            .wrapping_add(D0);

        h ^= a3.wrapping_add(a1);
        h = h.rotate_right(19).wrapping_mul(5).wrapping_add(D0);

        g ^= a4;
        g = bswap32(g).wrapping_mul(5);

        h = h.wrapping_add(a4.wrapping_mul(5));
        h = bswap32(h);

        f = f.wrapping_add(a0);

        permute3(&mut f, &mut h, &mut g);
    }

    g = g
        .rotate_right(11)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C1);
    f = f
        .rotate_right(11)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C1);
    h = h
        .wrapping_add(g)
        .rotate_right(19)
        .wrapping_mul(5)
        .wrapping_add(D0);
    h = h
        .rotate_right(17)
        .wrapping_mul(C1)
        .wrapping_add(f)
        .rotate_right(19);
    h = h
        .wrapping_mul(5)
        .wrapping_add(D0)
        .rotate_right(17)
        .wrapping_mul(C1);
    h
}

/// City hash implementation of the 32-bit hashing algorithm.
/// This version has the seed preset to 0
pub fn hash32<T: AsRef<[u8]>>(v: T) -> u32 {
    hash32_with_seed(v, 0)
}

#[inline(always)]
fn bswap32(h: u32) -> u32 {
    u32::from_be_bytes(h.to_le_bytes())
}

#[inline(always)]
fn fmix32(h: u32) -> u32 {
    let mut input = h;
    input ^= input >> 16;
    input = input.wrapping_mul(0x85ebca6b);
    input ^= input >> 13;
    input = input.wrapping_mul(0xC2b2ae35);
    input ^= input >> 16;
    input
}

#[inline(always)]
fn mur_combine(mut a: u32, mut h: u32) -> u32 {
    // Helper from Murmur3 for combining two 32-bit values.
    a = a.wrapping_mul(C1);
    a = a.rotate_right(17);
    a = a.wrapping_mul(C2);
    h ^= a;
    h = h.rotate_right(19);
    h.wrapping_mul(5).wrapping_add(0xe6546b64)
}

// from murmur3 get_u32()
#[inline(always)]
fn fetch32(data: &[u8], i: usize) -> u32 {
    let buf = [data[i], data[i + 1], data[i + 2], data[i + 3]];
    u32::from_le_bytes(buf)
}

#[inline(always)]
fn hash32_len_0_to_4(data: &[u8], seed: u32) -> u32 {
    let mut b = seed;
    let mut c = 9;
    for i in 0..data.len() {
        b = b.wrapping_mul(C1).wrapping_add(data[i] as u32);
        c ^= b;
    }
    fmix32(mur_combine(b, mur_combine(data.len() as u32, c)))
}

#[inline(always)]
fn hash32_len_5_to_12(data: &[u8], seed: u32) -> u32 {
    let mut a = (data.len() as u32).wrapping_add(seed);
    let mut b = (data.len() as u32) * 5;
    let mut c: u32 = 9;
    let d = b;
    a = a.wrapping_add(fetch32(data, 0));
    b = b.wrapping_add(fetch32(data, data.len() - 4));
    c = c.wrapping_add(fetch32(data, (data.len() >> 1) & 4));
    fmix32(mur_combine(c, mur_combine(b, mur_combine(a, d))))
}

#[inline(always)]
fn hash32_len_13_to_24(data: &[u8], seed: u32) -> u32 {
    let h = seed.wrapping_add(data.len() as u32);
    let a = fetch32(data, (data.len() >> 1) - 4);
    let b = fetch32(data, 4);
    let c = fetch32(data, data.len() - 8);
    let d = fetch32(data, data.len() >> 1);
    let e = fetch32(data, 0);
    let f = fetch32(data, data.len() - 4);
    fmix32(mur_combine(
        f,
        mur_combine(
            e,
            mur_combine(d, mur_combine(c, mur_combine(b, mur_combine(a, h)))),
        ),
    ))
}

#[inline(always)]
fn permute3(a: &mut u32, b: &mut u32, c: &mut u32) {
    std::mem::swap(a, b);
    std::mem::swap(a,c);
}

#[cfg(test)]
mod test {
    #[test]
    fn compliance_test() {
        assert_eq!(crate::city::city_32::hash32_with_seed("abc", 0), 795041479);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn fasthash_interop_test() {
        let input = "This is a very long test string to make sure this project produces the same results as fasthash";
        let seed = 0;
        //println!("Input: '{}' ({}) seed: {}", input, input.len(), seed);
        assert_eq!(crate::city::city_32::hash32_with_seed(input, seed),
            fasthash::city::hash32_with_seed(input, seed));
    }
}
