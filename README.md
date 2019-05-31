### minisketch-rs

[![Crates.io](https://img.shields.io/crates/v/minisketch-rs.svg)](https://crates.io/crates/minisketch-rs)
[![Crates.io](https://img.shields.io/crates/d/minisketch-rs.svg)](https://crates.io/crates/minisketch-rs)
[![Docs.rs](https://docs.rs/minisketch-rs/badge.svg)](https://docs.rs/minisketch-rs/0.1.0/minisketch_rs/)
[![Build Status](https://travis-ci.com/eupn/minisketch-rs.svg?branch=master)](https://travis-ci.com/eupn/minisketch-rs)

`minisketch-rs` is a wrapper around [minisketch](https://github.com/sipa/minisketch),
a C library by [Pieter Wuille](https://github.com/sipa) for efficient set reconciliation.

> minisketch is proposed as part of an [Erlay](https://arxiv.org/abs/1905.10518) technique for bandwidth-efficient TX propagation in Bitcoin.

This library exposes type-safe Rust bindings for all `minisketch` functions by providing `Minisketch` structure.

#### Example

Cargo.toml:
```toml
[dependencies]
minisketch-rs = "0.1"
```

Example of simple set reconciliation between Alice and Bob:
```rust
use minisketch_rs::Minisketch;

// Alice's side
let mut sketch_a = Minisketch::try_new(12, 0, 4).unwrap();

println!("Alice's set:");
for i in 3_000..3_010 {
    println!("{}", i);
    sketch_a.add(i);
}

let sersize = sketch_a.serialized_size();
assert_eq!(sersize, 12 * 4 / 8);

// Serialize message for Bob
let mut buf_a = vec![0u8; sersize];
sketch_a.serialize(buf_a.as_mut_slice()).unwrap();

println!("Message: {}, {:?}", buf_a.len(), buf_a);

// Bob's side
{
    // Bob's sketch
    println!("Bob's set:");
    let mut sketch_b = Minisketch::try_new(12, 0, 4).unwrap();
    for i in 3_002..3_012 {
        println!("{}", i);
        sketch_b.add(i);
    }

    // Alice's sketch
    let mut sketch_a = Minisketch::try_new(12, 0, 4).unwrap();
    sketch_a.deserialize(&buf_a); // Load Alice's sketch

    // Merge the elements from sketch_a into sketch_b. The result is a sketch_b
    // which contains all elements that occurred in Alice's or Bob's sets, but not
    // in both.
    sketch_b.merge(&sketch_a).unwrap();

    let mut differences = [0u64; 4];
    let num_differences = sketch_b.decode(&mut differences[..]).unwrap();

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
```

Code snippet above will print:

```
Alice's set:
3000
3001
3002
3003
3004
3005
3006
3007
3008
3009
Message: 6, [1, 224, 210, 249, 116, 105]
Bob's set:
3002
3003
3004
3005
3006
3007
3008
3009
3010
3011
Differences between Alice and Bob: 4
Difference #1: 3000
Difference #2: 3001
Difference #3: 3010
Difference #4: 3011
```
