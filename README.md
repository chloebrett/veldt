# veldt
Collaborative, self-hosted, web-based digital audio workstation built in Rust

## Structure

* `hydric/` front end
* `mesic/` synthesis engine
* `xeric/` back end

## Commands

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

### Build shared

Only if necessary - it will be built transitively by xeric/hydric when needed.

```
cd shared
cargo build # for xeric
cargo build --target=wasm32-unknown-unknown # for hydric
```
### Experiments / demos

GRPC with envoy proxy: see instructions: https://github.com/grpc/grpc-web/blob/master/net/grpc/gateway/examples/helloworld/README.md

grpc-web is in the experimental folder.
