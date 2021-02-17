#!/bin/zsh

verbose=
quiet=--quiet
quiet_verbose=--quiet
for arg in "$@"; do
    case "$arg" in
        --verbose)
            verbose=--verbose
            quiet=
            quiet_verbose=--verbose
            ;;
        *)
            ;;
    esac
done

set -e

# make sure everything is executed relative to this script's location
cd "${0:a:h}"/..

/opt/git/github.com/fenhl/syncbin/master/bin/rust $quiet --no-project
git pull $quiet_verbose --ff-only
cargo $quiet_verbose run --release --package=oottracker-utils --bin=oottracker-release -- $verbose
