# bashcards

`bashcards` is a Rust terminal game for learning shell commands through short,
replayable arcade cabinets. The current build includes Shell Motel, Monastery of
Forms, and Midnight Carnival.

## Requirements

- Rust and Cargo, installed from <https://rustup.rs/>
- An interactive terminal. On Windows, use Windows Terminal or PowerShell.

## Run From Source

```powershell
git clone https://github.com/midwestman35/bashcards.git
cd bashcards
cargo run
```

## Install Locally

From the repository root:

```powershell
python setup.py
```

The setup script builds a release binary and copies it to a user-local install
directory. It also prints a PATH hint if that directory is not already available
from your terminal.

For a zip that already contains a prebuilt binary, install that binary without
requiring Rust:

```powershell
python setup.py --binary .\bashcards.exe
```

You can also install directly through Cargo:

```powershell
cargo install --path .
```

Then launch the game from any interactive terminal:

```powershell
bashcards
```

If the command is not found, make sure Cargo's bin directory is on your `PATH`.
On Windows this is usually `%USERPROFILE%\.cargo\bin`.

## Development Checks

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Save Data

Progress is stored in the platform app data directory under the `bashcards`
profile. For test runs, set `BASHCARDS_PROFILE_PATH` to an alternate TOML file
to avoid touching your normal profile.
