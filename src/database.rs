use std::sync::MutexGuard;

use crate::{
    query::{parse_query, QueryType},
    serialisation::Map,
    Db,
};

// TODO: Write some documentation
fn get_value<F>(db: MutexGuard<Map>, key: &str, format: F) -> String
where
    F: Fn((String, String)) -> String,
{
    match db.get(key) {
        Some(value) => format(value.to_owned()),
        None => "Error: Key not found!".to_string(),
    }
}

// TODO: Write some documentation
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
        QueryType::QueryTypeString => "Success".to_string(),
        QueryType::QueryTypeBitwise => "Error: Not yet implemented!".to_string(),
    }
}

// TODO: Write some documentation
// TODO: Write tests
pub fn process_query(db: Db, bytes: &[u8]) -> String {
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
