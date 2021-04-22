#! /bin/bash

set -e

# Manages the `taint` toolchain.
# Based on a script with the same name in the miri project
#
# https://github.com/rust-lang/miri/blob/master/rustup-toolchain

# Make sure rustup-toolchain-install-master is installed.
if ! which rustup-toolchain-install-master >/dev/null; then
    echo "Please install rustup-toolchain-install-master by running 'cargo install rustup-toolchain-install-master'"
    exit 1
fi

# Determine new commit.
if [[ "$1" == "" ]]; then
    NEW_COMMIT=$(cat rust-version)
elif [[ "$1" == "HEAD" ]]; then
    NEW_COMMIT=$(git ls-remote https://github.com/rust-lang/rust/ HEAD | cut -f 1)
else
    NEW_COMMIT="$1"
fi
echo "$NEW_COMMIT" > rust-version

# Check if we already are at that commit.
CUR_COMMIT=$(rustc +taint --version -v 2>/dev/null | egrep "^commit-hash: " | cut -d " " -f 2)
if [[ "$CUR_COMMIT" == "$NEW_COMMIT" ]]; then
    echo "taint toolchain is already at commit $CUR_COMMIT."
    rustup override set taint
    exit 0
fi

# Install and setup new toolchain.
rustup toolchain uninstall taint
rustup-toolchain-install-master -n taint -c rust-src -c rustc-dev -c llvm-tools -c rustfmt -- "$NEW_COMMIT"
rustup override set taint

# Cleanup.
cargo clean