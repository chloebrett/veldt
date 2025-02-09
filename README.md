# veldt
Collaborative, self-hosted, web-based digital audio workstation built in Rust

## Structure

* `hydric/` front end
* `mesic/` synthesis engine
* `xeric/` back end

## Commands

### Build shared

```
cd shared
cargo build # for xeric
cargo build --target=wasm32-unknown-unknown # for hydric
```

### Build and run hydric

```
cd hydric
cargo build --target=wasm32-unknown-unknown
trunk serve --open # note: need to `cargo install trunk` first
```

### Build and run xeric

```
cd xeric
cargo build
cargo run
```
