//! Example of simple set reconciliation between Alice and Bob. Bob is the one who wants to reconcile
//! his set of data with Alice.
//!
//! ```notrust
//! +-------+                            +-----+                                         
//! | Alice |                            | Bob |                                         
//! +-------+                            +-----+                                         
//!     |                                   |                                            
//!     | Sketch from a set [3000..3010]    |                                            
//!     |-------------------------------    |                                            
//!     |                              |    |                                            
//!     |<------------------------------    |                                            
//!     |                                   |                                            
//!     |                                   | Sketch from a set [3002..3012]             
//!     |                                   |-------------------------------             
//!     |                                   |                              |             
//!     |                                   |<------------------------------             
//!     |                                   |                                            
//!     | Serialized sketch                 |                                            
//!     |---------------------------------->|                                            
//!     |                                   |                                            
//!     |                                   | Deserialize sketch                         
//!     |                                   |-------------------                         
//!     |                                   |                  |                         
//!     |                                   |<------------------                         
//!     |                                   |                                            
//!     |                                   | Merge two sketches                         
//!     |                                   |-------------------                         
//!     |                                   |                  |                         
//!     |                                   |<------------------                         
//!     |                                   |                                            
//!     |                                   | Extract differences from the merged sketch
//!     |                                   |-------------------------------------------
//!     |                                   |                                          |
//!     |                                   |<------------------------------------------
//!     |                                   |                                            
//!     |       (Optional) Send differences |                                            
//!     |<----------------------------------|                                            
//!     |                                   |
//! ```
use minisketch_rs::{Minisketch, MinisketchError};

fn create_sketch(elements: impl IntoIterator<Item = u64>) -> Result<Minisketch, MinisketchError> {
    let mut sketch = Minisketch::try_new(12, 0, 4)?;
    for item in elements.into_iter() {
        sketch.add(item);
    }

    Ok(sketch)
}

fn create_sketch_alice() -> Result<Minisketch, MinisketchError> {
    let set = 3_000..3_010;
    println!(
        "Alice's set: {:?}",
        set.clone().into_iter().collect::<Vec<_>>()
    );

    Ok(create_sketch(set)?)
}

fn create_sketch_bob() -> Result<Minisketch, MinisketchError> {
    let set = 3_002..3_012;
    println!(
        "Bob's set: {:?}",
        set.clone().into_iter().collect::<Vec<_>>()
    );

    Ok(create_sketch(set)?)
}

fn reconcile_with_bob(msg_alice: &[u8]) -> Result<(), MinisketchError> {
    let mut sketch_bob = create_sketch_bob()?;

    // Restore Alice's sketch (not set!) from serialized message
    let mut sketch_alice = Minisketch::try_new(12, 0, 4)?;
    sketch_alice.deserialize(&msg_alice);

    // Reconcile sets by merging sketches
    sketch_bob.merge(&sketch_alice)?;

    // Extract difference between two sets from merged sketch
    let mut differences = [0u64; 4];
    let num_differences = sketch_bob.decode(&mut differences[..])?;

    println!("Differences between Alice and Bob: {}", num_differences);
    assert!(num_differences > 0);

    // Sort differences since they may come in arbitrary order from Minisketch::decode()
    let mut differences = Vec::from(&differences[..]);
    differences.sort();

    for (i, diff) in differences.iter().enumerate() {
        println!("Difference #{}: {}", (i + 1), diff);
    }

    assert_eq!(differences[0], 3_000);
    assert_eq!(differences[1], 3_001);
    assert_eq!(differences[2], 3_010);
    assert_eq!(differences[3], 3_011);

    Ok(())
}

pub fn main() -> Result<(), MinisketchError> {
    // Create sketch of Alice's set
    let sketch_alice = create_sketch_alice()?;

    // Serialize sketch as bytes
    let mut buf_a = vec![0u8; sketch_alice.serialized_size()];
    sketch_alice.serialize(buf_a.as_mut_slice())?;

    println!("Message: {}, {:?}", buf_a.len(), buf_a);

    // Send bytes to Bob for set reconciliation
    reconcile_with_bob(&buf_a)?;

    Ok(())
}
