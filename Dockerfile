# 1. This tells docker to use the Rust official image
FROM rust:slim-buster

# 2. Copy the files in your machine to the Docker image
COPY . /core

# Set working directory
WORKDIR /core
# Build your program for release
RUN cargo build -p core_server  --release

# Run the binary
EXPOSE 8080
CMD ["./target/release/core_server"]