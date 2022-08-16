FROM rust:slim-bullseye as builder
COPY . /root
RUN cargo install --path /root

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/ffly /usr/local/bin/ffly
EXPOSE 46600
CMD ["ffly", "--host", "0.0.0.0", "--port", "46600"]
