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

# make sure everything is executed relative to this script's location
cd "${0:a:h}"/..

function lock {
    echo 'acquiring rust lockdir'
    until mkdir /tmp/syncbin-startup-rust.lock &> /dev/null; do
        if [[ -f /tmp/syncbin-startup-rust.lock/pid ]] && ! ps -p "$(cat /tmp/syncbin-startup-rust.lock/pid)" &> /dev/null; then
            unlock
        fi
        sleep 1
    done
    trap 'rm -rf /tmp/syncbin-startup-rust.lock' HUP TERM INT # remove lock when script finishes
    echo $$ > "/tmp/syncbin-startup-rust.lock/pid"
}

function unlock {
    rm -f /tmp/syncbin-startup-rust.lock/pid # remove pidfile, if any
    rmdir /tmp/syncbin-startup-rust.lock # remove lock
    trap ':' HUP TERM INT # neutralize trap
}

# make sure multiple instances of rustup aren't running at the same time
lock
rustup $quiet_verbose update stable || exit $?
unlock

git pull $quiet_verbose --ff-only || exit $?
cargo $quiet_verbose run --release --package=oottracker-utils --bin=oottracker-release -- $verbose || exit $?
