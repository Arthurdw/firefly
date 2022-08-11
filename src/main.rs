extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::database::process_query;
use crate::query::QueryType;

mod bitwise_query;
mod database;
mod query;

#[cfg(test)]
mod test_query;

#[cfg(test)]
mod test_bitwise_query;

static LOGGING_ENV: &'static str = "LOG_LEVEL";
static DEFAULT_LOG_LEVEL: &'static str = "TRACE";
static BIND_ADDR: &'static str = "127.0.0.1:46600";
static MAX_QUERY_SIZE: usize = 512;

static SAVE_EVERY: u64 = 1;
static SAVE_TO: &'static str = "data.bincode";

static CLEAR_EVERY: u64 = 10;

pub type Map = HashMap<String, (String, String)>;
pub type Db = Arc<Mutex<Map>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO: Refactor this function
    if env::var_os(LOGGING_ENV).is_none() {
        env::set_var(LOGGING_ENV, DEFAULT_LOG_LEVEL);
    }

    pretty_env_logger::init_custom_env(LOGGING_ENV);

    let listener = TcpListener::bind(BIND_ADDR).await?;
    info!("Binding connection to {}", BIND_ADDR);

    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let items_changed = Arc::new(Mutex::new(0));

    if Path::new(SAVE_TO).exists() {
        info!("Loading database from: {}", SAVE_TO);
        let start_load = Instant::now();
        let mut file = File::open(SAVE_TO).unwrap();
        info!("Reading data from file...");
        let mut start = Instant::now();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        if data.len() < 1 {
            warn!("No data found in file");
        } else {
            info!(
                "Read {} bytes in {:.2?}, started deserialisation...",
                data.len(),
                start.elapsed()
            );
            start = Instant::now();
            let map: Map = bincode::deserialize(&mut data).unwrap();
            info!(
                "Deserialised {} items in {:.2?}, finished loading in {:.2?}",
                map.len(),
                start.elapsed(),
                start_load.elapsed()
            );
            let mut db = db.lock().unwrap();
            *db = map;
        }
    } else {
        info!("Database file not found, starting fresh!");
    }

    {
        let db = db.clone();
        let changed = items_changed.clone();

        tokio::spawn(async move {
            // TODO: Work away the unwraps
            let duration = Duration::from_secs(SAVE_EVERY);
            info!("Check for record changes every {} seconds", SAVE_EVERY);

            loop {
                sleep(duration);
                trace!("Checking if any data has been changed!");

                let mut changed = changed.lock().unwrap();
                if *changed != 0 {
                    debug!("{} record(s) changed... writing the data!", *changed);
                    *changed = 0;
                    drop(changed);

                    let db = db.lock().unwrap();
                    let buffer = bincode::serialize(&db.to_owned()).unwrap();
                    drop(db);

                    let compressed = buffer;
                    let mut file = File::create(SAVE_TO).unwrap();
                    file.write_all(&compressed).unwrap();
                }
            }
        });
    }

    {
        let db = db.clone();
        let changed = items_changed.clone();

        tokio::spawn(async move {
            // TODO: Work away the unwraps
            let duration = Duration::from_secs(CLEAR_EVERY);
            info!(
                "Checking for record expirations every {} seconds",
                CLEAR_EVERY
            );

            loop {
                sleep(duration);
                trace!("Checking if record's got expired.");
                let mut db = db.lock().unwrap();
                let records = db.to_owned();

                let current_epoch = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Woah, your system time is before the UNIX EPOCH!")
                    .as_secs()
                    .to_string();

                for (key, (_, ttl)) in records {
                    if ttl == "0" {
                        continue;
                    }

                    if ttl > current_epoch {
                        continue;
                    }

                    trace!("Dropping record with key {}", key);
                    db.remove(&key);

                    let mut changed = changed.lock().unwrap();
                    *changed += 1;
                }
            }
        });
    }

    info!("Started listening for connections... ({})", BIND_ADDR);
    loop {
        let (mut socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);
        let db = db.clone();
        let changed = items_changed.clone();

        tokio::spawn(async move {
            let mut buf = vec![0; MAX_QUERY_SIZE];
            let mut is_bitwise = false;

            loop {
                let incoming = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(_) => break,
                };

                let (query_type, res) = process_query(db.clone(), &buf[..incoming], is_bitwise);
                let response = socket.write_all(res.as_bytes()).await;

                let mut changed_data = false;

                {
                    use QueryType::*;

                    match query_type {
                        Some(QueryTypeBitwise) => is_bitwise = true,
                        Some(QueryTypeString) => is_bitwise = false,
                        Some(New | Drop | DropAll) => changed_data = true,
                        _ => (),
                    }
                }

                if changed_data {
                    // TODO: Work away the unwraps
                    let mut changed = changed.lock().unwrap();
                    *changed += 1;
                }

                if let Err(_) = response {
                    break;
                }
            }
        });
    }
}
