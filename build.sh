#!/bin/bash
set -euxo pipefail

cargo build --release

cd examples
mkdir -p build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build .
