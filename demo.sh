#!/bin/sh

# setup
rm -rf /tmp/meld_test
cargo build --release
export PATH=$PWD/target/release/:$PATH

echo "> RUST_LOG=debug meld /tmp/meld_test init"
RUST_LOG=debug meld /tmp/meld_test init

echo "> meld /tmp/meld_test push Cargo.toml"
meld /tmp/meld_test push Cargo.toml

echo "> echo \"NONSENSE\" > Cargo.toml"
echo \"NONSENSE\" > Cargo.toml

echo "> meld /tmp/meld_test push Cargo.toml # oh noes, a bad config overwrite"
meld /tmp/meld_test push Cargo.toml

echo "> rm Cargo.toml"
rm Cargo.toml

echo "> meld /tmp/meld_test pull ~/Projects/meld/meld-rust/Cargo.toml -v 1"
meld /tmp/meld_test pull ~/Projects/meld/meld-rust/Cargo.toml -v 1

echo "> head Cargo.toml"
head Cargo.toml

echo "> meld /tmp/meld_test push src/"
meld /tmp/meld_test push src/

echo "> tree /tmp/meld_test"
tree /tmp/meld_test

echo "> cat /tmp/meld_test/maps/*"
cat /tmp/meld_test/maps/*