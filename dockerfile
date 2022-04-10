FROM rust:1.60.0

COPY ./ ./
RUN mkdir upload
RUN cargo build --release
CMD ["./target/release/litebin"]
