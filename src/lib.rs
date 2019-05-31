#![deny(warnings)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unused_results)]
#![deny(dead_code)]
#![doc(html_root_url = "https://docs.rs/minisketch_rs/0.1.0")]

//! # minisketch-rs
//!
//! `minisketch-rs` is a wrapper around [minisketch],
//! a C++ library by [Pieter Wuille] for efficient set reconciliation.
//!
//! Minisketch is proposed as part of an [Erlay] technique for bandwidth-efficient TX propagation in Bitcoin.
//!
//! This library exposes type-safe Rust bindings for all minisketch functions by providing [`Minisketch`] structure.
//!
//! ## Example
//!
//! Cargo.toml:
//! ```toml
//! [dependencies]
//! minisketch-rs = "0.1"
//! ```
//!
//! Example of simple set reconciliation between Alice and Bob:
//! ```edition2018
//! # pub fn main() -> Result<(), ()> {
//! use minisketch_rs::Minisketch;
//!
//! // Alice's side
//! let mut sketch_a = Minisketch::try_new(12, 0, 4)?;
//!
//! println!("Alice's set:");
//! for i in 3_000..3_010 {
//!     println!("{}", i);
//!     sketch_a.add(i);
//! }
//!
//! let sersize = sketch_a.serialized_size();
//! assert_eq!(sersize, 12 * 4 / 8);
//!
//! // Serialize message for Bob
//! let mut buf_a = vec![0u8; sersize];
//! sketch_a.serialize(buf_a.as_mut_slice())?;
//!
//! println!("Message: {}, {:?}", buf_a.len(), buf_a);
//!
//! // Bob's side
//! {
//!     // Bob's sketch
//!     println!("Bob's set:");
//!     let mut sketch_b = Minisketch::try_new(12, 0, 4)?;
//!     for i in 3_002..3_012 {
//!         println!("{}", i);
//!         sketch_b.add(i);
//!     }
//!
//!     // Alice's sketch
//!     let mut sketch_a = Minisketch::try_new(12, 0, 4)?;
//!     sketch_a.deserialize(&buf_a); // Load Alice's sketch
//!
//!     // Merge the elements from sketch_a into sketch_b. The result is a sketch_b
//!     // which contains all elements that occurred in Alice's or Bob's sets, but not
//!     // in both.
//!     sketch_b.merge(&sketch_a)?;
//!
//!     let mut differences = [0u64; 4];
//!     let num_differences = sketch_b.decode(&mut differences[..])?;
//!
//!     println!("Differences between Alice and Bob: {}", num_differences);
//!
//!     assert!(num_differences > 0);
//!
//!     // Sort differences since they may come in arbitrary order from minisketch_decode()
//!     let mut differences = Vec::from(&differences[..]);
//!     differences.sort();
//!
//!     for (i, diff) in differences.iter().enumerate() {
//!         println!("Difference #{}: {}", (i + 1), diff);
//!     }
//!
//!     assert_eq!(differences[0], 3_000);
//!     assert_eq!(differences[1], 3_001);
//!     assert_eq!(differences[2], 3_010);
//!     assert_eq!(differences[3], 3_011);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Code snippet above will print:
//!
//! ```norust
//! Alice's set:
//! 3000
//! 3001
//! 3002
//! 3003
//! 3004
//! 3005
//! 3006
//! 3007
//! 3008
//! 3009
//! Message: 6, [1, 224, 210, 249, 116, 105]
//! Bob's set:
//! 3002
//! 3003
//! 3004
//! 3005
//! 3006
//! 3007
//! 3008
//! 3009
//! 3010
//! 3011
//! Differences between Alice and Bob: 4
//! Difference #1: 3000
//! Difference #2: 3001
//! Difference #3: 3010
//! Difference #4: 3011
//! ```
//!
//! [minisketch]: https://github.com/sipa/minisketch
//! [`Minisketch`]: struct.Minisketch.html
//! [Pieter Wuille]: https://github.com/sipa
//! [Erlay]: https://arxiv.org/abs/1905.10518

use std::fmt::{Debug, Error, Formatter};
use std::ops::BitXorAssign;

#[doc(hidden)]
mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// Describes decoded sketches and holding underlying opaque type inside.
pub struct Minisketch {
    inner: *mut ffi::minisketch,
    bits: u32,
    implementation: u32,
    capacity: usize,
}

impl Minisketch {
    /// Tries to create a new empty sketch.
    ///
    /// # Errors
    ///
    /// If the combination of `bits` and `implementation` is unavailable, or if
    /// `capacity` is 0, an `Err(())` is returned.
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// # pub fn main() -> Result<(), ()> {
    /// use minisketch_rs::Minisketch;
    /// let sketch = Minisketch::try_new(12, 0, 4)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_new(bits: u32, implementation: u32, capacity: usize) -> Result<Self, ()> {
        let inner = unsafe { ffi::minisketch_create(bits, implementation, capacity) };

        if inner != std::ptr::null_mut() {
            Ok(Minisketch {
                inner,
                bits,
                implementation,
                capacity,
            })
        } else {
            Err(())
        }
    }

    /// Returns element size in a sketch in bits.
    pub fn bits(&self) -> u32 {
        unsafe { ffi::minisketch_bits(self.inner) }
    }

    /// Returns capacity of a sketch in number of elements.
    pub fn capacity(&self) -> usize {
        unsafe { ffi::minisketch_capacity(self.inner) }
    }

    /// Returns implementation version number.
    pub fn implementation(&self) -> u32 {
        unsafe { ffi::minisketch_implementation(self.inner) }
    }

    /// Returns the size in bytes for serializing a given sketch.
    pub fn serialized_size(&self) -> usize {
        unsafe { ffi::minisketch_serialized_size(self.inner) }
    }

    /// Adds a `u64` element to a sketch.
    ///
    /// If the element to be added is too large for the sketch, the most significant
    /// bits of the element are dropped. More precisely, if the element size of
    /// `sketch` is b bits, then this function adds the unsigned integer represented
    /// by the b least significant bits of `element` to `sketch`.
    ///
    /// If the element to be added is 0 (after potentially dropping the most significant
    /// bits), then this function is a no-op. Sketches cannot contain an element with
    /// the value 0.
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// # pub fn main() -> Result<(), ()> {
    /// use minisketch_rs::Minisketch;
    /// let mut sketch = Minisketch::try_new(12, 0, 4)?;
    /// sketch.add(42);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn add(&mut self, element: u64) {
        unsafe { ffi::minisketch_add_uint64(self.inner, element) }
    }

    /// Set the seed for randomizing algorithm choices to a fixed value.
    ///
    /// By default, sketches are initialized with a random seed. This is important
    /// to avoid scenarios where an attacker could force worst-case behavior.
    ///
    /// This function initializes the seed to a user-provided value (any 64-bit
    /// integer is acceptable, regardless of field size).
    ///
    /// When seed is `std::u64::MAX`, a fixed internal value with predictable behavior is used.
    /// It is only intended for testing.
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// # pub fn main() -> Result<(), ()> {
    /// use minisketch_rs::Minisketch;
    /// let mut sketch = Minisketch::try_new(12, 0, 4)?;
    /// sketch.set_seed(42);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_seed(&mut self, seed: u64) {
        unsafe { ffi::minisketch_set_seed(self.inner, seed) }
    }

    /// Merge the elements of another sketch into this sketch.
    ///
    /// After merging, `sketch` will contain every element that existed in one but not
    /// both of the input sketches. It can be seen as an exclusive or operation on
    /// the set elements.  If the capacity of `other_sketch` is lower than `sketch`'s,
    /// merging reduces the capacity of `sketch` to that of `other_sketch`.
    ///
    /// Returns the `Ok(capacity)` of `sketch` after merging has been performed (where this capacity
    /// is at least 1)
    ///
    /// It is also possible to perform this operation directly on the serializations
    /// of two sketches with the same element size and capacity by performing a bitwise XOR
    /// of the serializations.
    ///
    /// You can also merge two sketches by doing xor-assignment (`^=`).
    ///
    /// # Errors
    ///
    /// Returns `Err(())` to indicate that merging has failed
    /// because the two input sketches differ in their element size or implementation. If `Err` is
    /// returned, `sketch` (and its capacity) have not been modified.
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// # pub fn main() -> Result<(), ()> {
    /// use minisketch_rs::Minisketch;
    /// let mut sketch_a = Minisketch::try_new(12, 0, 4)?;
    /// sketch_a.add(10);
    /// sketch_a.add(43);
    ///
    /// let mut sketch_b = Minisketch::try_new(12, 0, 4)?;
    /// sketch_b.add(42);
    /// sketch_b.add(43);
    ///
    /// // Merge two sketches and extract a difference
    /// let num_diffs = sketch_a.merge(&sketch_b)?;
    ///
    /// // Extract difference
    /// let mut differences = vec![0u64; num_diffs];
    /// sketch_a.decode(&mut differences)?;
    ///
    /// assert!((differences[0] == 42 || differences[0] == 10) && (differences[1] == 10 || differences[1] == 42));
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn merge(&mut self, other: &Self) -> Result<usize, ()> {
        let capacity = unsafe { ffi::minisketch_merge(self.inner, other.inner) };

        if capacity == 0 {
            Err(())
        } else {
            Ok(capacity)
        }
    }

    /// Decode a sketch.
    ///
    /// `elements` is a mutable reference to a buffer of `u64`, which will be filled with the
    /// elements in this sketch.
    ///
    /// Returns `Ok(num. of decoded elements)`
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if decoding failed for any reason.
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// # pub fn main() -> Result<(), ()> {
    /// use minisketch_rs::Minisketch;
    /// let mut sketch = Minisketch::try_new(12, 0, 2)?;
    /// sketch.add(42);
    /// sketch.add(10);
    /// let mut elements = [0u64; 2];
    /// sketch.decode(&mut elements)?;
    ///
    /// // Elements may come in arbitrary order, so check all possible variants
    /// assert!((elements[0] == 42 || elements[0] == 10) && (elements[1] == 10 || elements[1] == 42));
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode(&self, elements: &mut [u64]) -> Result<usize, ()> {
        let result =
            unsafe { ffi::minisketch_decode(self.inner, elements.len(), elements.as_mut_ptr()) };

        if result == -1 {
            Err(())
        } else {
            Ok(result as usize)
        }
    }

    /// Deserialize a sketch from bytes.
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// # pub fn main() -> Result<(), ()> {
    /// use minisketch_rs::Minisketch;
    ///
    /// // Create Alice's sketch
    /// let mut sketch_alice = Minisketch::try_new(12, 0, 2)?;
    /// sketch_alice.add(42);
    /// sketch_alice.add(10);
    ///
    /// // Serialize sketch on Alice's side
    /// let mut message = vec![0u8; sketch_alice.serialized_size()];
    /// sketch_alice.serialize(&mut message);
    ///
    /// // ... message is sent from Alice to Bob ...
    ///
    /// // Deserialize sketch from Alice on Bob's side
    /// let mut sketch_bob = Minisketch::try_new(12, 0, 2)?;
    /// sketch_bob.deserialize(&message);
    ///
    /// // Decode received sketch
    /// let mut elements = [0u64; 2];
    /// sketch_bob.decode(&mut elements)?;
    /// // Elements may come in arbitrary order, so check all possible variants
    /// assert!((elements[0] == 42 || elements[0] == 10) && (elements[1] == 10 || elements[1] == 42));
    /// # Ok(())
    /// # }
    /// ```
    pub fn deserialize(&mut self, buf: &[u8]) {
        unsafe { ffi::minisketch_deserialize(self.inner, buf.as_ptr()) }
    }

    /// Serialize a sketch to bytes.
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if `.len()` of the provided buffer `buf` is less than a size in bytes of
    /// the serialized representation of the sketch.
    ///
    /// # Examples
    ///
    /// ```edition2018
    /// # pub fn main() -> Result<(), ()> {
    /// use minisketch_rs::Minisketch;
    /// let mut sketch = Minisketch::try_new(12, 0, 2)?;
    /// sketch.add(42);
    /// sketch.add(10);
    ///
    /// let mut buf = vec![0u8; sketch.serialized_size()];
    /// sketch.serialize(&mut buf);
    /// # Ok(())
    /// # }
    /// ```
    pub fn serialize(&self, buf: &mut [u8]) -> Result<(), ()> {
        let size = self.serialized_size();

        if size < buf.len() {
            return Err(());
        }

        unsafe { ffi::minisketch_serialize(self.inner, buf.as_mut_ptr()) }
        Ok(())
    }
}

/// Custom `Debug` implementation that shows basic information about opaque `minisketch`.
impl Debug for Minisketch {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Minisketch {{ bits = {}, implementation = {}, capacity = {} }}",
            self.bits(),
            self.implementation(),
            self.capacity(),
        )
    }
}

/// Custom `Drop` implementation that frees an underlying opaque sketch.
#[doc(hidden)]
impl Drop for Minisketch {
    fn drop(&mut self) {
        unsafe {
            ffi::minisketch_destroy(self.inner);
        }
    }
}

/// Custom `Clone` implementation that clones an underlying opaque sketch.
#[doc(hidden)]
impl Clone for Minisketch {
    fn clone(&self) -> Self {
        let inner = unsafe { ffi::minisketch_clone(self.inner) };

        Minisketch {
            inner,
            bits: self.bits,
            implementation: self.implementation,
            capacity: self.capacity,
        }
    }
}

/// Custom `^=` operator implementation on two sketches that performs merging.
///
/// # Example
///
/// ```edition2018
/// # pub fn main() -> Result<(), ()> {
/// use minisketch_rs::Minisketch;
/// let mut sketch_a = Minisketch::try_new(12, 0, 4)?;
/// sketch_a.add(10);
/// sketch_a.add(43);
///
/// let mut sketch_b = Minisketch::try_new(12, 0, 4)?;
/// sketch_b.add(42);
/// sketch_b.add(43);
///
/// // Merge two sketches with ^= operator
/// sketch_a ^= sketch_b;
///
/// // Extract difference
/// let mut differences = vec![0u64; 2];
/// sketch_a.decode(&mut differences)?;
///
/// assert!((differences[0] == 42 || differences[0] == 10) && (differences[1] == 10 || differences[1] == 42));
///
/// # Ok(())
/// # }
/// ```
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
        use ffi::*;
        unsafe {
            // Alice's side
            let sketch_a = minisketch_create(12, 0, 4);
            for i in 3_000..3_010 {
                minisketch_add_uint64(sketch_a, i as u64);
            }

            let sersize = minisketch_serialized_size(sketch_a);
            assert_eq!(sersize, 12 * 4 / 8);

            let mut buf_a = vec![0u8; sersize];
            minisketch_serialize(sketch_a, buf_a.as_mut_slice().as_mut_ptr());
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
                let _ = minisketch_merge(sketch_b, sketch_a);

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

        assert_eq!(sketch_a.bits(), 12);
        assert_eq!(sketch_a.implementation(), 0);
        assert_eq!(sketch_a.capacity(), 4);

        for i in 3_000..3_010 {
            sketch_a.add(i);
        }

        let sersize = sketch_a.serialized_size();
        assert_eq!(sersize, 12 * 4 / 8);

        let mut buf_a = vec![0u8; sersize];
        sketch_a.serialize(buf_a.as_mut_slice()).unwrap();

        let mut sketch_b = Minisketch::try_new(12, 0, 4).unwrap();
        for i in 3_002..3_012 {
            sketch_b.add(i);
        }

        // Bob's side (with .merge() method)
        {
            let mut sketch_b = sketch_b.clone();
            // Alice's sketch
            let mut sketch_a = Minisketch::try_new(12, 0, 4).unwrap();
            sketch_a.deserialize(&buf_a); // Load Alice's sketch

            // Merge the elements from sketch_a into sketch_b. The result is a sketch_b
            // which contains all elements that occurred in Alice's or Bob's sets, but not
            // in both.
            let _ = sketch_b.merge(&sketch_a).unwrap();

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
            sketch_a.deserialize(&buf_a); // Load Alice's sketch

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
