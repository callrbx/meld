FROM rust:latest
RUN rustup component add rustfmt
LABEL authors="icon @callrbx"