extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use clap::Parser;
use tokio::net::TcpListener;

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::{env, process};

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

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The host (ip) the server should bind to.
    #[clap(short, long, default_value = "127.0.0.1")]
    host: String,

    /// The port the server should bind to.
    #[clap(short, long, default_value = "46600")]
    port: u16,

    /// The path to the database file.
    #[clap(short, long, default_value = "data.bincode")]
    out: String,

    /// Save the database every N seconds.
    #[clap(short, long, default_value = "1")]
    save_every: u64,

    /// Check if there are expired keys every N seconds.
    /// 0 disables this.
    #[clap(short, long, default_value = "10")]
    clear_every: u64,

    /// Max query size in bytes.
    #[clap(short, long, default_value = "512")]
    max_query_size: usize,

    /// Log level (TRACE, DEBUG, INFO, WARN, ERROR, FATAL).
    #[clap(short, long, default_value = "INFO")]
    log_level: String,
}

pub type Map = HashMap<String, (String, String)>;
pub type Db = Arc<Mutex<Map>>;
pub type Changed = Arc<Mutex<usize>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const LOG_LEVELS: &[&str] = &["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];

    let args = Args::parse();

    if env::var_os(LOGGING_ENV).is_none() {
        let log_level: String = match args.log_level.to_uppercase() {
            level if LOG_LEVELS.contains(&level.as_str()) => level.to_string(),
            _ => {
                println!("Invalid log level: {}", args.log_level);
                println!("Valid log levels: {:?}", LOG_LEVELS);
                process::exit(1);
            }
        };
        env::set_var(LOGGING_ENV, log_level);
    }

    let bind_addr = format!("{}:{}", args.host, args.port);

    pretty_env_logger::init_custom_env(LOGGING_ENV);

    let listener = TcpListener::bind(&bind_addr).await?;
    info!("Binding connection to {}", bind_addr);

    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let items_changed: Changed = Arc::new(Mutex::new(0));

    load_db(db.clone(), &args.out);
    detect_changes(
        db.clone(),
        items_changed.clone(),
        args.out.to_string(),
        args.save_every,
    );

    if args.clear_every > 0 {
        detect_expirations(db.clone(), items_changed.clone(), args.clear_every);
    }

    info!("Started listening for connections... ({})", bind_addr);
    loop {
        let (socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);
        handle_connection(
            socket,
            args.max_query_size,
            db.clone(),
            items_changed.clone(),
        );
    }
}
