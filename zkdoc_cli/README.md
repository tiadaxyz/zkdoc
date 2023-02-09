# `zkdoc_cli`

See it in action!

![gif](./cli_gif.gif)

## Usage
```bash
# Build a release for core_cli
cargo build -p core_cli --release
```
```bash
ZKDoc CLI

Usage: zkdoc_cli [COMMAND]

Commands:
  gen-commitment  Generates a commitment for a given file
  gen-proof       Generates a proof for a given file
  verify-proof    Verifies a proof against a given commitment
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
