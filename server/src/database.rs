use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::MutexGuard,
    thread::sleep,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::{bitwise_query, query, query::QueryType, Changed, Db, Map};

/// Try to get a value from the database. If the value is not found, return
/// an error string.
///
/// This allows us to blindly return the response to the client without having
/// to worry about whether the value is found or not.
///
/// # Arguments
///
/// * `db` - The database to get the value from.
/// * `key` - The key to get the value from.
/// * `format` - A closure which should format the expected response.
fn get_value<F>(db: MutexGuard<Map>, key: &str, format: F) -> String
where
    F: Fn((String, String)) -> String,
{
    match db.get(key) {
        Some(value) => format(value.to_owned()),
        None => "Error: Key not found!".to_string(),
    }
}

/// Perform the proper action for the given query.
///
/// # Arguments
///
/// * `query_type` - The type of query to perform.
/// * `arguments` - The arguments to the query.
/// * `db` - The database to perform the action on.
fn execute_query(query_type: QueryType, arguments: Vec<String>, db: &Db) -> String {
    let mut db = db.lock().unwrap();

    match query_type {
        QueryType::New => {
            db.insert(
                arguments[0].to_owned(),
                (arguments[1].to_owned(), arguments[2].to_owned()),
            );
            return "Ok".to_string();
        }
        QueryType::Get => get_value(db, &arguments[0], |(value, ttl)| {
            format!("{}\0{}", value, ttl)
        }),
        QueryType::GetValue => get_value(db, &arguments[0], |(value, _)| value),
        QueryType::GetTTL => get_value(db, &arguments[0], |(_, ttl)| ttl),
        QueryType::Drop => {
            db.remove(&arguments[0]);
            return "Ok".to_string();
        }
        QueryType::DropAll => {
            db.retain(|_, (value, _)| *value != arguments[0]);
            return "Ok".to_string();
        }
        QueryType::QueryTypeString => "Ok".to_string(),
        QueryType::QueryTypeBitwise => "Ok".to_string(),
    }
}

/// Try to parse and execute a query. If something failed it will return an
/// error describing the problem.
///
/// # Arguments
///
/// * `db` - The database to execute the query on.
/// * `bytes` - The received query bytes.
/// * `is_bitwise` - If the query should be parsed as a bitwise query.
pub fn process_query(db: Db, bytes: &[u8], is_bitwise: bool) -> (Option<QueryType>, String) {
    let message = String::from_utf8(bytes.to_vec()).unwrap_or_default();
    let mut res = String::default();

    let valid_message = message != "" && message != "\n" && message.is_ascii();
    let mut query_type = None;

    if valid_message {
        let parsed = if is_bitwise {
            bitwise_query::parse_query(message.clone())
        } else {
            if message.contains(0 as char) {
                res = "Non bitwise queries cannot contain null bytes".to_string();
            }
            query::parse_query(message.clone())
        };

        if let Ok((qt, arguments)) = parsed {
            query_type = Some(qt);
            let result = execute_query(qt, arguments, &db);
            res.push_str(&result);
        } else {
            res = "Could not properly parse query!".to_string();
        }
    } else {
        res = "Invalid or empty query (must be valid ascii).".to_string();
    }

    trace!("{:?}", res);

    return (query_type, res);
}

/// Try to parse and fill a database from a file path. If the file does not
/// exist it will do nothing.
///
/// # Arguments
///
/// * `db` - The database to fill.
/// * `path` - The path to the file to parse.
pub fn load_db(db: Db, path: &str) {
    if Path::new(path).exists() {
        info!("Loading database from: {}", path);
        let start_load = Instant::now();
        let mut file = File::open(path).unwrap();
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
    }
}

/// Check if there were any changes detected. If there were write the data to a
/// file. If no changes were detected do nothing.
///
/// This spawns a new tokio thread to check this every x seconds.
///
/// # Arguments
///
/// * `db` - The database to check for changes.
/// * `changed` -  A mutex to check if there were any changes.
/// * `file_path` - The path to the file to write to.
/// * `interval` - The interval to check for changes.
pub fn detect_changes(db: Db, changed: Changed, file_path: String, interval: u64) {
    tokio::spawn(async move {
        // TODO: Work away the unwraps
        let duration = Duration::from_secs(interval);
        info!("Check for record changes every {} seconds", interval);

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
                let mut file = File::create(&file_path).unwrap();
                file.write_all(&compressed).unwrap();
            }
        }
    });
}

/// An intensive routine to check if data has become invalid. (it ttl expired)
/// It will drop any value that has expired.
///
/// # Arguments
///
/// * `db` - The database to check for expired values.
/// * `changed` - A mutex to check if there were any changes.
/// * `interval` - The interval to check for expired values.
pub fn detect_expirations(db: Db, changed: Changed, interval: u64) {
    tokio::spawn(async move {
        // TODO: Work away the unwraps
        let duration = Duration::from_secs(interval);
        info!("Checking for record expirations every {} seconds", interval);

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
                if ttl == "0" || ttl > current_epoch {
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
