#!/usr/bin/env bash
set -e
ver=$(dove -V | sed 's|\.|000|g;s|Dove ||')
if [ 100020002 -gt $ver ]; then
    echo "Your Dove is too old. Please, update at least to version 1.2.2."
    exit 1
fi

dove build -u
dove tx "store_u64(13)"
dove tx "tx_test<0x01::Pontem::T>(100)"
dove build -p -o "valid_pack" -u
dove build -p -o "invalid_pack" -e "Store" -u
