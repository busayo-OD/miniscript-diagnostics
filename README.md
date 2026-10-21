# miniscript-diagnostics

A Rust library and CLI that explains Miniscript satisfaction state under a supplied spending context.

It walks the public `rust-miniscript` AST and produces a diagnostic tree showing why each supported fragment is `SATISFIED`, `UNAVAILABLE`, or `IMPOSSIBLE`. Fragments outside the supported set are reported as `UNSUPPORTED`. Each diagnostic node includes a reason when applicable, explaining why that status was reached.

This tool explains satisfaction feasibility under a supplied context; it does not generate witnesses or sign transactions.

For details on the design and evaluation flow, see the [Architecture documentation](docs/architecture.md).

## Example

```text
AND_V [UNAVAILABLE]
    PK [SATISFIED]
    OLDER [UNAVAILABLE]
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

`multi_a` is supported by the evaluator only when it is given a Miniscript AST that uses the Tap context. `parse`, `parse_and_evaluate` and the CLI are fixed to the Segwitv0 context and reject `multi_a`. General Taproot/Tapscript support is future work.

The wrappers `a:`, `s:`, `c:`, `d:`, `v:`, `j:` and `n:` are evaluated transparently and do not appear as separate nodes in the diagnostic tree.

### Unsupported fragments

The fragments `0` and `1`, and everything that expands to them (`and_n`, `t:`, `l:` and `u:`) are reported as `UNSUPPORTED`. An unsupported child makes its parent `UNSUPPORTED` unless the other children already decide the result: in an AND-type fragment an `IMPOSSIBLE` sibling still makes the result `IMPOSSIBLE`, and in an OR-type fragment a `SATISFIED` sibling still makes it `SATISFIED`. For example, `and_n(X,Y)` is `andor(X,Y,0)`, so it is `UNSUPPORTED` unless both `X` and `Y` are satisfied, even when `X` is impossible.

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

Install [Just](https://just.systems/) to run the project's development commands.

```bash
just p
```

This runs the pre-push checks, including formatting, compilation, linting, tests, and documentation checks.

See the [justfile](justfile) for all available commands.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License

This project is licensed under [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/). See [LICENSE](LICENSE) for details.