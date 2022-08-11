extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use tokio::net::TcpListener;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::connection::handle_connection;
use crate::database::{detect_changes, detect_expirations, load_db};

mod bitwise_query;
mod connection;
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
pub type Changed = Arc<Mutex<usize>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var_os(LOGGING_ENV).is_none() {
        env::set_var(LOGGING_ENV, DEFAULT_LOG_LEVEL);
    }

    pretty_env_logger::init_custom_env(LOGGING_ENV);

    let listener = TcpListener::bind(BIND_ADDR).await?;
    info!("Binding connection to {}", BIND_ADDR);

    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let items_changed: Changed = Arc::new(Mutex::new(0));

    load_db(db.clone(), SAVE_TO);
    detect_changes(
        db.clone(),
        items_changed.clone(),
        SAVE_TO.to_string(),
        SAVE_EVERY,
    );
    detect_expirations(db.clone(), items_changed.clone(), CLEAR_EVERY);

    info!("Started listening for connections... ({})", BIND_ADDR);
    loop {
        let (socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);
        handle_connection(socket, MAX_QUERY_SIZE, db.clone(), items_changed.clone());
    }
}
