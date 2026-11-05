# Massa smart-contract examples in Rust

Example contracts written with [massa-sc-sdk](https://github.com/massalabs/massa-rust-sc-sdk),
each with native unit tests and a TypeScript script that deploys and drives it
with [massa-web3](https://github.com/massalabs/massa-web3) on buildnet (or any
node, including a local C-Layer sandbox).

| contract | shows | script |
|---|---|---|
| [`hello-world`](contracts/hello-world) | the smallest contract: `main` for `ExecuteSC`, `constructor`, an entry point returning a string, events | `npm run hello` |
| [`counter`](contracts/counter) | typed storage, `Args` in/out, `#[derive(Serializable)]` return struct, `Result` entry points, owner checks, mock-VM tests | `npm run counter` |
| [`token`](contracts/token) | MRC20 (mintable, burnable, ownable) from `massa-sc-standards` + a custom `airdrop`; driven by massa-web3's `MRC20` wrapper | `npm run token` |
| [`nft`](contracts/nft) | MRC721 enumerable + metadata, paid public mint with coin refund, owner withdraw | `npm run nft` |
| [`multi-token`](contracts/multi-token) | MRC1155 with per-token metadata, `minter` role (role-based access control), burn | `npm run multi-token` |
| [`vault`](contracts/vault) | cross-contract calls through an `Mrc20` client (allowance + `transferFrom`), storage-cost forwarding, deploying an embedded child contract, `local_call` | `npm run vault` |
| [`autonomous`](contracts/autonomous) | a self-rescheduling deferred call (guaranteed autonomous execution), cancel, legacy async messages | `npm run autonomous` |
| [`deployer`](contracts/deployer) | `ExecuteSC` reading bytecode / constructor args / coins from the operation datastore (port of the standard deployer) | `npm run deployer` |
| [`crypto`](contracts/crypto) | blake3 / sha256 / keccak256, Massa signature verification, EVM address recovery, address helpers | `npm run crypto` |
| [`clayer-mimc`](contracts/clayer-mimc) | C-Layer specific ABI (`mimc` hash), commit/reveal; only instantiable on C-Layer nodes | `npm run clayer-mimc` |

## Build and test

```sh
rustup target add wasm32v1-none
cargo install --git https://github.com/massalabs/massa-rust-sc-sdk cargo-massa
cargo install wasm-opt              # optional: smaller modules, less gas

cargo test                          # unit tests of every contract, natively (mock VM)
cargo massa build                   # build/*.wasm, optimized and validated for the node
```

`cargo massa build` prints, for each module, its size, imports, entry points
and any reason a node would reject it. Modules land in `build/`.

Run a module locally in the real Massa engine without a node:

```sh
cargo install --git https://github.com/massalabs/massa-rust-sc-sdk massa-sc-runner
massa-sc-runner build/counter.wasm constructor --args u64:10,u64:5 --deploy \; increment --args u64:3 \; get
```

## Deploy on buildnet

```sh
cd scripts && npm install
cp .env.example .env            # set PRIVATE_KEY to a funded buildnet account
npm run counter                 # or hello, token, nft, multi-token, vault, autonomous, deployer, crypto
npm run all
```

Each script deploys the contract (sending 0.1 MAS to pay for what the
constructor writes), calls it, reads it, and prints the events. Get buildnet
coins from the [faucet](https://discord.gg/massa) (`#buildnet-faucet`).

## Run on a local C-Layer sandbox

[C-Layer](https://github.com/massalabs/c-layer) is a Massa fork with a few extra
host functions (`clayer-mimc` needs `assembly_script_hash_mimc`). To test
against it, build its node with the `sandbox` feature (`cargo build --release
--features sandbox -p massa-node -p massa-client`), give your address coins and
rolls in `massa-node/base_config/initial_ledger.json` / `initial_rolls.json`,
set `bootstrap_list = []` in `massa-node/config/config.toml`, start the node
and `node_start_staking <address>` with the client. Then:

```sh
JSON_RPC_URL=http://127.0.0.1:23035 PRIVATE_KEY=S1... npm run clayer-mimc
```

Every other example runs there too (same ABI); the sandbox has 0.5 s slots,
so things are fast. Note that C-Layer nodes do not expose the MIP-0002
paginated key ABIs yet: `cargo massa build` warns when a module imports them.

## Layout

```
contracts/<name>/       one crate per example (cdylib + rlib for tests)
scripts/src/<name>.ts   massa-web3 script deploying and calling it
build/                  output of cargo massa build (ignored)
```

Contracts depend on the SDK with `git = "https://github.com/massalabs/massa-rust-sc-sdk.git"`
(see the workspace `Cargo.toml`); pin a tag in your own projects.
