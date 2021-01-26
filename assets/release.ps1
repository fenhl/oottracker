function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

$env:PYO3_PYTHON = "python"

cargo lrun --release --package=oottracker-utils --bin=oottracker-release
ThrowOnNativeFailure #TODO if the error is “same version”, auto-increment and run release script again
