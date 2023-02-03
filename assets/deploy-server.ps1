function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

$env:PYO3_PYTHON = "python"

ssh mercredi 'cd /opt/git/github.com/fenhl/oottracker/master && git pull --ff-only'
ThrowOnNativeFailure

ssh mercredi "env -C /opt/git/github.com/fenhl/oottracker/master $(Get-Content .\assets\web\env.txt) cargo build --release --package=oottracker-web"
ThrowOnNativeFailure

ssh mercredi 'sudo systemctl restart oottracker-web'
ThrowOnNativeFailure
