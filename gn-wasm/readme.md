## Wasm bindings for interacting with Guild Network from the browser
There are two main test suites each with it's own caveat. By default, the
`queries` test module is hidden behind the respective `queries` feature flag.
This is because those tests require the `join` example to be successfully run
in the `gn-client` package. On the other hand, the `transactions` test module
is always run when running the tests.

### Running only `transactions` tests

- compile the test node by running `cargo build --release` in the workspace root
- start the node by running `./start.sh dev`
- in a separate terminal, start an oracle instance by running
```
cd gn-oracle
cargo run --release -- --log trace --register
```
- run the wasm tests
```
cd gn-wasm
WASM_BINDGEN_TEST_TIMEOUT=60 wasm-pack test --firefox --headless
```

### Running all tests
- compile the test node by running `cargo build --release` in the workspace root
- start the node by running `./start.sh dev`
- in a separate terminal, start an oracle instance by running
```
cd gn-oracle
cargo run --release -- --log trace --register
```
- in a separate terminal run the example in `gn-client`
```
cd gn-client
cargo run --release --example guild --features external-oracle -- --example join
```
- after the example in `gn-client` has successfully completed run the wasm tests
```
cd gn-wasm
WASM_BINDGEN_TEST_TIMEOUT=60 wasm-pack test --firefox --headless --features queries
```
