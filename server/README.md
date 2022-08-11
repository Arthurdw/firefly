# Firefly server

The TCP server to handle firefly requests, this uses the Tokio runtime.

## Running the server

### Dependencies

-   Rust
-   Cargo

### Build

Build the server, the binary is located in the `target` directory.

```bash
$ cargo build --release
```

## Customization

Customizing the server can currently be done by modifying values within the
`src/main.rs` file.
