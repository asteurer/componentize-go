# Contributing

This project welcomes contributions for bug fixes, documentation updates, new
features, or whatever you might like. Development is done through GitHub pull
requests. Feel free to reach out on the [Bytecode Alliance
Zulip](https://bytecodealliance.zulipchat.com/) as well if you'd like assistance
in contributing or would just like to say hi.

## Code of Conduct

This is a [Bytecode Alliance](https://bytecodealliance.org/) project, and follows the Bytecode Alliance's [Code of Conduct](CODE_OF_CONDUCT.md).

## License

This project is licensed under the Apache 2.0 license with the LLVM exception.
See [LICENSE](LICENSE) for more details.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be licensed as above, without any additional terms or conditions.

## Notes for Maintainers

### Release process

Bump the version field in [Cargo.toml](/Cargo.toml) and the release variable in [main.go](/main.go) (both should be the same), PR that into main, and tag the result. Once the tag has been pushed to main, the [release](/.github/workflows/release.yml) workflow will automatically create the release artifacts.

