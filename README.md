Examples of how to use massa-rust-sc-sdk


# Workaround
cargo make does not seems to enforce build order from `workspace.members` so one has to do:
```
cd generate_event && cargo make wasm --no-workspace
```
first.

# Dependencies

In the project directory (so the target is added to the current toolchain), run:

```
rustup target add wasm32-unknown-unknown
```

```
cargo install wasmer-cli  --features cranelift
```

```
wasm-strip
wasm-opt (apt install bunaryen)
         (cargo install wasm-opt --locked)
```

# Build

## candidate 1
/ ! \\ does not seem to make it, see candidate 2 (more raw but also more promising)
Kept as a source of information
`wasm-pack build --target web`

MAYBE useless if project is build with wasm-pack

## candidate 2
/ ! \\ this is fine but it breaks testing feature in vscode, so better set the target at build time.
to force the target to wasm add this file to the Rust project.

```toml
File: .cargo/config.toml
────────────────────────
[build]
target = "wasm32-unknown-unknown"
```

## candidate 3
Set the target explicitly in `Makefile.toml` and build with `cargo make`

## build command
We need a post build hook to prepend the `header` (0x01) to the produced `.wasm` file.
AFAIK `cargo` does not have this feature. `cargo make` solves this problem in a simple way.
(maybe cargo xtask should be investigated)

to install `cargo make`
```shell
cargo install cargo-make
```

Classical `cargo` commands are still working, in order to fully build the projet in one command use:

to build the project
```shell
cargo make wasm
```

## test command
If the default target set in `.cargo/config.toml` is `wasm32-unknown-unknown`,
one need to set a different one for test to work, example:

```shell
cargo test -p massa-rust-sc-examples --lib --target x86_64-unknown-linux-gnu
```

# various
Experimental Rust is required, at least very useful to have `cargo expand` out of the box,
hence the `rust-toolchain.toml`.
```toml
File: rust-toolchain.toml
─────────────────────────
[toolchain]
channel = "nightly-2023-02-27"
```

```shell
cargo make build
```
will build all examples

```shell
cargo make build -p <package_name>
```
will build only `package_name`

As each example contains its own `Makefile.toml` they should work (aka build) outside the workspace (not tested yet).


# Code
The sdk is `no_std`, this implies some manual imports from the `alloc` crate.
For the shake of simplicity the sdk tries to reexport common structs / functions...
If some are missing, for example `ToOwned`, it can be added manualy this way

```rust
extern crate alloc;
use alloc::borrow::ToOwned;
```