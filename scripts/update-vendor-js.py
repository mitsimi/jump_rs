#!/usr/bin/env python3
"""Update vendored HTMX and Alpine browser bundles.

Downloads the requested npm package versions, extracts the published browser
bundle from each tarball, and writes a manifest with source and checksum data.
"""

from __future__ import annotations

import argparse
import base64
import difflib
import hashlib
import io
import json
import sys
import tarfile
import tempfile
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VENDOR_DIR = ROOT / "static" / "vendor"
MANIFEST_PATH = VENDOR_DIR / "manifest.json"
NPM_REGISTRY = "https://registry.npmjs.org"


@dataclass(frozen=True)
class VendorAsset:
    key: str
    package: str
    package_file: str
    destination: Path
    version_arg: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Update static/vendor HTMX and Alpine bundles from npm."
    )
    parser.add_argument(
        "--htmx",
        default="latest",
        help="htmx.org version or dist-tag to vendor. Defaults to latest.",
    )
    parser.add_argument(
        "--alpine",
        default="latest",
        help="alpinejs version or dist-tag to vendor. Defaults to latest.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Fail if vendored files or manifest are not already up to date.",
    )
    return parser.parse_args()


def fetch_json(url: str) -> dict:
    with urllib.request.urlopen(url, timeout=30) as response:
        return json.load(response)


def fetch_bytes(url: str) -> bytes:
    with urllib.request.urlopen(url, timeout=60) as response:
        return response.read()


def resolve_version(package: str, requested: str) -> tuple[str, dict]:
    metadata = fetch_json(f"{NPM_REGISTRY}/{package.replace('/', '%2f')}")
    versions = metadata.get("versions", {})

    if requested in versions:
        return requested, versions[requested]

    dist_tags = metadata.get("dist-tags", {})
    version = dist_tags.get(requested)
    if version and version in versions:
        return version, versions[version]

    available_tags = ", ".join(sorted(dist_tags)) or "none"
    raise RuntimeError(
        f"{package}: unknown version or dist-tag {requested!r} "
        f"(available dist-tags: {available_tags})"
    )


def extract_file(tarball: bytes, package_file: str) -> bytes:
    expected_name = f"package/{package_file}"
    with tarfile.open(fileobj=io.BytesIO(tarball), mode="r:gz") as archive:
        member = archive.getmember(expected_name)
        extracted = archive.extractfile(member)
        if extracted is None:
            raise RuntimeError(f"Could not extract {expected_name}")
        return extracted.read()


def sri_sha384(data: bytes) -> str:
    digest = hashlib.sha384(data).digest()
    return "sha384-" + base64.b64encode(digest).decode("ascii")


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def validate_asset(asset: VendorAsset, content: bytes) -> None:
    if not content.strip():
        raise RuntimeError(f"{asset.destination} download was empty")

    probes = {
        "htmx": b"htmx",
        "alpine": b"Alpine",
    }
    probe = probes[asset.key]
    if probe not in content:
        raise RuntimeError(
            f"{asset.destination} did not look like the expected {asset.key} bundle"
        )


def vendor_asset(asset: VendorAsset) -> tuple[bytes, dict]:
    version, package_meta = resolve_version(asset.package, asset.version_arg)
    tarball_url = package_meta["dist"]["tarball"]
    content = extract_file(fetch_bytes(tarball_url), asset.package_file)
    validate_asset(asset, content)

    manifest_entry = {
        "package": asset.package,
        "version": version,
        "package_file": asset.package_file,
        "destination": str(asset.destination.relative_to(ROOT)),
        "tarball": tarball_url,
        "integrity": package_meta["dist"].get("integrity"),
        "sha256": sha256_hex(content),
        "sri": sri_sha384(content),
    }
    return content, manifest_entry


def render_manifest(entries: dict[str, dict]) -> bytes:
    manifest = {
        "schema": 1,
        "assets": entries,
    }
    return (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")


def write_or_check(path: Path, content: bytes, check: bool) -> bool:
    current = path.read_bytes() if path.exists() else None
    if current == content:
        return False

    if check:
        print(f"{path.relative_to(ROOT)} is not up to date", file=sys.stderr)
        if current is not None and path.suffix in {".json", ".js"}:
            current_text = current.decode("utf-8", errors="replace").splitlines()
            next_text = content.decode("utf-8", errors="replace").splitlines()
            diff = difflib.unified_diff(
                current_text,
                next_text,
                fromfile=f"current/{path.relative_to(ROOT)}",
                tofile=f"updated/{path.relative_to(ROOT)}",
                lineterm="",
            )
            for line in diff:
                print(line, file=sys.stderr)
        return True

    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as tmp:
        tmp.write(content)
        tmp_path = Path(tmp.name)
    tmp_path.replace(path)
    return True


def main() -> int:
    args = parse_args()
    assets = [
        VendorAsset(
            key="htmx",
            package="htmx.org",
            package_file="dist/htmx.min.js",
            destination=VENDOR_DIR / "htmx.min.js",
            version_arg=args.htmx,
        ),
        VendorAsset(
            key="alpine",
            package="alpinejs",
            package_file="dist/cdn.min.js",
            destination=VENDOR_DIR / "alpine.min.js",
            version_arg=args.alpine,
        ),
    ]

    entries: dict[str, dict] = {}
    contents: dict[Path, bytes] = {}
    try:
        for asset in assets:
            content, manifest_entry = vendor_asset(asset)
            contents[asset.destination] = content
            entries[asset.key] = manifest_entry
    except (KeyError, tarfile.TarError, urllib.error.URLError, RuntimeError) as err:
        print(f"error: {err}", file=sys.stderr)
        return 1

    changed = False
    for path, content in contents.items():
        changed |= write_or_check(path, content, args.check)
    changed |= write_or_check(MANIFEST_PATH, render_manifest(entries), args.check)

    if args.check and changed:
        return 1

    for key, entry in entries.items():
        action = "checked" if args.check else "vendored"
        print(f"{action} {key} {entry['version']} -> {entry['destination']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
