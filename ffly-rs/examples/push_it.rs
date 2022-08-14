/// Lets try to push to the limits of Firefly
/// ffly -c 0 -s 300
/// cargo run --release --example push_it
///
/// Performance on a 16gb - Intel i7-10510U (8 cores @ 4.9GHz)
/// ~167k ops/sec
use ffly_rs::FireflyStream;
use std::{iter::repeat_with, time::Instant};
use uuid::Uuid;

static FIREFLY_ADDR: &'static str = "127.0.0.1:46600";

static THREADS: usize = 10;
static REQUESTS_TOTAL: usize = 1_000_000;

async fn add_records(amount: usize) {
    let firefly = FireflyStream::connect(FIREFLY_ADDR).await.unwrap();
    let user = Uuid::new_v4().to_string();

    for _ in 0..amount {
        let key: String = repeat_with(fastrand::alphanumeric).take(64).collect();

        firefly.new(&key, &user).await.expect("Query failed!")
    }
}

#[tokio::main]
async fn main() {
    let requests_per_thread = REQUESTS_TOTAL / THREADS;
    let mut futures = Vec::with_capacity(THREADS);

    for _ in 0..THREADS {
        futures.push(add_records(requests_per_thread));
    }

    println!(
        "Starting to send {} requests per thread. ({} threads, {} requests in total)",
        requests_per_thread, THREADS, REQUESTS_TOTAL
    );
    let start = Instant::now();
    futures::future::join_all(futures).await;
    println!(
        "Created {} new records by using {} connections in {:?}.",
        REQUESTS_TOTAL,
        THREADS,
        start.elapsed()
    );
    println!(
        "This comes down to {} requests per second.",
        REQUESTS_TOTAL / start.elapsed().as_secs() as usize
    );
}
