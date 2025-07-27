#!/bin/bash
cargo build 2>&1 | grep -E "warning:|unused" | grep -E "runtime/src|glr-core/src|tablegen/src" | sort | uniq