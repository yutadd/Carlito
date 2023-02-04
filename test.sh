#!/bin/bash
rm -f bin/*
cd source
cargo rustc --tests
cd ..
cp source/target/debug/deps/Carlito-* bin/
rm -f bin/*.d
./bin/Carlito-*