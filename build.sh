#!/bin/bash

cargo build
sudo setcap CAP_NET_ADMIN=+eip target/debug/wireguard-web-autopeer
RUST_LOG=INFO ./target/debug/wireguard-web-autopeer