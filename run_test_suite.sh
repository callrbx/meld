#!/bin/sh

git submodule update --remote
cd meld-test-suite
CLIENTBIN=$(pwd)/../target/release/meld ./run_tests.sh