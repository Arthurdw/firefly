use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{database::process_query, query::QueryType, Changed, Db};

/// Handle if a query has changed the query type (string or bitwise) or if it
/// has changed data.
///
/// # Arguments
///
/// * `query_type` - The query type.
/// * `is_bitwise` - The variable indicating if the current session is using bitwise queries.
/// * `changed_data` - The variable indicating if the query has changed.
fn check_query_impact(
    query_type: Option<QueryType>,
    is_bitwise: &mut bool,
    changed_data: &mut bool,
) {
    use QueryType::*;

    match query_type {
        Some(QueryTypeBitwise) => *is_bitwise = true,
        Some(QueryTypeString) => *is_bitwise = false,
        Some(New | Drop | DropAll) => *changed_data = true,
        _ => (),
    };
}

/// Process a query its impact beyond the usual database interaction. Also if
/// the query manipulated data let the changed muted know that. (by incrementing)
///
/// # Arguments
///
/// * `query_type` - The type of the query that got executed.
/// * `is_bitwise` - The variable indicating if the current session is using bitwise queries.
/// * `changed` - A mutex that is used to let the server know if the query has changed the database.
fn process_query_impact(query_type: Option<QueryType>, is_bitwise: &mut bool, changed: Changed) {
    let mut changed_data = false;
    check_query_impact(query_type, is_bitwise, &mut changed_data);

    if changed_data {
        // TODO: Work away the unwrap
        let mut changed = changed.lock().unwrap();
        *changed += 1;
    }
}

/// Handle a TCP stream/session. This contains the client interaction logic.
///
/// # Arguments
///
/// * `socket` - The TCP stream/session.
/// * `max_query_size` - The maximum expected query size, this is the default
///                         vector allocation size.
/// * `db` - The database.
/// * `changed` - A mutex that is used to let the server know if the query has changed the database.
pub fn handle_connection(mut socket: TcpStream, max_query_size: usize, db: Db, changed: Changed) {
    tokio::spawn(async move {
        let mut buf = vec![0; max_query_size];
        let mut is_bitwise = false;

        loop {
            let incoming = match socket.read(&mut buf).await {
                Ok(n) if n == 0 => return,
                Ok(n) => n,
                Err(_) => break,
            };

            let (query_type, res) = process_query(db.clone(), &buf[..incoming], is_bitwise);
            let response = socket.write_all(res.as_bytes()).await;
            process_query_impact(query_type, &mut is_bitwise, changed.clone());

            if let Err(_) = response {
                break;
            }
        }
    });
}
