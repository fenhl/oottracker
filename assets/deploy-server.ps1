function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

$env:PYO3_PYTHON = "python"

git push
ThrowOnNativeFailure

ssh fenhl.net 'cd /opt/git/github.com/fenhl/oottracker/main && git pull --ff-only'
ThrowOnNativeFailure

ssh fenhl.net "env -C /opt/git/github.com/fenhl/oottracker/main $(Get-Content .\assets\web\env.txt) cargo build --release --package=oottracker-web"
ThrowOnNativeFailure

ssh fenhl.net 'sudo systemctl restart oottracker-web'
ThrowOnNativeFailure
