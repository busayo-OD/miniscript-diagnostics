# miniscript-diagnostics

A Rust library and CLI that explains Miniscript satisfaction state under a supplied spending context.

It walks the public `rust-miniscript` AST and produces a diagnostic tree showing why each supported fragment is `SATISFIED`, `UNAVAILABLE`, or `IMPOSSIBLE`. Fragments outside the supported set are reported as `UNSUPPORTED`.

## Example

```text
AND_V [UNAVAILABLE]
├── pk(...) [SATISFIED]
└── older(144) [UNAVAILABLE]
    required: 144 blocks
    available: 83 blocks
    remaining: 61 blocks

ROOT STATUS: UNAVAILABLE
````

## Supported

* `pk`, `pkh`
* `sha256`, `hash256`, `ripemd160`, `hash160`
* `after`, `older`
* `and_v`, `and_b`, `or_i`
* `thresh(k, ...)`

## Usage

```bash
cargo run -- \
  'pk(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)' \
  --key 0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798
```

Run `cargo run -- --help` for available options.

Use `--json` for machine-readable output.

## Development

```bash
cargo test
```

See [ARCHITECTURE.md](docs/architecture.md) for design details.
