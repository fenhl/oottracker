function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

git push
ThrowOnNativeFailure

ssh fenhl.net 'cd /opt/git/github.com/fenhl/oottracker/branch/mw && git pull --ff-only'
ThrowOnNativeFailure

ssh fenhl.net "env -C /opt/git/github.com/fenhl/oottracker/branch/mw $(Get-Content .\assets\web\env.txt) cargo build --release --package=oottracker-web"
ThrowOnNativeFailure

ssh fenhl.net 'sudo systemctl restart oottracker-web'
ThrowOnNativeFailure
