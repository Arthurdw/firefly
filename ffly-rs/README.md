# Firefly Rust library

> **⚠️ NOTE**: All documentation still has to be written

A utility library to provide optimized queries for the [Firefly](https://github.com/arthurdw/firefly) library.

## Using the library

The crate can be found at [crates.io ffly-rs](https://crates.io/crates/ffly-rs).

## Example

```rs
use ffly_rs::FireflyStream;

static FIREFLY_ADDR: &'static str = "127.0.0.1:46600";

#[tokio::main]
async fn main() {
    let mut firefly = FireflyStream::connect(FIREFLY_ADDR)
        .await
        .expect("Could not connect to Firefly server!");

    firefly.default_ttl = 60 * 60 * 24 * 7; // 7 days
    firefly
        .new("key", "value")
        .await
        .expect("Could not create a new record!");

    assert_eq!(
        firefly
            .get_value("key")
            .await
            .expect("Could not get value!"),
        "value"
    );
}
```
