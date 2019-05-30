#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::io::{Error, Read, Write};
use std::ops::{BitXor, BitXorAssign};
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct Minisketch {
    inner: *mut minisketch,
}

impl Minisketch {
    pub fn try_new(bits: u32, implementation: u32, capacity: usize) -> Result<Self, ()> {
        let inner = unsafe { minisketch_create(bits, implementation, capacity) };

        if inner != std::ptr::null_mut() {
            Ok(Minisketch { inner })
        } else {
            Err(())
        }
    }

    fn new_from_opaque(inner: *mut minisketch) -> Self {
        Minisketch { inner }
    }

    pub fn bits(&self) -> u32 {
        unsafe { minisketch_bits(self.inner) }
    }

    pub fn capacity(&self) -> usize {
        unsafe { minisketch_capacity(self.inner) }
    }

    pub fn serialized_size(&self) -> usize {
        unsafe { minisketch_serialized_size(self.inner) }
    }

    pub fn add(&mut self, element: u64) {
        unsafe { minisketch_add_uint64(self.inner, element) }
    }

    pub fn merge(&mut self, other: &Self) -> Result<usize, ()> {
        let capacity = unsafe { minisketch_merge(self.inner, other.inner) };

        if capacity == 0 {
            Err(())
        } else {
            Ok(capacity)
        }
    }

    pub fn decode(&self, elements: &mut [u64]) -> Result<usize, ()> {
        let result =
            unsafe { minisketch_decode(self.inner, elements.len(), elements.as_mut_ptr()) };

        if result == -1 {
            Err(())
        } else {
            Ok(result as usize)
        }
    }
}

impl Read for Minisketch {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let size = self.serialized_size();
        unsafe { minisketch_serialize(self.inner, buf.as_mut_ptr()) }

        Ok(size)
    }
}

impl Write for Minisketch {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        unsafe { minisketch_deserialize(self.inner, buf.as_ptr()) }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl Drop for Minisketch {
    fn drop(&mut self) {
        unsafe {
            minisketch_destroy(self.inner);
        }
    }
}

impl Clone for Minisketch {
    fn clone(&self) -> Self {
        let inner = unsafe { minisketch_clone(self.inner) };
        Self::new_from_opaque(inner)
    }
}

impl BitXorAssign for Minisketch {
    fn bitxor_assign(&mut self, rhs: Minisketch) {
        let _ = self.merge(&rhs);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn validate_elements(elements: &[u64]) {
        // Sort differences since they're come in arbitrary order from minisketch_decode()
        let mut differences = Vec::from(elements);
        differences.sort();

        assert_eq!(differences[0], 3_000);
        assert_eq!(differences[1], 3_001);
        assert_eq!(differences[2], 3_010);
        assert_eq!(differences[3], 3_011);
    }

    #[test]
    // Implement an example from minisketch's README
    pub fn test_sanity_check() {
        unsafe {
            // Alice's side
            let sketch_a = minisketch_create(12, 0, 4);
            for i in 3_000..3_010 {
                minisketch_add_uint64(sketch_a, i as u64);
            }

            let sersize = minisketch_serialized_size(sketch_a);
            assert_eq!(sersize, 12 * 4 / 8);

            let mut buf_a = Vec::with_capacity(sersize);
            minisketch_serialize(sketch_a, buf_a.as_mut_ptr());
            minisketch_destroy(sketch_a);

            let sketch_b = minisketch_create(12, 0, 4);
            for i in 3_002..3_012 {
                minisketch_add_uint64(sketch_b, i as u64);
            }

            // Bob's side
            {
                let sketch_a = minisketch_create(12, 0, 4); // Alice's sketch
                minisketch_deserialize(sketch_a, buf_a.as_ptr()); // Load Alice's sketch

                // Merge the elements from sketch_a into sketch_b. The result is a sketch_b
                // which contains all elements that occurred in Alice's or Bob's sets, but not
                // in both.
                minisketch_merge(sketch_b, sketch_a);

                let mut differences = [0u64; 4];
                let num_differences = minisketch_decode(sketch_b, 4, differences.as_mut_ptr());
                minisketch_destroy(sketch_a);
                minisketch_destroy(sketch_b);

                assert!(num_differences > 0);
                validate_elements(&differences[..]);
            }
        };
    }

    #[test]
    // Implement a README's example with Rust API as a sanity check
    pub fn sanity_check_rust_types() {
        let mut sketch_a = Minisketch::try_new(12, 0, 4).unwrap();
        for i in 3_000..3_010 {
            sketch_a.add(i);
        }

        let sersize = sketch_a.serialized_size();
        assert_eq!(sersize, 12 * 4 / 8);

        let mut buf_a = Vec::with_capacity(sersize);
        sketch_a.read(&mut buf_a).unwrap();

        let mut sketch_b = Minisketch::try_new(12, 0, 4).unwrap();
        for i in 3_002..3_012 {
            sketch_b.add(i);
        }

        // Bob's side (with .merge() method)
        {
            let mut sketch_b = sketch_b.clone();
            // Alice's sketch
            let mut sketch_a = Minisketch::try_new(12, 0, 4).unwrap();
            sketch_a.write(&buf_a).unwrap(); // Load Alice's sketch

            // Merge the elements from sketch_a into sketch_b. The result is a sketch_b
            // which contains all elements that occurred in Alice's or Bob's sets, but not
            // in both.
            sketch_b.merge(&sketch_a).unwrap();

            let mut differences = [0u64; 4];
            let num_differences = sketch_b.decode(&mut differences[..]).unwrap();

            assert!(num_differences > 0);
            validate_elements(&differences[..]);
        }

        // Bob's side (with ^= instead of .merge())
        {
            let mut sketch_b = sketch_b.clone();

            // Alice's sketch
            let mut sketch_a = Minisketch::try_new(12, 0, 4).unwrap();
            sketch_a.write(&buf_a).unwrap(); // Load Alice's sketch

            // Merge the elements from sketch_a into sketch_b. The result is a sketch_b
            // which contains all elements that occurred in Alice's or Bob's sets, but not
            // in both.
            sketch_b ^= sketch_a;

            let mut differences = [0u64; 4];
            let num_differences = sketch_b.decode(&mut differences[..]).unwrap();

            assert!(num_differences > 0);
            validate_elements(&differences[..]);
        }
    }
}
