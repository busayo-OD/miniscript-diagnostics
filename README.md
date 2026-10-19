# miniscript-diagnostics

A Rust library and CLI that explains Miniscript satisfaction state under a supplied spending context.

It walks the public `rust-miniscript` AST and produces a diagnostic tree showing why each supported fragment is `SATISFIED`, `UNAVAILABLE`, or `IMPOSSIBLE`. Fragments outside the supported set are reported as `UNSUPPORTED`. Each diagnostic node includes a reason when applicable, explaining why that status was reached.

This tool explains satisfaction feasibility under a supplied context; it does not generate witnesses or sign transactions.

## Example

```text
AND_V [UNAVAILABLE]
├── pk(...) [SATISFIED]
└── older(144) [UNAVAILABLE]
    required: 144 blocks
    available: 83 blocks
    remaining: 61 blocks

ROOT STATUS: UNAVAILABLE
```

## Supported fragments

- `pk`, `pkh`
- `sha256`, `hash256`, `ripemd160`, `hash160`
- `after`, `older`
- `and_v`, `and_b`, `or_i`
- `or_b`, `or_c`, `or_d`, `andor`
- `thresh(k, ...)`, `multi`, `multi_a`

## CLI usage

```bash
cargo run -- \
  'pk(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)' \
  --key 0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798
```

Run `cargo run -- --help` for available options.

Use `--json` for machine-readable output.

## Library usage

You can use `miniscript-diagnostics` as a Rust library to evaluate a Miniscript expression against a supplied spending context.

```rust
use miniscript::bitcoin::PublicKey;
use miniscript_diagnostics::{parse_and_evaluate, DiagnosticContext};

fn main() {
    let supplied_key: PublicKey =
        "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
            .parse()
            .expect("valid public key");

    let context = DiagnosticContext::new().with_key(supplied_key);

    let diagnostic = parse_and_evaluate(
        "or_i(pk(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798),pk(0245d3b9ce0f54f4d6a17edfe3f9e0993b94d6b299c1a6e5a728ff036ecd9e139f))",
        &context,
    )
    .expect("evaluation should succeed");

    println!("{diagnostic:#?}");
}
```

In this example, the overall expression is `Satisfied`. The first branch is `Satisfied` because its public key was supplied, while the second is `Impossible` because its public key was not supplied in the context. The diagnostic tree preserves the status and reason for each branch.

## Development

Run the test suite with:

```bash
cargo test
```

See [Architecture](docs/architecture.md) for design details.
