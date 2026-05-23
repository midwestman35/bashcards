#!/usr/bin/env python3
"""
Build and install bashcards from a source checkout.

This is not a Python package setup file. It is a small distribution helper for
people who have Python and Rust installed and want a one-command local install.
"""

from __future__ import annotations

import argparse
import os
import platform
import shutil
import stat
import subprocess
import sys
from pathlib import Path
from typing import Sequence


APP_NAME = "bashcards"


def binary_name(system: str | None = None) -> str:
    system = system or platform.system()
    return f"{APP_NAME}.exe" if system == "Windows" else APP_NAME


def default_install_dir(
    system: str | None = None,
    home: Path | None = None,
    local_appdata: str | None = None,
) -> Path:
    system = system or platform.system()
    home = home or Path.home()
    if local_appdata is None:
        local_appdata = os.environ.get("LOCALAPPDATA")

    if system == "Windows":
        base = Path(local_appdata) if local_appdata else home / "AppData" / "Local"
        return base / "Programs" / APP_NAME

    return home / ".local" / "bin"


def release_artifact_path(repo_root: Path, system: str | None = None) -> Path:
    return repo_root / "target" / "release" / binary_name(system)


def run_command(
    command: Sequence[str],
    *,
    cwd: Path,
    dry_run: bool = False,
) -> None:
    rendered = " ".join(command)
    if dry_run:
        print(f"Would run: {rendered}")
        return

    print(f"Running: {rendered}")
    subprocess.run(command, cwd=cwd, check=True)


def ensure_cargo_available() -> None:
    if shutil.which("cargo") is None:
        raise SystemExit(
            "Cargo was not found on PATH. Install Rust from https://rustup.rs/ "
            "and run this script again."
        )


def install_release_artifact(
    repo_root: Path,
    install_dir: Path,
    system: str | None = None,
) -> Path:
    source = release_artifact_path(repo_root, system)
    if not source.exists():
        raise FileNotFoundError(
            f"Release binary not found at {source}. Run `cargo build --release` "
            "before installing."
        )

    return install_binary(source, install_dir, system)


def install_binary(
    source: Path,
    install_dir: Path,
    system: str | None = None,
) -> Path:
    if not source.exists():
        raise FileNotFoundError(f"Binary not found at {source}.")
    if not source.is_file():
        raise FileNotFoundError(f"Expected a binary file at {source}.")

    install_dir.mkdir(parents=True, exist_ok=True)
    destination = install_dir / binary_name(system)
    shutil.copy2(source, destination)

    if (system or platform.system()) != "Windows":
        destination.chmod(
            destination.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH
        )

    return destination


def path_entries() -> list[Path]:
    raw_path = os.environ.get("PATH", "")
    return [Path(entry).expanduser().resolve() for entry in raw_path.split(os.pathsep) if entry]


def install_dir_is_on_path(install_dir: Path) -> bool:
    target = install_dir.expanduser().resolve()
    return target in path_entries()


def print_path_hint(install_dir: Path, system: str | None = None) -> None:
    system = system or platform.system()
    if install_dir_is_on_path(install_dir):
        return

    print()
    print(f"{install_dir} is not currently on PATH.")
    if system == "Windows":
        print("Add it to your user PATH, then open a new terminal.")
        print(f'PowerShell example: setx PATH "$env:PATH;{install_dir}"')
    else:
        print("Add this to your shell profile, then open a new terminal:")
        print(f'export PATH="{install_dir}:$PATH"')


def verify_installed_binary(installed: Path) -> None:
    result = subprocess.run(
        [str(installed)],
        check=True,
        capture_output=True,
        text=True,
    )
    output = (result.stdout + result.stderr).strip()
    if output:
        print(output)


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build and install the bashcards terminal game from source."
    )
    parser.add_argument(
        "--install-dir",
        type=Path,
        default=None,
        help="Directory to copy the release binary into. Defaults to a user-local bin directory.",
    )
    parser.add_argument(
        "--binary",
        type=Path,
        default=None,
        help="Install a prebuilt bashcards binary instead of building from source.",
    )
    parser.add_argument(
        "--skip-build",
        action="store_true",
        help="Copy an existing target/release binary without running cargo build first.",
    )
    parser.add_argument(
        "--no-locked",
        action="store_true",
        help="Run cargo build without --locked.",
    )
    parser.add_argument(
        "--no-verify",
        action="store_true",
        help="Do not run the installed binary after copying it.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the actions that would run without building or copying files.",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv if argv is not None else sys.argv[1:])
    repo_root = Path(__file__).resolve().parent
    system = platform.system()
    install_dir = args.install_dir or default_install_dir(system)

    if args.binary is None and not args.skip_build:
        ensure_cargo_available()
        command = ["cargo", "build", "--release"]
        if not args.no_locked:
            command.append("--locked")
        run_command(command, cwd=repo_root, dry_run=args.dry_run)

    if args.dry_run:
        source = args.binary or release_artifact_path(repo_root, system)
        print(f"Would copy: {source} -> {install_dir / binary_name(system)}")
        print("Would verify: installed binary launches outside an interactive terminal")
        print_path_hint(install_dir, system)
        return 0

    if args.binary is not None:
        installed = install_binary(args.binary, install_dir, system)
    else:
        installed = install_release_artifact(repo_root, install_dir, system)
    print(f"Installed {APP_NAME} to {installed}")

    if not args.no_verify:
        verify_installed_binary(installed)

    print_path_hint(install_dir, system)
    print()
    print(f"Launch with: {APP_NAME}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
