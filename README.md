# Simple Lending App

To understand the framework better, please read the overview in the
[cosmwasm repo](https://github.com/CosmWasm/cosmwasm/blob/master/README.md),
and dig into the [cosmwasm docs](https://www.cosmwasm.com).

## Installation

Install the recent version of rust and cargo (v1.58.1+)
(via [rustup](https://rustup.rs/))

Then run 
```sh
cargo fetch
```
to install dependencies from the Cargo.lock file.

## Production Build
Run this at the root to compile all contracts within the contract/ folder

```shell
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.6
```
