# zkdoc

This repository contains the source code for `zkdoc_sdk`, `zkdoc_cli` and `zkdoc_server`.

## `zkdoc_sdk`

### Usage

## `zkdoc_cli`

### Usage
```bash
# Build a release for core_cli
cargo build -p core_cli --release
```
```bash
Medi0 CLI

Usage: core_cli [COMMAND]

Commands:
  gen-commitment  Generates a commitment for a given file
  gen-proof       Generates a proof for a given file
  verify-proof    Verifies a proof against a given commitment
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## `zkdoc_server`

This is not published on crates.io, but if you would like to run your own version, you can simply clone this repository down and run it yourself.
The server is powered by `zkdoc_sdk`.

### Usage

After cloning the repository, simple do:

```bash
cargo run -p zkdoc_server
```

With that, you should have a server running at port `8080`.
