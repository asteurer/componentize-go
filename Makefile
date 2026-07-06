.PHONY: ci-checks
ci-checks:
	cargo clippy --all-targets --workspace -- -D warnings
	cargo fmt --all -- --check
	cargo test --workspace