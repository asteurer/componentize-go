GO := ../../go-$(shell uname -s | tr 'A-Z' 'a-z')-$(shell uname -m | sed -e 's/aarch64/arm64/' -e 's/x86_64/amd64/')-bootstrap

$(GO)/bin/go:
	curl -OL https://github.com/dicej/go/releases/download/go1.25.5-wasi-on-idle/$(GO).tbz
	tar xf $(GO).tbz

generate-bindings: $(GO)/bin/go
	wit-bindgen go -w wasip3-example --out-dir . ./wit
	sed -i 's|github.com/bytecodealliance/wit-bindgen/crates/go/src/package v0.50.0|../../../wit-bindgen/crates/go/src/package|' ./go.mod
	$(GO)/bin/go mod tidy

build-component: generate-bindings
	cargo build --release --manifest-path ../../Cargo.toml
	../../target/release/componentize-go --world wasip3-example componentize --go $(GO)/bin/go

.PHONY: run
run: build-component
	wasmtime serve -Sp3,cli -Wcomponent-model-async main.wasm
