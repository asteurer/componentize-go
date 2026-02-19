<div align="center">
  <h1><code>componentize-go</code></h1>

  <p>
    <strong>Build WebAssembly components with Go</strong>
  </p>

  <strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

  <p>
    <a href="https://github.com/bytecodealliance/componentize-go/actions?query=workflow%3ACI"><img src="https://github.com/bytecodealliance/componentize-go/workflows/CI/badge.svg" alt="build status" /></a>
  </p>
</div>

This is a tool to convert a Go application to a [WebAssembly component](https://github.com/WebAssembly/component-model). It takes the following as input:

- a [WIT](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md) file or directory
- the name of a [WIT world](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md#wit-worlds) defined in the above file or directory
- the directory containing a Go module which targets said world

The output is a component which may be run using e.g. [`wasmtime`](https://github.com/bytecodealliance/wasmtime).

## Installation

### Download a release

You can download a specific release from the [release page](https://github.com/bytecodealliance/componentize-go/releases).

### Build from source

#### Prerequisites

- [**Rust toolchain**](https://rust-lang.org/) - Latest version

#### Run

```sh
cargo install --git https://github.com/bytecodealliance/componentize-go
```

## Usage

Please reference the `README.md` and `Makefile` files in each of the directories in [examples](./examples/).
