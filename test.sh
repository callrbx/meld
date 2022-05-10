#!/bin/sh

echo "[*} Running Rust Function Tests"
cargo test
RS_RES=$?

if [ 0 -ne $RS_RES ]; then
    echo "[-] Rust Tests Failed" && return 1
fi

echo "[*] Running Integration Suite Tests"
git submodule update --remote
cd meld-test-suite || return 1
CLIENTBIN=$(pwd)/../target/release/meld ./run_tests.sh
SUITE_RES=$?

if [ 0 -ne $SUITE_RES ]; then
    echo "[-] Suite Tests Failed" && return 1
fi

echo ""
echo ""
echo "[+] All Tests Passed!"