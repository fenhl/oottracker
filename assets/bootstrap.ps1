# runs build commands that may be required by other build commands (since some crates include code from other crates, e.g. updaters)

function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

$env:PYO3_PYTHON = "python"

cargo build --release --target=x86_64-pc-windows-msvc --package=oottracker-updater
ThrowOnNativeFailure
cargo build --release --target=x86_64-pc-windows-msvc --package=oottracker-updater-bizhawk
ThrowOnNativeFailure
cargo build --package=oottracker-csharp
ThrowOnNativeFailure
cargo build --release --package=oottracker-csharp
ThrowOnNativeFailure
