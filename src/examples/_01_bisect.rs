//! Example of set reconciliation that uses "bisection" approach when number of differences
//! between two sets is exceeding an estimate and simple reconciliation fails.
//!
//! Bisection leverages the "divide-and-conquer" approach. It is possible thanks to the linearity of
//! the set sketches.
//!
//! In following example Bob wants to reconcile his set with Alice:
//!
//! ```notrust
//! +-------+                            +-------+
//! |  Bob  |                            | Alice |
//! +-------+                            +-------+
//!     |                                   |
//!     | Sketch from a set [0..8]          |  Sketch from a set [0..32]
//!     |-------------------------------    |-------------------------------
//!     |                              |    |                              |
//!     |<------------------------------    |<------------------------------
//!     |                                   |
//!     | Serialized sketch (B)             |
//!     |---------------------------------->|
//!     |                                   |
//!     |                                   | Deserialize sketch
//!     |                                   |-------------------
//!     |                                   |                  |
//!     |                                   |<------------------
//!     |                                   |
//!     |                                   | reconcile(A, B)
//!     |                                   |-------------------------------
//!     |                                   |                              | failed!
//!     |                                   |<------------------------------
//!     |                                   |
//!     |      Ask for B/2 (bisect request) |
//!     |<----------------------------------|
//!     |                                   |
//!     | Sketch from B/2 [0, 2, 4, 6]      |
//!     |-----------------------------      |
//!     |                            |      |
//!     |<----------------------------      |
//!     |                                   |
//!     | Serialized sketch (B/2)           |
//!     |---------------------------------->|
//!     |                                   |
//!     |                                   | Deserialize sketch
//!     |                                   |-------------------
//!     |                                   |                  |
//!     |                                   |<------------------
//!     |                                   |
//!     |                                   | Compute: A/2, reconcile(A, A/2), reconcile(B, B/2)
//!     |                                   |---------------------------------------------------
//!     |                                   |                                                  |
//!     |                                   |<--------------------------------------------------
//!     |                                   |
//!     |                                   | Perform bisection:
//!     |                                   | diff1 = reconcile(A/2, B/2)
//!     |                                   | diff2 = reconcile(A - A/2, B - B/2)
//!     |                                   | bob_missing = diff1 ∪ diff2
//!     |                                   |-------------------------------------
//!     |                                   |                                    |
//!     |                                   |<------------------------------------
//!     |                                   |
//!     |                                   |
//!     |                  Send bob_missing |
//!     |<----------------------------------|
//!     |                                   |
//! ```
//!
//! ```
//! use minisketch_rs::Minisketch;
//! 
//! /// Extracts remainder sketch from a difference of two sketches
//! fn sub_sketches(s1: &[u8], s2: &[u8], d: usize, seed: Option<u64>) -> Vec<u8> {
//!     let mut a = create_minisketch(d, seed);
//!     if let Some(seed) = seed {
//!         a.set_seed(seed);
//!     }
//!     a.deserialize(s1);
//! 
//!     let mut b = create_minisketch(d, seed);
//!     if let Some(seed) = seed {
//!         b.set_seed(seed);
//!     }
//!     b.deserialize(s2);
//! 
//!     a.merge(&b).expect("Sketch sub merge");
//! 
//!     let mut sketch = vec![0u8; a.serialized_size()];
//!     a.serialize(&mut sketch).expect("Serialize sketch sub");
//! 
//!     sketch
//! }
//! 
//! /// Creates a_whole set from a_whole range of elements
//! fn sketch_from_range(
//!     range: impl IntoIterator<Item = u64>,
//!     capacity: usize,
//!     seed: Option<u64>,
//! ) -> Minisketch {
//!     let mut sketch = create_minisketch(capacity, seed);
//!     for i in range {
//!         sketch.add(i);
//!     }
//!     sketch
//! }
//! 
//! /// Creates `Minisketch` for given `capacity` and optional `seed`.
//! fn create_minisketch(capacity: usize, seed: Option<u64>) -> Minisketch {
//!     let mut minisketch = Minisketch::try_new(64, 0, capacity).unwrap();
//! 
//!     if let Some(seed) = seed {
//!         minisketch.set_seed(seed);
//!     }
//! 
//!     minisketch
//! }
//! 
//! /// Creates serialized sketch.
//! fn serialize_sketch(sketch: Minisketch) -> Vec<u8> {
//!     let mut buf = vec![0u8; sketch.serialized_size()];
//!     sketch.serialize(&mut buf).expect("Minisketch serialize");
//! 
//!     buf
//! }
//! 
//! /// Does set reconciliation from two sets.
//! fn reconcile(
//!     sketch_a: &[u8],
//!     sketch_b: &[u8],
//!     capacity: usize,
//!     seed: Option<u64>,
//! ) -> Result<Vec<u64>, ()> {
//!     let mut a = create_minisketch(capacity, seed);
//!     a.deserialize(sketch_a);
//! 
//!     let mut b = create_minisketch(capacity, seed);
//!     b.deserialize(sketch_b);
//! 
//!     a.merge(&b).expect("Minisketch merge");
//! 
//!     let mut diffs = vec![0u64; capacity];
//!     let num_diffs = a.decode(&mut diffs).map_err(|_| ())?;
//! 
//!     Ok(diffs.into_iter().take(num_diffs).collect())
//! }
//! 
//! fn example(capacity: usize) -> Result<Vec<u64>, ()> {
//!     let seed = None;
//! 
//!     // There is exactly 24 differences, but since capacity = 16, simple set reconciliation will fail
//!     let a = 0..32;
//!     let b = 0..8;
//! 
//!     // Count difference between two sets
//!     let set_diff = a.clone().into_iter().filter(|e| !b.contains(e)).count();
//! 
//!     println!(
//!         "Alice's set: {:?}",
//!         a.clone().into_iter().collect::<Vec<_>>()
//!     );
//!     println!("Bob's set: {:?}", b.clone().into_iter().collect::<Vec<_>>());
//! 
//!     // To increase chance of bisect success, take only even elements of the set,
//!     // so they're distributed uniformly.
//!     let b_half = b
//!         .clone()
//!         .into_iter()
//!         .enumerate()
//!         .filter(|(i, _)| *i % 2 == 0)
//!         .map(|(_, n)| n)
//!         .collect::<Vec<_>>();
//!     let a_half = a
//!         .clone()
//!         .into_iter()
//!         .enumerate()
//!         .filter(|(i, _)| *i % 2 == 0)
//!         .map(|(_, n)| n)
//!         .collect::<Vec<_>>();
//! 
//!     let alice_set_full = sketch_from_range(a, capacity, seed);
//!     let a_whole = serialize_sketch(alice_set_full);
//!     let a_half = serialize_sketch(sketch_from_range(a_half, capacity, seed));
//! 
//!     let bob_set_full = sketch_from_range(b, capacity, seed);
//!     let b_whole = serialize_sketch(bob_set_full);
//!     let b_half = serialize_sketch(sketch_from_range(b_half, capacity, seed));
//! 
//!     println!("Trying simple reconciliation");
//!     let simple = reconcile(&a_whole, &b_whole, capacity, seed);
//!     if let Err(()) = simple {
//!         println!(
//!             "Error. Difference exceeds sketch capacity: {} > {}",
//!             set_diff, capacity
//!         );
//!         println!("Trying bisection");
//! 
//!         let a_minus_a_2 = sub_sketches(&a_whole, &a_half, capacity, seed);
//!         let b_minus_b_2 = sub_sketches(&b_whole, &b_half, capacity, seed);
//! 
//!         let res_1 = reconcile(&a_half, &b_half, capacity, seed);
//!         let res_2 = reconcile(&a_minus_a_2, &b_minus_b_2, capacity, seed);
//! 
//!         let res = res_1.and_then(|diffs1| {
//!             res_2.and_then(|diffs2| {
//!                 Ok(diffs1
//!                     .into_iter()
//!                     .chain(diffs2.into_iter())
//!                     .collect::<Vec<_>>())
//!             })
//!         });
//! 
//!         res
//!     } else {
//!         Ok(simple.unwrap())
//!     }
//! }
//! 
//! pub fn main() {
//!     let capacity = 16; // Try to change it to 24 and compare results
//! 
//!     match example(capacity) {
//!         Ok(mut diffs) => {
//!             // Sort differences for result readability (not required)
//!             diffs.sort();
//! 
//!             println!("Success!");
//!             println!("Differences: {:?}", diffs);
//!         }
//! 
//!         Err(()) => println!("Example failed"),
//!     }
//! }
//! ```
// Auto-generated. Do not modify.
