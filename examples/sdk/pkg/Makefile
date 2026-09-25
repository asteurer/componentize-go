# The export_wasi_cli_run files are hand-written, and thus are not removed
generate-bindings:
	@rm -rf bindings/imports
	@rm -rf bindings/exports/wit_exports
	@componentize-go \
		--ignore-toml-files \
		bindings \
		--output bindings \
		--pkg-name pkg/bindings \
		--format
