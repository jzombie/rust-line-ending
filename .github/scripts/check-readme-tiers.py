#!/usr/bin/env python3
"""Verify README tier annotations by executing every Rust example
under its annotated feature set, and by compiling it under a
strict tier prelude.

Convention enforced on README.md:
- Every ```rust block must carry its OWN tier note directly above it
  (exactly one of the three strings below). A note applies to the
  single block that follows it and is then consumed: a reworded note
  matches nothing, so the next block has no tier and the script exits
  non-zero naming the offending block. There is no inheritance and
  no silent skipping.
    * "`no_std`-compatible: core only"            -> core tier
    * "Requires the `alloc` feature"              -> alloc tier
    * "Requires `std`"                            -> std tier
  Fences with suffixes are honored the way rustdoc honors them:
    * ```rust,ignore blocks are skipped entirely (never compiled).
    * ```rust,no_run blocks are compile-checked in src/lib.rs but
      never executed (no #[test] emitted).
  Every fence — including suffixed ones — must still carry its own
  tier note, so flipping a block to runnable later cannot silently
  land it in the wrong tier.
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
    """Return [(tier, body, directives)] for every ```rust block, in order.

    directives is the fence suffix set, e.g. {"no_run"} for
    ```rust,no_run (empty for plain ```rust).

    Strict consume-on-use: a tier note arms exactly one following block.
    Anything else (missing note, reworded note, unclosed fence) is a
    hard error naming the block.
    """
    blocks = []
    tier = None
    in_fence = False
    fence_first_line = 0
    directives: set[str] = set()
    buf = []
    for lineno, line in enumerate(readme.read_text().splitlines(), start=1):
        stripped = line.strip()
        if stripped == "```rust" or stripped.startswith("```rust,"):
            if in_fence:
                sys.exit(f"error: nested ```rust fence at line {lineno}")
            if tier is None:
                sys.exit(
                    f"error: ```rust block at line {lineno} has no tier "
                    "annotation above it"
                )
            in_fence = True
            fence_first_line = lineno
            directives = (
                set(stripped[len("```rust") + 1 :].split(",")) if "," in stripped else set()
            )
            buf = []
        elif stripped == "```" and in_fence:
            in_fence = False
            blocks.append((tier, "\n".join(buf), directives))
            tier = None  # consumed: each block needs its own note
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
        sys.exit(f"error: unclosed ```rust fence opened at line {fence_first_line}")
    return blocks


def write_crate(parent: Path, tier: str, bodies: list[tuple[str, set[str]]]):
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
    lib_n = 0
    for body, directives in bodies:
        if "ignore" in directives:
            continue  # rustdoc contract: never compiled
        lib.append(f"pub fn readme_block_{lib_n}() {{\n{body}\n}}\n")
        lib_n += 1
    (crate / "src" / "lib.rs").write_text("\n".join(lib))
    # Execution phase (tests/readme.rs): harness links std, so this
    # proves API availability and assertion correctness, not purity.
    if tier == "alloc":
        # Test-crate prelude: vec!/String come from alloc here, not std.
        test_prelude = "#[macro_use]\nextern crate alloc;\n\n"
    else:
        test_prelude = ""
    tests = [test_prelude]
    test_n = 0
    for body, directives in bodies:
        if "ignore" in directives or "no_run" in directives:
            continue  # rustdoc contract: ignore = never compiled,
            # no_run = compiled (lib phase) but never executed
        tests.append(f"#[test]\nfn readme_block_{test_n}() {{\n{body}\n}}\n")
        test_n += 1
    (crate / "tests" / "readme.rs").write_text("\n".join(tests))
    return crate, len(bodies), lib_n, test_n


def main() -> int:
    blocks = extract_blocks(README)
    by_tier: dict[str, list[tuple[str, set[str]]]] = {"core": [], "alloc": [], "std": []}
    for tier, body, directives in blocks:
        by_tier[tier].append((body, directives))
    print(f"README rust blocks: {len(blocks)} " + str({k: len(v) for k, v in by_tier.items()}))

    tmp = Path(tempfile.mkdtemp(prefix="readme-tiers-"))
    try:
        for tier, bodies in by_tier.items():
            if not bodies:
                print(f"[{tier}] no blocks, skipping")
                continue
            crate, total, lib_n, test_n = write_crate(tmp, tier, bodies)
            skipped = total - lib_n
            norun = lib_n - test_n
            print(
                f"[{tier}] strict check ({lib_n} blocks"
                + (f", {skipped} ignored" if skipped else "")
                + ") ...",
                flush=True,
            )
            r = subprocess.run(
                ["cargo", "check", "--quiet"], cwd=crate, capture_output=True, text=True
            )
            if r.returncode != 0:
                print(f"[{tier}] STRICT CHECK FAILED:\n{r.stdout}\n{r.stderr}")
                return 1
            print(f"[{tier}] strict check ok")
            print(
                f"[{tier}] testing {test_n} blocks"
                + (f" ({norun} no_run, compile only)" if norun else "")
                + " ...",
                flush=True,
            )
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
