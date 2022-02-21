param (
    [switch] $major,
    [switch] $minor,
    [switch] $patch
)

function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

$env:PYO3_PYTHON = "python"

#TODO don't increment if local version is already a version above the latest release (depending on param)
if ($major) {
    cargo run --release --package=oottracker-utils --bin=oottracker-version-bump -- major
    ThrowOnNativeFailure
    throw 'committing/pushing version bumps not yet implemented' #TODO
} elseif ($minor) {
    cargo run --release --package=oottracker-utils --bin=oottracker-version-bump -- minor
    ThrowOnNativeFailure
    throw 'committing/pushing version bumps not yet implemented' #TODO
} elseif ($patch) {
    cargo run --release --package=oottracker-utils --bin=oottracker-version-bump -- patch
    ThrowOnNativeFailure
    throw 'committing/pushing version bumps not yet implemented' #TODO
}

cargo run --release --package=oottracker-utils --bin=oottracker-release
ThrowOnNativeFailure
