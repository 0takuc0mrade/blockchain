# Continuous integration

The GitHub Actions CI workflow runs formatting, Clippy with warnings denied,
the workspace tests, and an optimized WebAssembly build using the toolchain
declared in [`rust-toolchain.toml`](../rust-toolchain.toml).

The optimized ticketing contract WASM must remain at or below **524,288 bytes
(512 KiB)**. The limit is enforced in `.github/workflows/ci.yml`; a build over
the limit fails CI. Update this documented limit deliberately when contract
functionality or the deployment budget changes.

The separate security workflow runs `cargo audit` weekly. When the scheduled
audit fails, it opens a GitHub issue (while avoiding duplicate open reports).
