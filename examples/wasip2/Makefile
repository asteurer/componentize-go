generate-bindings:
	@componentize-go --world wasip2-example bindings --format
	@go mod tidy

build-component: generate-bindings
	@componentize-go --world wasip2-example build

.PHONY: run
run: build-component
	@wasmtime serve -Sp3,cli main.wasm

build-tests: generate-bindings
	@componentize-go test --wasip1 --pkg ./unit_tests_should_pass --pkg ./unit_tests_should_fail

run-tests: build-tests
	@echo "===== Running tests that will pass ====="
	@wasmtime run test_unit_tests_should_pass.wasm
	@echo ""

	@echo "===== Running tests that will fail ====="
	@wasmtime run test_unit_tests_should_fail.wasm || true
	@echo ""
