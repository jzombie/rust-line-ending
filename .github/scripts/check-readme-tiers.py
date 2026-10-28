#!/usr/bin/env python3
"""Verify README tier annotations by executing every Rust example
under its annotated feature set, and by compiling it under a
strict tier prelude.

Convention enforced on README.md:
- Every ```rust block must be preceded (nearest tier note above it) by one of:
    * "`no_std`-compatible: core only"            -> core tier
    * "Requires the `alloc` feature"              -> alloc tier
    * "Requires `std`"                            -> std tier
- Each tier's blocks go into a scratch crate depending on line-ending
  with the matching features, in two forms:
    * tests/readme.rs — blocks as #[test] fns, run with `cargo test`.
      Catches API drift and assertion rot. (The test harness itself
      links std, so this phase cannot prove prelude purity.)
    * src/lib.rs — blocks as plain `pub fn`s under a strict prelude
      (`#![no_std]`; plus `extern crate alloc` and explicit alloc
      imports for the alloc tier), checked with `cargo check`.
      Catches std/alloc leakage in the example bodies themselves.
  A block using an API outside its tier fails one phase or the other,
  so drift between annotation and code is caught either way.

Usage: python3 .github/scripts/check-readme-tiers.py  (run from repo root)
"""

import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
README = ROOT / "README.md"

CORE_MARK = "`no_std`-compatible: core only"
ALLOC_MARK = "Requires the `alloc` feature"
STD_MARK = "Requires `std`"


def extract_blocks(readme: Path):
    """Return [(tier, body)] for every ```rust block in document order."""
    blocks = []
    tier = None
    in_fence = False
    buf = []
    for line in readme.read_text().splitlines():
        stripped = line.strip()
        if stripped == "```rust":
            if in_fence:
                sys.exit("error: nested ```rust fence")
            in_fence = True
            buf = []
        elif stripped == "```" and in_fence:
            in_fence = False
            if tier is None:
                sys.exit(
                    "error: ```rust block has no tier annotation above it:\n"
                    + "\n".join(buf[:5])
                )
            blocks.append((tier, "\n".join(buf)))
        elif in_fence:
            buf.append(line)
        else:
            if CORE_MARK in line:
                tier = "core"
            elif ALLOC_MARK in line:
                tier = "alloc"
            elif STD_MARK in line:
                tier = "std"
    if in_fence:
        sys.exit("error: unclosed ```rust fence")
    return blocks


def write_crate(parent: Path, tier: str, bodies: list[str]):
    crate = parent / f"readme-tier-{tier}"
    (crate / "src").mkdir(parents=True)
    (crate / "tests").mkdir(parents=True)
    dep = f'line-ending = {{ path = "{ROOT}", default-features = false }}'
    if tier == "alloc":
        dep = f'line-ending = {{ path = "{ROOT}", default-features = false, features = ["alloc"] }}'
    elif tier == "std":
        dep = f'line-ending = {{ path = "{ROOT}" }}'
    (crate / "Cargo.toml").write_text(
        f'[package]\nname = "readme-tier-{tier}"\nversion = "0.0.0"\nedition = "2021"\n\n'
        f"[dependencies]\n{dep}\n"
    )
    # Strict phase (src/lib.rs): exact tier prelude, no test harness.
    # Catches std/alloc leakage in the example bodies.
    if tier == "core":
        lib_prelude = "#![no_std]\n#![allow(unused_imports)]\n\n"
    elif tier == "alloc":
        lib_prelude = (
            "#![no_std]\n"
            "#![allow(unused_imports)]\n"
            "extern crate alloc;\n"
            "use alloc::string::{String, ToString};\n"
            "use alloc::vec::Vec;\n"
            "use alloc::vec;\n"
            "use alloc::{boxed::Box, format};\n\n"
        )
    else:
        lib_prelude = ""
    lib = [lib_prelude]
    for i, body in enumerate(bodies):
        lib.append(f"pub fn readme_block_{i}() {{\n{body}\n}}\n")
    (crate / "src" / "lib.rs").write_text("\n".join(lib))
    # Execution phase (tests/readme.rs): harness links std, so this
    # proves API availability and assertion correctness, not purity.
    if tier == "alloc":
        # Test-crate prelude: vec!/String come from alloc here, not std.
        test_prelude = "#[macro_use]\nextern crate alloc;\n\n"
    else:
        test_prelude = ""
    tests = [test_prelude]
    for i, body in enumerate(bodies):
        tests.append(f"#[test]\nfn readme_block_{i}() {{\n{body}\n}}\n")
    (crate / "tests" / "readme.rs").write_text("\n".join(tests))
    return crate


def main() -> int:
    blocks = extract_blocks(README)
    by_tier: dict[str, list[str]] = {"core": [], "alloc": [], "std": []}
    for tier, body in blocks:
        by_tier[tier].append(body)
    print(f"README rust blocks: {len(blocks)} " + str({k: len(v) for k, v in by_tier.items()}))

    tmp = Path(tempfile.mkdtemp(prefix="readme-tiers-"))
    try:
        for tier, bodies in by_tier.items():
            if not bodies:
                print(f"[{tier}] no blocks, skipping")
                continue
            crate = write_crate(tmp, tier, bodies)
            print(f"[{tier}] strict check ({len(bodies)} blocks) ...", flush=True)
            r = subprocess.run(
                ["cargo", "check", "--quiet"], cwd=crate, capture_output=True, text=True
            )
            if r.returncode != 0:
                print(f"[{tier}] STRICT CHECK FAILED:\n{r.stdout}\n{r.stderr}")
                return 1
            print(f"[{tier}] strict check ok")
            print(f"[{tier}] testing {len(bodies)} blocks ...", flush=True)
            r = subprocess.run(
                ["cargo", "test", "--quiet"], cwd=crate, capture_output=True, text=True
            )
            if r.returncode != 0:
                print(f"[{tier}] FAILED:\n{r.stdout}\n{r.stderr}")
                return 1
            m = re.findall(r"test result: ok\. (\d+) passed", r.stdout + r.stderr)
            print(f"[{tier}] ok ({sum(map(int, m)) if m else '?'} passed)")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print("All README tier annotations check out.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
