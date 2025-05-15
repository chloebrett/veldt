#!/usr/bin/env python3
import os
import subprocess

root_dir = os.path.dirname(__file__)

os.environ.update(
    {"RUSTFLAGS": "-C target-feature=+atomics,+bulk-memory,+mutable-globals"}
)

subprocess.run(
    [
        "cargo",
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "--release",
        "-Zbuild-std=std,panic_abort",
    ],
    cwd=root_dir,
).check_returncode()

subprocess.run(
    [
        "wasm-bindgen",
        os.path.join(
            root_dir,
            "..",
            "..",
            "target",
            "wasm32-unknown-unknown",
            "release",
            "rng.wasm",
        ),
        "--out-dir",
        os.path.join(root_dir, "pkg"),
        "--target",
        "web",
    ],
    cwd=root_dir,
).check_returncode()
