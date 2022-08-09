#![allow(unused)]
extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
static BIND_ADDR: &'static str = "127.0.0.1:46600";
static MAX_QUERY_SIZE: usize = 512;

static WRITE_EVERY_N_QUERIES: usize = 100;
static SAVE_TO: &'static str = "data.bincode";

pub type Map = HashMap<String, (String, String)>;
pub type Db = Arc<Mutex<Map>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os(LOGGING_ENV).is_none() {
        env::set_var(LOGGING_ENV, "INFO");
    }

    pretty_env_logger::init_custom_env(LOGGING_ENV);

    let listener = TcpListener::bind(BIND_ADDR).await?;
    info!("Binding connection to {}", BIND_ADDR);

    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let items_changed = Arc::new(Mutex::new(0));
    let is_writing = Arc::new(Mutex::new(false));

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

    info!("Started listening for connections... ({})", BIND_ADDR);
    loop {
        let (mut socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);
        let db = db.clone();
        let changed = items_changed.clone();
        let is_writing = is_writing.clone();

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

                match query_type {
                    Some(QueryType::QueryTypeBitwise) => is_bitwise = true,
                    Some(QueryType::QueryTypeString) => is_bitwise = false,
                    Some(QueryType::New) => changed_data = true,
                    Some(QueryType::Drop) => changed_data = true,
                    Some(QueryType::DropAll) => changed_data = true,
                    _ => (),
                }

                if changed_data {
                    // TODO: Work away the unwraps
                    let mut changed = changed.lock().unwrap();
                    *changed += 1;

                    if *changed % WRITE_EVERY_N_QUERIES == 0 {
                        if let Ok(mut is_writing) = is_writing.try_lock() {
                            if *is_writing {
                                continue;
                            }

                            *is_writing = true;
                            let db = db.lock().unwrap();

                            let buffer = bincode::serialize(&db.to_owned()).unwrap();
                            let compressed = buffer;
                            let mut file = File::create(SAVE_TO).unwrap();
                            file.write_all(&compressed).unwrap();
                            *changed = 0;
                            *is_writing = false;
                        }
                    }
                }

                if let Err(_) = response {
                    break;
                }
            }
        });
    }
}
