"""Validate version consistency and first-party documentation before packaging."""
from pathlib import Path
import json
import re
import subprocess
import sys
import tomllib
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parent.parent


def check() -> None:
    package = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))
    tauri = json.loads((ROOT / "src-tauri/tauri.conf.json").read_text(encoding="utf-8"))
    cargo = tomllib.loads((ROOT / "src-tauri/Cargo.toml").read_text(encoding="utf-8"))
    lock = tomllib.loads((ROOT / "src-tauri/Cargo.lock").read_text(encoding="utf-8"))
    version = package["version"]
    assert re.fullmatch(r"\d+\.\d+\.\d+(?:-beta\.[1-9]\d*)?", version), "Use a three-part version with optional -beta.N"
    assert tauri["version"] == cargo["package"]["version"] == version, "Version metadata differs"
    app = [p for p in lock["package"] if p["name"] == cargo["package"]["name"] and "source" not in p]
    assert len(app) == 1 and app[0]["version"] == version, "Cargo.lock app version differs"
    assert (ROOT / "LICENSE").read_text(encoding="utf-8").startswith("MIT License"), "MIT license missing"
    assert tauri["bundle"]["macOS"]["minimumSystemVersion"] == "13.3", "Document macOS requirements consistently"

    files = subprocess.check_output(
        ["git", "ls-files", "--cached", "-z"],
        cwd=ROOT,
    ).decode("utf-8").split("\0")
    docs = sorted({ROOT / f for f in files if f.endswith(".md") and not f.startswith("third-party/") and (ROOT / f).is_file()})
    assert docs, "Documentation missing"
    for path in docs:
        if path.name.endswith(".zh-CN.md"):
            counterpart = path.with_name(path.name.replace(".zh-CN.md", ".md"))
        else:
            counterpart = path.with_name(path.stem + ".zh-CN.md")
        assert counterpart.is_file(), f"Translation missing: {counterpart.relative_to(ROOT)}"
        content = path.read_text(encoding="utf-8")
        # Local Markdown and HTML links are checked without making remote requests.
        links = re.findall(r"\]\(([^\s)]+)\)", content) + re.findall(r'(?:href|src)="([^"]+)"', content)
        for link in links:
            if re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*:", link) or link.startswith("#"):
                continue
            target = unquote(link.split("#", 1)[0])
            if not target:
                continue
            resolved = (path.parent / target).resolve()
            assert resolved.is_relative_to(ROOT), f"Documentation links outside repository: {link}"
            assert resolved.exists(), f"Broken link in {path.relative_to(ROOT)}: {link}"
        if path.name.startswith("README") and "-" not in version:
            assert f"releases/download/v{version}/" in content, "README download links use a different version"
    for suffix in [".md", ".zh-CN.md"]:
        assert (ROOT / "docs/releases" / f"v{version}{suffix}").is_file(), "Release notes missing"
    print(f"Release {version}: metadata, bilingual documents, and local links verified ({len(docs)} documents).")


if __name__ == "__main__":
    try:
        check()
    except (AssertionError, KeyError, ValueError) as error:
        print(f"Release check failed: {error}", file=sys.stderr)
        sys.exit(1)
