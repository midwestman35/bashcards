import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SETUP_PATH = ROOT / "setup.py"


def load_setup_module():
    spec = importlib.util.spec_from_file_location("bashcards_setup", SETUP_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SetupScriptTests(unittest.TestCase):
    def test_default_install_dir_uses_platform_user_bin_location(self):
        setup = load_setup_module()

        self.assertEqual(
            setup.default_install_dir("Windows", Path("C:/Users/Ada"), "C:/Local"),
            Path("C:/Local/Programs/bashcards"),
        )
        self.assertEqual(
            setup.default_install_dir("Linux", Path("/home/ada"), None),
            Path("/home/ada/.local/bin"),
        )
        self.assertEqual(
            setup.default_install_dir("Darwin", Path("/Users/ada"), None),
            Path("/Users/ada/.local/bin"),
        )

    def test_install_release_artifact_copies_expected_binary_and_returns_destination(self):
        setup = load_setup_module()

        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp) / "repo"
            install_dir = Path(tmp) / "bin"
            artifact_dir = repo / "target" / "release"
            artifact_dir.mkdir(parents=True)
            source = artifact_dir / "bashcards.exe"
            source.write_bytes(b"fake executable")

            installed = setup.install_release_artifact(repo, install_dir, "Windows")

            self.assertEqual(installed, install_dir / "bashcards.exe")
            self.assertEqual(installed.read_bytes(), b"fake executable")

    def test_install_binary_copies_prebuilt_distribution_artifact(self):
        setup = load_setup_module()

        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / "dist" / "bashcards.exe"
            install_dir = Path(tmp) / "bin"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"prebuilt executable")

            installed = setup.install_binary(source, install_dir, "Windows")

            self.assertEqual(installed, install_dir / "bashcards.exe")
            self.assertEqual(installed.read_bytes(), b"prebuilt executable")

    def test_install_release_artifact_errors_when_binary_was_not_built(self):
        setup = load_setup_module()

        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaisesRegex(FileNotFoundError, "cargo build --release"):
                setup.install_release_artifact(Path(tmp), Path(tmp) / "bin", "Linux")


if __name__ == "__main__":
    unittest.main()
