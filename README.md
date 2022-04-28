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

## Gitpod integration

[Gitpod](https://www.gitpod.io/) container-based development platform will be enabled on your project by default.

Workspace contains:
 - **rust**: for builds
 - [wasmd](https://github.com/CosmWasm/wasmd): for local node setup and client
 - **jq**: shell JSON manipulation tool

Follow [Gitpod Getting Started](https://www.gitpod.io/docs/getting-started) and launch your workspace.
