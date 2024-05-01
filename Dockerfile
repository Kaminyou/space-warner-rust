FROM rust:1.77.2
COPY . /root/app/
WORKDIR /root/app/
RUN cargo build --release
CMD ["./target/release/space-warner-rust"]