# veldt
Collaborative, self-hosted, web-based digital audio workstation built in Rust

## Structure

* `hydric/` front end
* `mesic/` synthesis engine
* `xeric/` back end

...yes, the three app layers are named after [habitat classifications](https://en.wikipedia.org/wiki/Mesic_habitat) :)

* `shared/` currently just protos
* `experimental` misc experiments and dependency try-ons

## Architecture

Hydric is a Leptos web app, which runs in WASM context. This means it's single threaded, limited to 4GB of RAM, and has limited access to OS APIs like the filesystem. Hydric is just the UI: it talks to mesic in order to make synthesis happen.

Mesic needs to run in both a WASM context - when it's running directly in a browser and talking to hydric - as well as generic server context when it's communicating with xeric, the server. It is therefore agnostic as to the specific build target.

Mesic encapsulates all of the audio processing logic needed by the app. The data model is symmetrically used by both the server and client - so mesic running on the server can handle rendering out audio just as it can on the client. This opens us up to all sorts of fancy features and optimisations, like:

* The client can start rendering a track, and then at the same time, also tell the server (which already has the entire project context) to render the same track. Whichever finishes first will send its results to the other.
* By tracking dependencies between tracks, instruments, samples, effects and so on at a granular level, we can make use of multi-threaded processing on the server. Therefore, the server often has an advantage in its processing time and can help the client along to make rendering faster.
* Once a particular audio sequence is rendered, it then gets pushed to all clients. Client-rendered audio can be shared back with the server in a similar fashion. Therefore, when working on a collaborative session, repeated rendering work is minimized.
* The client can choose to render audio out at a lower quality (to reduce latency), while the server works in the background to produce and ship over higher-quality renderings. The end result is a hybrid of both without straining the client.
* The client only has access to 4 GB of memory - a major limitation of WASM and one reason that web-based clients have historically been less powerful than desktop ones. Veldt however, can happily evict rendering data on the client to free up space because it knows that the server still has a copy.

There are some limitations to be aware of when working with WASM:
* Tokio doesn't work. Anything multi-threaded *especially* doesn't work.
* Communication with the server needs to account for CORS.
* Some dependencies aren't WASM-friendly. Default features need to be disabled, or sometimes the dependencies can't be used at all.

## Commands

### Build and run hydric

```
cd hydric
cargo build --target=wasm32-unknown-unknown
trunk serve --open # note: need to `cargo install trunk` first
```


### Build and run mesic

```
cd mesic
cargo build
cargo run
# then see 'play rendered sounds' below
```
```
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

#### GRPC with envoy proxy

see instructions: https://github.com/grpc/grpc-web/blob/master/net/grpc/gateway/examples/helloworld/README.md

grpc-web is in the experimental folder.

#### Play rendered sounds

Install ffplay first.

```
cd mesic
ffplay -f f32le -ar 48000 -showmode 1 out.bin
```
