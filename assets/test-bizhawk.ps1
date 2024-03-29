function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

$env:PYO3_PYTHON = "python"

cargo run --package=oottracker-utils --bin=oottracker-check-bizhawk-version
ThrowOnNativeFailure

cargo build --package=oottracker-csharp
ThrowOnNativeFailure

cargo build --package=oottracker-bizhawk
ThrowOnNativeFailure

Set-Location .\crate\oottracker-bizhawk\OotAutoTracker\BizHawk
.\EmuHawk.exe --open-ext-tool-dll=OotAutoTracker
Set-Location ..\..\..\..
