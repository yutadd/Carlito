#!/bin/bash
rm -f bin/*
cd source
cargo build
cd ..
cp source/target/debug/Carlito bin/
./bin/Carlito $*