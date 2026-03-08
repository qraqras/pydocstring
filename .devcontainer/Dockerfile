FROM rust:1-bookworm

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install -y --no-install-recommends \
    git \
    build-essential \
    python3 \
    python3-venv \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt

WORKDIR /workspace
