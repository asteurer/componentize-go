# Overview of componentize-go's Integration Tests

The integration tests have been separated into 2 categories:
- Examples: Run the components in the /examples directory
- Fixtures: Run any non-example components in the /tests/fixtures directory

## Running the Integration Tests

Run:
```sh
cargo test --workspace
```

## How wasmtime is used in tests

Most of the tests just use the Wasmtime CLI; however, there are some fixture tests that require a custom Wasmtime implementation. If you have a fixture or example test that needs a custom Wasmtime impl, feel free to use the [memory-pressure](/tests/src/fixtures.rs) test as a reference.
