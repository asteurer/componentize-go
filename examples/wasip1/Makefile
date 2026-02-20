build:
	@componentize-go build --wasip1

.PHONY: run
run: build
	@wasmtime run main.wasm
