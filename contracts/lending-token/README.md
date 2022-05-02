# Lending Token

Lending token for the Lending protocol, follows CW20 token standard.

## Building

TODO figure out how to get optimized wasm binary, step below doesn't work... (optimized eventually needed for bombay deployment)
For a production-ready (optimized) build, run the following build command: https://github.com/CosmWasm/cw-plus#compiling.

Run `cargo wasm` in this directory to compile the lending_token.wasm file, which can be used in localterra testing. The binary will be found in the root directory's `arget/wasm32-unknown-unknown/release/` folder.