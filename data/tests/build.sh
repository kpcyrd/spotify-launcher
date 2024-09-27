#!/usr/bin/env sh
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR

rm -f *.deb

# compression_methods=("xz" "gzip" "none")
compression_methods=("none")

for method in "${compression_methods[@]}"; do
    dpkg-deb -Z$method --build fake_deb_for_unit_testing fake_deb_for_unit_testing.$method.deb > /dev/null
done


for i in *.deb; do
    echo "$i -> $(ar t "$i" | grep data)"
done