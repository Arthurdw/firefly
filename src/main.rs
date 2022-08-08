extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use serialisation::Map;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::database::process_query;

mod ascii_optimisation;
mod database;
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
static MAX_QUERY_SIZE: usize = 512;

pub type Db = Arc<Mutex<Map>>;

//     let buffer = to_vec(sample).unwrap();
//     let compressed = compress_slice(&buffer).unwrap();
//     let decompressed = decompress_slice(&compressed);
//     let decoded = from_slice(&decompressed).unwrap();
//
//     let mut file = File::create("messages.ffly").unwrap();
//     file.write(&compressed).unwrap();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os(LOGGING_ENV).is_none() {
        env::set_var(LOGGING_ENV, "INFO");
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
            let mut buf = vec![0; MAX_QUERY_SIZE];

            loop {
                let incoming = match socket.read(&mut buf).await {
                    Ok(n) => n,
                    Err(_) => break,
                };

                if incoming == 0 {
                    return;
                }

                let res = process_query(db.clone(), &buf[..incoming]);

                let response = socket.write_all(res.as_bytes()).await;

                if let Err(_) = response {
                    break;
                }
            }
        });
    }
}
