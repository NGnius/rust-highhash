//! City string-hashing algorithm by Geoff Pike and Jyrki Alakuijala
mod city_128;
mod city_32;
mod city_64;

pub use city_32::{hash32, hash32_with_seed, CityHash32, CityHasher32};
