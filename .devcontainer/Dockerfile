# See here for image contents: https://github.com/sarisia/mikanos-docker
FROM ghcr.io/sarisia/mikanos:amd64

# [Optional] Uncomment this section to install additional OS packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-here>

# RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh \
#     && source ~/.cargo/env \
#     && rustup install nightly-2022-06-20 \
#     && rustup component add rust-src --toolchain nightly-2022-06-20-x86_64-unknown-linux-gnu

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y