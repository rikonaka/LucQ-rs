#!/bin/bash

# rustup target add x86_64-unknown-linux-musl
# rustup target add x86_64-unknown-freebsd
# rustup target add x86_64-unknown-linux-gnu
# rustup target add x86_64-unknown-netbsd
# rustup target add x86_64-pc-solaris
# rustup target add x86_64-apple-darwin
# rustup target add aarch64-apple-darwin
# rustup target add aarch64-unknown-linux-gnu
# rustup target add aarch64-unknown-linux-musl

cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-gnu
# cargo build --release --target x86_64-unknown-freebsd
# cargo build --release --target x86_64-unknown-netbsd
# cargo build --release --target x86_64-pc-solaris
# cargo build --release --target x86_64-apple-darwin
# cargo build --release --target aarch64-apple-darwin
# cargo build --release --target aarch64-unknown-linux-gnu
# cargo build --release --target aarch64-unknown-linux-musl
