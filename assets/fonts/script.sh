#!/bin/bash
set -euxo pipefail

# Download Inter font from Google Fonts or the official repository
# Place Inter-Regular.ttf in plotffi/assets/
curl -L "https://github.com/rsms/inter/releases/download/v4.0/Inter-4.0.zip" -o inter.zip
trap "rm -rf inter.zip extras" EXIT
unzip inter.zip "extras/ttf/Inter-Regular.ttf" -d ./
mv extras/ttf/Inter-Regular.ttf ./
unzip inter.zip "LICENSE.txt"
mv LICENSE.txt ../../LICENSES/OFL-1.1.txt
