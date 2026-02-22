FROM ubuntu:24.04

RUN apt-get update -y && apt-get install -y \
    nodejs npm build-essential vim curl cmake \
    pkg-config libclang-dev docker.io docker-buildx qemu-kvm

RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN npm install -g @anthropic-ai/claude-code
