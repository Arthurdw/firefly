extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use query::QueryType;
use serialisation::Map;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::Instant;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::query::parse_query;

mod ascii_optimisation;
mod query;
mod serialisation;

#[cfg(test)]
mod test_query;

#[cfg(test)]
mod test_serialisation;

#[cfg(test)]
mod test_ascii_optimisation;

static LOGGING_ENV: &'static str = "LOG_LEVEL";
static BIND_ADDR: &'static str = "127.0.0.1:46600";

type Db = Arc<Mutex<Map>>;

fn get_value<F>(db: MutexGuard<Map>, key: &str, format: F) -> String
where
    F: Fn((String, String)) -> String,
{
    match db.get(key) {
        Some(value) => format(value.to_owned()),
        None => "Error: Key not found!".to_string(),
    }
}

fn execute_query(query_type: QueryType, arguments: Vec<String>, db: &Db) -> String {
    let mut db = db.lock().unwrap();
    let key = arguments[0].to_owned();

    match query_type {
        QueryType::New => {
            db.insert(key, (arguments[1].to_owned(), arguments[2].to_owned()));
            return "Success".to_string();
        }
        QueryType::Get => get_value(db, &key, |(value, ttl)| format!("{},{}", value, ttl)),
        QueryType::GetValue => get_value(db, &key, |(value, _)| value),
        QueryType::GetTTL => get_value(db, &key, |(_, ttl)| ttl),
        QueryType::Drop => {
            db.remove(&arguments[0]);
            return "Success".to_string();
        }
        QueryType::DropAll => {
            db.retain(|_, (value, _)| *value != arguments[0]);
            return "Success".to_string();
        }
        QueryType::QueryTypeString => "QueryTypeString".to_string(),
        QueryType::QueryTypeBitwise => "QueryTypeBitwise".to_string(),
    }
}

fn process_query(db: Db, bytes: &[u8]) -> String {
    let message = String::from_utf8(bytes.to_vec()).unwrap_or_default();
    let mut res = String::default();
    let valid_message = message != "" && message != "\n" && message.is_ascii();

    if valid_message {
        if let Ok((query_type, arguments)) = parse_query(message.clone()) {
            let result = execute_query(query_type, arguments, &db);
            debug!("{:?}", message);
            res.push_str(&result);
        } else {
            res = "Could not properly parse query!".to_string();
        }
    } else {
        res = "Invalid or empty query (all values must be valid ascii).".to_string();
    }

    res.push('\n');

    return res;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os(LOGGING_ENV).is_none() {
        env::set_var(LOGGING_ENV, "DEBUG");
    }

    pretty_env_logger::init_custom_env(LOGGING_ENV);

    let listener = TcpListener::bind(BIND_ADDR).await?;
    info!("Listening on: {}", BIND_ADDR);

    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (mut socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);
        let db = db.clone();

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

                let res = process_query(db.clone(), &buf[..n]);

                socket
                    .write_all(res.as_bytes())
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}
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
