use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{database::process_query, query::QueryType, Changed, Db};

// TODO: Document this
// TODO: Refactor this
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
                // TODO: Work away the unwrap
                let mut changed = changed.lock().unwrap();
                *changed += 1;
            }

            if let Err(_) = response {
                break;
            }
        }
    });
}
