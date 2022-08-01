extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::env;
use std::error::Error;

static LOGGING_ENV: &'static str = "LOG_LEVEL";
static BIND_ADDR: &'static str = "127.0.0.1:46600";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os(LOGGING_ENV).is_none() {
        env::set_var(LOGGING_ENV, "INFO");
    }

    pretty_env_logger::init_custom_env(LOGGING_ENV);

    let listener = TcpListener::bind(BIND_ADDR).await?;
    info!("Listening on: {}", BIND_ADDR);

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                info!("{:?}", String::from_utf8_lossy(&buf[..n]));
                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}
// use std::{collections::HashMap, fs::File, io::Write, time::Instant};
//
// use crate::ascii_optimisation::{compress_slice, decompress_slice};
// use crate::serialisation::{from_slice, to_vec, Map};
//
// mod ascii_optimisation;
// mod serialisation;
//
// #[cfg(test)]
// mod test_serialisation;
//
// #[cfg(test)]
// mod test_ascii_optimisation;
//
// fn main() {
//     let sample: Map = {
//         use rand::{distributions::Alphanumeric, Rng};
//
//         let mut map: Map = HashMap::new();
//
//         for i in 0..50 {
//             let key: String = rand::thread_rng()
//                 .sample_iter(&Alphanumeric)
//                 .take(64)
//                 .map(char::from)
//                 .collect();
//             let value = (i.to_string(), "2100-07-26T14:17:06+00:00".to_string());
//
//             map.insert(key, value);
//         }
//
//         map
//     };
//
//     println!("Firefly specific encoder:");
//     println!("Applying tests to {} items...", sample.len());
//     println!("{:?}", sample);
//
//     let start = Instant::now();
//     let buffer = to_vec(sample).unwrap();
//     let elapsed = start.elapsed();
//     println!(
//         "Serialisation time: {:.2?}, size: {:?}",
//         elapsed,
//         &buffer.len()
//     );
//
//     let compressed_start = Instant::now();
//     let compressed = compress_slice(&buffer).unwrap();
//     let compressed_elapsed = compressed_start.elapsed();
//     println!(
//         "Compression time: {:.2?}, size: {:?}",
//         compressed_elapsed,
//         compressed.len()
//     );
//
//     let decompression_start = Instant::now();
//     let decompressed = decompress_slice(&compressed);
//     let decompression_elapsed = decompression_start.elapsed();
//     println!("Decompression time: {:.2?}", decompression_elapsed);
//
//     let decode_start = Instant::now();
//     let _decoded = from_slice(&decompressed).unwrap();
//     let decode_elapsed = decode_start.elapsed();
//     println!("Deserialisation time: {:.2?}", decode_elapsed);
//
//     // Write buffer to file
//     let mut file = File::create("messages.ffly").unwrap();
//     file.write(&compressed).unwrap();
// }
