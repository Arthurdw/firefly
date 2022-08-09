use std::sync::MutexGuard;

use crate::{bitwise_query, query, query::QueryType, serialisation::Map, Db};

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

    match query_type {
        QueryType::New => {
            db.insert(
                arguments[0].to_owned(),
                (arguments[1].to_owned(), arguments[2].to_owned()),
            );
            return "Ok".to_string();
        }
        QueryType::Get => get_value(db, &arguments[0], |(value, ttl)| {
            format!("{},{}", value, ttl)
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

// TODO: Write some documentation
// TODO: Write tests
pub fn process_query(db: Db, bytes: &[u8], is_bitwise: bool) -> (Option<QueryType>, String) {
    let message = String::from_utf8(bytes.to_vec()).unwrap_or_default();
    let mut res = String::default();

    let valid_message = message != "" && message != "\n" && message.is_ascii();
    let mut query_type = None;

    if valid_message {
        let parsed = if is_bitwise {
            bitwise_query::parse_query(message.clone())
        } else {
            query::parse_query(message.clone())
        };

        if let Ok((qt, arguments)) = parsed {
            query_type = Some(qt);
            let result = execute_query(qt, arguments, &db);
            debug!("{:?}", message);
            res.push_str(&result);
        } else {
            res = "Could not properly parse query!".to_string();
        }
    } else {
        res = "Invalid or empty query (all values must be valid ascii).".to_string();
    }

    res.push('\n');
    trace!("{:?}", res);

    return (query_type, res);
}
