#!/bin/sh

echo "[*] Building New Release"
cargo build --release

echo "[*] Running Rust Function Tests"
cargo test --release
RS_RES=$?

if [ 0 -ne $RS_RES ]; then
    echo "[-] Rust Tests Failed" && return 1
else
    echo "[+] Rust Tests Passed"
fi

echo "[*] Running Integration Suite Tests"
git submodule update --remote
cd meld-test-suite || return 1
CLIENTBIN=$(pwd)/../target/release/meld ./run_tests.sh
SUITE_RES=$?

if [ 0 -ne $SUITE_RES ]; then
    echo "[-] Suite Tests Failed" && return 1
else
    echo "[+] Suite Tests Passed"
fi

echo ""
echo ""
echo "[+] All Tests Passed!"