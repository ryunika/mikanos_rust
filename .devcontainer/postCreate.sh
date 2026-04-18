#!/bin/sh
rustup toolchain install nightly-2022-06-20-x86_64-unknown-linux-gnu
rustup component add rust-src --toolchain nightly-2022-06-20-x86_64-unknown-linux-gnu
rustup component add rustfmt --toolchain nightly-2022-06-20-x86_64-unknown-linux-gnu

cd ~/edk2/
cp /workspace/target.txt ~/edk2/Conf/target.txt
ln -sf /workspace/mikanos/MikanLoaderPkg/ ./
