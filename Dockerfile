FROM rust:latest
RUN rustup component add rustfmt
RUN apt update
RUN apt install sqlite3
LABEL authors="icon @callrbx"