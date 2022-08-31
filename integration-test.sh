#!/bin/bash
set -e

cd ./integration-tests && cargo run --example integration-tests
cd ..

