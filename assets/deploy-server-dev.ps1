function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

wsl -d debian-m2 cargo build --package=oottracker-web
ThrowOnNativeFailure

ssh fenhl.net 'sudo systemctl stop oottracker-web'
ThrowOnNativeFailure

scp target/debug/oottracker-web fenhl.net:bin/oottracker-web-dev
ThrowOnNativeFailure

ssh fenhl.net "chmod +x bin/oottracker-web-dev && env -C /opt/git/github.com/fenhl/oottracker/branch/mw $(Get-Content .\assets\web\env.txt) /home/fenhl/bin/oottracker-web-dev"
ThrowOnNativeFailure
