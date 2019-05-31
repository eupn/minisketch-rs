use minisketch_rs::Minisketch;

pub fn main() -> Result<(), ()> {
    // Alice's side
    let mut sketch_a = Minisketch::try_new(12, 0, 4)?;

    println!("Alice's set:");
    for i in 3_000..3_010 {
        println!("{}", i);
        sketch_a.add(i);
    }

    let sersize = sketch_a.serialized_size();
    assert_eq!(sersize, 12 * 4 / 8);

    // Serialize message for Bob
    let mut buf_a = vec![0u8; sersize];
    sketch_a.serialize(buf_a.as_mut_slice())?;

    println!("Message: {}, {:?}", buf_a.len(), buf_a);

    // Bob's side
    {
        // Bob's sketch
        println!("Bob's set:");
        let mut sketch_b = Minisketch::try_new(12, 0, 4)?;
        for i in 3_002..3_012 {
            println!("{}", i);
            sketch_b.add(i);
        }

        // Alice's sketch
        let mut sketch_a = Minisketch::try_new(12, 0, 4)?;
        sketch_a.deserialize(&buf_a); // Load Alice's sketch

        // Merge the elements from sketch_a into sketch_b. The result is a sketch_b
        // which contains all elements that occurred in Alice's or Bob's sets, but not
        // in both.
        sketch_b.merge(&sketch_a)?;

        let mut differences = [0u64; 4];
        let num_differences = sketch_b.decode(&mut differences[..])?;

        println!("Differences between Alice and Bob: {}", num_differences);

        assert!(num_differences > 0);

        // Sort differences since they may come in arbitrary order from minisketch_decode()
        let mut differences = Vec::from(&differences[..]);
        differences.sort();

        for (i, diff) in differences.iter().enumerate() {
            println!("Difference #{}: {}", (i + 1), diff);
        }

        assert_eq!(differences[0], 3_000);
        assert_eq!(differences[1], 3_001);
        assert_eq!(differences[2], 3_010);
        assert_eq!(differences[3], 3_011);
    }

    Ok(())
}
