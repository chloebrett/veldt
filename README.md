# veldt
Collaborative, self-hosted, web-based digital audio workstation built in Rust

## Structure

* `hydric/` front end
* `mesic/` synthesis engine
* `xeric/` back end

...yes, the three app layers are named after [habitat classifications](https://en.wikipedia.org/wiki/Mesic_habitat) :)

* `shared/` contains the data model
* `state/` contains the state store, which contains all of the application state that is relevant to undo/redo and collaborative editing. This includes all of the project state, but excludes things like window states.

## Architecture

Hydric is a egui web app, which runs in WASM. This means it's single threaded, limited to 4GB of RAM, and has limited access to OS APIs like the filesystem. Hydric is just the UI: it talks to mesic in order to make synthesis happen.

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
* The client has only 4 GB of memory.

## Install dependencies

```
# Protobuf compiler
apt install -y protobuf-compiler # OS dependent, see https://grpc.io/docs/protoc-installation/

# Yarn (used for easy pre-commit hooks and other commands only)
sudo npm i -g yarn
# Note: on Linux you may have the "wrong yarn" to begin with.
# If this happens, uninstall it: https://stackoverflow.com/questions/53471063/yarn-error-there-are-no-scenarios-must-have-at-least-one

# Yarn deps
yarn

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # Or see https://www.rust-lang.org/tools/install
# Then run `rustup` and follow instructions to install latest rust.

# WASM target
rustup target add wasm32-unknown-unknown

# Clippy
rustup component add clippy

# Trunk
cargo install trunk
```

## Commands

```
# Pre-commit hook (will also run automatically before each commit)
yarn p

# Building
yarn b # everything
yarn hb # hydric
yarn mb # mesic
yarn sb # shared
yarn stb # state
yarn xb # xeric

# Running
yarn r # everything (hydric and xeric)
yarn hr # hydric - note: visit localhost:8080
yarn xr # xeric
# Note: mesic, shared, and state can't be run as they're libraries.
# They get built by implication when hydric/xeric are run.

# Lint
yarn hl # hydric
yarn ml # mesic
yarn sl # shared
yarn st # state
yarn xl # xeric
yarn l # all

# Testing
yarn t # everything (currently excluding Hydric)
yarn mt # mesic
yarn st # shared
yarn stt # state
yarn xt # xeric
```
