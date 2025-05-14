#!/usr/bin/env python3
import os
import subprocess

root_dir = os.path.dirname(__file__)

os.environ.update(
    {"RUSTFLAGS": "-C target-feature=+atomics,+bulk-memory,+mutable-globals"}
)

# Build the WASM target
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


def run_native():
    """
    Builds and runs the native Rust code (the main function).
    This is separate from the WASM build process.
    """
    subprocess.run(
        ["cargo", "build"],  # Build the native target (debug build)
        cwd=root_dir,
    ).check_returncode()
    subprocess.run(
        ["cargo", "run"],  # Run the native executable
        cwd=root_dir,
    ).check_returncode()

if __name__ == "__main__":
    # Build WASM
    print("Building WASM...")
    # The WASM build commands are already in the main part of the script.
    # Run the native code.
    print("\nRunning native Rust code...")
    run_native()
