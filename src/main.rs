// TODO: Make use of multiprocessing
use std::{collections::HashMap, fs::File, io::Write, time::Instant};

use crate::ascii_optimisation::{compress_slice, decompress_slice};
use crate::serialisation::{from_slice, to_vec, Map};

mod ascii_optimisation;
mod serialisation;

#[cfg(test)]
mod test_serialisation;

#[cfg(test)]
mod test_ascii_optimisation;

fn main() {
    let sample: Map = HashMap::from([
        (
            "hello".to_string(),
            ("world".to_string(), "2022-04-12T22:10:57+02:00".to_string()),
        ),
        (
            "foo".to_string(),
            ("bar".to_string(), "2022-04-12T22:10:57+02:00".to_string()),
        ),
        (
            "bake".to_string(),
            ("eggs".to_string(), "2022-04-12T22:10:57+02:00".to_string()),
        ),
    ]);

    println!("Firefly specific encoder:");

    let start = Instant::now();
    let buffer = to_vec(sample).unwrap();
    let elapsed = start.elapsed();
    println!(
        "Serialisation time: {:.2?}, size: {:?}",
        elapsed,
        &buffer.len()
    );

    let compressed_start = Instant::now();
    let compressed = compress_slice(&buffer).unwrap();
    let compressed_elapsed = compressed_start.elapsed();
    println!(
        "Compression time: {:.2?}, size: {:?}",
        compressed_elapsed,
        compressed.len()
    );

    let decompression_start = Instant::now();
    let decompressed = decompress_slice(&compressed);
    let decompression_elapsed = decompression_start.elapsed();
    println!("Decompression time: {:.2?}", decompression_elapsed);

    let decode_start = Instant::now();
    let _decoded = from_slice(&decompressed).unwrap();
    let decode_elapsed = decode_start.elapsed();
    println!("Deserialisation time: {:.2?}", decode_elapsed);

    // Write buffer to file
    let mut file = File::create("messages.ffly").unwrap();
    file.write(&compressed).unwrap();
}
