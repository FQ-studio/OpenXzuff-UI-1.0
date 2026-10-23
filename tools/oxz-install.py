#!/usr/bin/env python3
"""Safe first-stage installer for OpenXzuff .lap/.luap ZIP packages.

This host-side developer tool validates and installs package files; it does not
execute scripts or claim kernel/GUI integration.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import stat
import sys
import tempfile
import zipfile
from pathlib import Path, PurePosixPath

MAX_ARCHIVE_BYTES = 64 * 1024 * 1024
MAX_UNPACKED_BYTES = 128 * 1024 * 1024
MAX_FILES = 2048
MAX_ENTRY_BYTES = 32 * 1024 * 1024
ID_RE = re.compile(r"^[a-zA-Z0-9]+(?:[.-][a-zA-Z0-9]+)+$")
SEMVER_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$")
MANIFEST_KEYS = {
    "format", "formatVersion", "id", "name", "version", "language",
    "runtime", "runtimeApi", "entry", "permissions", "assets",
}


class PackageError(ValueError):
    """Invalid or unsafe package."""


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise PackageError(f"manifest contains duplicate JSON key: {key}")
        result[key] = value
    return result


def _safe_member(name: str) -> PurePosixPath:
    if not name or "\\" in name or "\x00" in name:
        raise PackageError(f"unsafe archive path: {name!r}")
    path = PurePosixPath(name)
    if not path.parts or path.is_absolute() or any(part in ("", ".", "..") for part in path.parts):
        raise PackageError(f"unsafe archive path: {name!r}")
    if path.parts and re.match(r"^[A-Za-z]:", path.parts[0]):
        raise PackageError(f"drive path is not allowed: {name!r}")
    return path


def validate_package(package: Path):
    package = Path(package)
    if package.suffix.lower() not in (".lap", ".luap"):
        raise PackageError("package filename must end with .lap or .luap")
    if not package.is_file():
        raise PackageError("package file does not exist")
    if package.stat().st_size > MAX_ARCHIVE_BYTES:
        raise PackageError("archive exceeds 64 MiB limit")

    expected_format = "LAP" if package.suffix.lower() == ".lap" else "LUAP"
    expected_language = "Luau" if expected_format == "LAP" else "Lua"
    expected_entry_suffix = ".luau" if expected_format == "LAP" else ".lua"

    try:
        archive = zipfile.ZipFile(package, "r")
    except (OSError, zipfile.BadZipFile) as exc:
        raise PackageError(f"not a valid ZIP package: {exc}") from exc

    with archive:
        infos = archive.infolist()
        if not infos or len(infos) > MAX_FILES:
            raise PackageError(f"archive must contain 1 to {MAX_FILES} entries")
        names = {}
        total = 0
        for info in infos:
            raw_name = info.filename
            is_dir = raw_name.endswith("/")
            normalized = raw_name[:-1] if is_dir else raw_name
            safe_path = _safe_member(normalized)
            key = safe_path.as_posix().casefold()
            if key in names:
                raise PackageError(f"duplicate normalized archive path: {raw_name}")
            names[key] = info
            if info.flag_bits & 0x1:
                raise PackageError("encrypted ZIP entries are not supported")
            mode = (info.external_attr >> 16) & 0xFFFF
            if stat.S_ISLNK(mode):
                raise PackageError("symbolic links are not allowed in packages")
            if info.file_size < 0 or info.file_size > MAX_ENTRY_BYTES:
                raise PackageError(f"entry exceeds per-file size limit: {raw_name}")
            total += info.file_size
            if total > MAX_UNPACKED_BYTES:
                raise PackageError("unpacked archive exceeds 128 MiB limit")
            if info.compress_type not in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED):
                raise PackageError(f"unsupported compression method: {raw_name}")

        manifest_info = names.get("manifest.json")
        if manifest_info is None or manifest_info.is_dir():
            raise PackageError("manifest.json is required at archive root")
        try:
            manifest_raw = archive.read(manifest_info)
            manifest = json.loads(manifest_raw.decode("utf-8"), object_pairs_hook=_unique_object)
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise PackageError(f"manifest.json must be valid UTF-8 JSON: {exc}") from exc
        if not isinstance(manifest, dict):
            raise PackageError("manifest root must be a JSON object")

        missing = MANIFEST_KEYS - manifest.keys()
        if missing:
            raise PackageError("manifest missing fields: " + ", ".join(sorted(missing)))
        if (manifest["format"] != expected_format or not isinstance(manifest["formatVersion"], int)
                or isinstance(manifest["formatVersion"], bool) or manifest["formatVersion"] != 1):
            raise PackageError("package format or formatVersion does not match extension")
        if manifest["language"] != expected_language:
            raise PackageError("package language does not match extension")
        if not isinstance(manifest["id"], str) or not ID_RE.fullmatch(manifest["id"]):
            raise PackageError("id must be a reverse-DNS-style identifier")
        if not isinstance(manifest["name"], str) or not manifest["name"].strip() or len(manifest["name"]) > 120:
            raise PackageError("name must be 1 to 120 non-whitespace characters")
        if not isinstance(manifest["version"], str) or not SEMVER_RE.fullmatch(manifest["version"]):
            raise PackageError("version must follow semantic versioning, e.g. 1.0.0")
        if not isinstance(manifest["runtime"], str) or not manifest["runtime"].strip():
            raise PackageError("runtime must be a non-empty string")
        if not isinstance(manifest["runtimeApi"], int) or isinstance(manifest["runtimeApi"], bool) or manifest["runtimeApi"] < 1:
            raise PackageError("runtimeApi must be a positive integer")
        entry = manifest["entry"]
        if not isinstance(entry, str) or not entry.endswith(expected_entry_suffix):
            raise PackageError(f"entry must point to a {expected_entry_suffix} script")
        entry_path = _safe_member(entry).as_posix()
        entry_info = names.get(entry_path.casefold())
        if entry_info is None or entry_info.is_dir():
            raise PackageError("entry script is not present in the archive")
        if not isinstance(manifest["permissions"], list) or any(not isinstance(p, str) for p in manifest["permissions"]):
            raise PackageError("permissions must be a list of strings")
        # Until a permission registry exists, fail closed instead of granting unknown access.
        if manifest["permissions"]:
            raise PackageError("no permissions are currently supported; use an empty permissions list")
        if not isinstance(manifest["assets"], list) or any(not isinstance(p, str) for p in manifest["assets"]):
            raise PackageError("assets must be a list of paths")
        for asset in manifest["assets"]:
            asset_path = _safe_member(asset.rstrip("/")).as_posix()
            prefix = asset_path.casefold()
            if prefix not in names and not any(n.startswith(prefix + "/") for n in names):
                raise PackageError(f"declared asset path does not exist: {asset}")
        return manifest, infos


def install_package(package: Path, app_root: Path, replace: bool = False) -> Path:
    manifest, _ = validate_package(package)
    app_root = Path(app_root).resolve()
    destination = app_root / manifest["id"]
    if destination.exists() and not replace:
        raise PackageError(f"app already installed at {destination}; use --replace to replace it")
    app_root.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=".oxz-stage-", dir=app_root))
    backup = None
    try:
        with zipfile.ZipFile(package, "r") as archive:
            for info in archive.infolist():
                is_dir = info.filename.endswith("/")
                relative = _safe_member(info.filename[:-1] if is_dir else info.filename)
                target = staging.joinpath(*relative.parts)
                # Resolve containment even if the staging tree changes unexpectedly.
                if staging.resolve() not in target.resolve().parents and target.resolve() != staging.resolve():
                    raise PackageError("archive path escaped staging directory")
                if is_dir:
                    target.mkdir(parents=True, exist_ok=True)
                    continue
                target.parent.mkdir(parents=True, exist_ok=True)
                with archive.open(info, "r") as source, target.open("xb") as output:
                    shutil.copyfileobj(source, output, length=64 * 1024)
        (staging / "INSTALL-INFO.txt").write_text(
            f"Installed by OpenXzuff package installer\nID: {manifest['id']}\n"
            f"Version: {manifest['version']}\nFormat: {manifest['format']}\n"
            "Scripts were not executed during installation.\n",
            encoding="utf-8",
        )
        if destination.exists():
            backup = app_root / (".oxz-backup-" + manifest["id"].replace("/", "_"))
            if backup.exists():
                shutil.rmtree(backup)
            destination.rename(backup)
        staging.rename(destination)
        if backup is not None:
            shutil.rmtree(backup)
        return destination
    except Exception:
        if backup is not None and backup.exists() and not destination.exists():
            backup.rename(destination)
        raise
    finally:
        if staging.exists():
            shutil.rmtree(staging, ignore_errors=True)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="Validate/install OpenXzuff .lap and .luap packages")
    sub = parser.add_subparsers(dest="command", required=True)
    check = sub.add_parser("check", help="validate a package without installing it")
    check.add_argument("package", type=Path)
    install = sub.add_parser("install", help="install package files into a local app directory")
    install.add_argument("package", type=Path)
    install.add_argument("--root", type=Path, default=Path.home() / ".local/share/openxzuff/apps")
    install.add_argument("--replace", action="store_true", help="replace an existing app with the same ID")
    args = parser.parse_args(argv)
    try:
        manifest, _ = validate_package(args.package)
        if args.command == "check":
            print(f"VALID: {manifest['format']} {manifest['id']} {manifest['version']}")
            print("Note: validation does not execute scripts or prove runtime compatibility.")
        else:
            destination = install_package(args.package, args.root, args.replace)
            print(f"INSTALLED: {manifest['id']} {manifest['version']}")
            print(f"PATH: {destination}")
            print("Scripts were not executed; an OXLE/Lua runtime is still required.")
        return 0
    except (PackageError, OSError, zipfile.BadZipFile) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
