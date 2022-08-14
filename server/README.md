# Firefly server

The TCP server to handle firefly requests, this uses the Tokio runtime.

## Installing the server

The [cargo ffly](https://crates.io/crates/ffly) crate can be used to install the server.

`$ cargo install ffly`

Or install it from the AUR.
`$ paru -S ffly`

For testing purposes you can also use the docker image:
[arthurdw/firefly](https://hub.docker.com/repository/docker/arthurdw/firefly)

## Building the server

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
