#!/bin/bash
rm -f bin/*
rm -f source/target/debug/deps/Carlito-*
cd source
cargo rustc --tests
cd ..
cp source/target/debug/deps/Carlito-* bin/
rm -f bin/*.d
export RUST_BACKTRACE=1
./bin/Carlito-* $* --exact --nocapture