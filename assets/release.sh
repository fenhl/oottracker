#!/bin/sh

set -e

cd /opt/git/github.com/fenhl/oottracker/master
/opt/git/github.com/fenhl/syncbin/master/bin/rust --no-project
git pull --ff
cargo run --release --package=oottracker-utils --bin=oottracker-release
