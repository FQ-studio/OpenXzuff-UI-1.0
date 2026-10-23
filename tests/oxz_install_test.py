import json
import tempfile
import unittest
import zipfile
from pathlib import Path

from oxz_install import PackageError, install_package, validate_package


def make_package(path: Path, ext=".lap", manifest_overrides=None, files=None):
    is_lap = ext == ".lap"
    manifest = {
        "format": "LAP" if is_lap else "LUAP",
        "formatVersion": 1,
        "id": "org.example.hello",
        "name": "Hello App",
        "version": "1.0.0",
        "language": "Luau" if is_lap else "Lua",
        "runtime": "OXLE" if is_lap else "Lua 5.4",
        "runtimeApi": 1,
        "entry": "main.luau" if is_lap else "main.lua",
        "permissions": [],
        "assets": [],
    }
    manifest.update(manifest_overrides or {})
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("manifest.json", json.dumps(manifest))
        z.writestr(manifest["entry"], "print('hello')")
        for name, data in (files or {}).items():
            z.writestr(name, data)
    return path


class InstallerTests(unittest.TestCase):
    def test_valid_lap_and_luap(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for ext in (".lap", ".luap"):
                package = make_package(root / ("hello" + ext), ext)
                manifest, _ = validate_package(package)
                self.assertEqual(manifest["id"], "org.example.hello")

    def test_installs_without_executing_script(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            package = make_package(root / "hello.lap")
            destination = install_package(package, root / "apps")
            self.assertTrue((destination / "main.luau").is_file())
            self.assertTrue((destination / "INSTALL-INFO.txt").is_file())

    def test_rejects_extension_language_mismatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            package = make_package(root / "bad.lap", ".luap")
            with self.assertRaises(PackageError):
                validate_package(package)

    def test_rejects_path_traversal(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            package = root / "evil.lap"
            with zipfile.ZipFile(package, "w") as z:
                z.writestr("manifest.json", "{}")
                z.writestr("../escape.txt", "bad")
            with self.assertRaises(PackageError):
                validate_package(package)

    def test_rejects_duplicate_manifest_keys(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            package = root / "dup.lap"
            with zipfile.ZipFile(package, "w") as z:
                z.writestr("manifest.json", '{"format":"LAP","format":"LUAP"}')
                z.writestr("main.luau", "print(1)")
            with self.assertRaises(PackageError):
                validate_package(package)

    def test_rejects_permissions_until_registry_exists(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            package = make_package(root / "perm.lap", manifest_overrides={"permissions": ["filesystem.write"]})
            with self.assertRaises(PackageError):
                validate_package(package)

    def test_refuses_overwrite_by_default(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            package = make_package(root / "hello.lap")
            install_package(package, root / "apps")
            with self.assertRaises(PackageError):
                install_package(package, root / "apps")


if __name__ == "__main__":
    unittest.main()
