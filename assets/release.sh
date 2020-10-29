#!/bin/zsh

set -e

# make sure everything is executed relative to this script's location
cd "${0:a:h}"/..

/opt/git/github.com/fenhl/syncbin/master/bin/rust --no-project
git pull --ff-only
cargo run --release --package=oottracker-utils --bin=oottracker-release
