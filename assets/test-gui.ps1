function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

$env:PYO3_PYTHON = "python"

cargo run --package=oottracker-gui -- --checks #TODO move this option to GUI settings?
ThrowOnNativeFailure
