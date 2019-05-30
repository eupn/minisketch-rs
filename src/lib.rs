#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use crate::*;

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

                // Sort differences since they're come in arbitrary order from minisketch_decode()
                let mut differences = Vec::from(&differences[..]);
                differences.sort();

                assert_eq!(differences[0], 3_000);
                assert_eq!(differences[1], 3_001);
                assert_eq!(differences[2], 3_010);
                assert_eq!(differences[3], 3_011);
            }
        };
    }
}
