#!/bin/sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

source ~/.cargo/env
# 日付を指定して、Rust の nightly をインストールする (nightly だけでも良いが、後々の変更が入ることを見込み、2022-06-20 時点では動くということを明示する)
# rustup install nightly-2022-06-20
# rust-src も必要になるので用意しておく
# rustup component add rust-src --toolchain nightly-2022-06-20-x86_64-unknown-linux-gnu