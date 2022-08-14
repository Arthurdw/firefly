FROM rust

WORKDIR /
COPY . .

RUN cargo install --path .

EXPOSE 46600

CMD ["ffly --host 0.0.0.0"]
