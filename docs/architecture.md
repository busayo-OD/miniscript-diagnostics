# Architecture

`miniscript-diagnostics` is a diagnostic layer over the public API of `rust-miniscript`.

## Overview

```text
Miniscript
    │
    ▼
Parser
    │
    ▼
Miniscript AST
    │
    ▼
Diagnostic Evaluator
    │
    ▼
Diagnostic Tree
    │
    ├── Library API
    └── CLI
````

`rust-miniscript` remains responsible for parsing and Miniscript semantics. This project walks the resulting AST and produces a diagnostic tree.

## Components

### `context.rs`

Provides the spending context used during evaluation:

* available keys
* available preimages
* chain height and time
* relative timelock state

### `evaluator.rs`

Recursively walks the public Miniscript AST, evaluates supported fragments, and generates diagnostic reasons explaining their current status. It does not perform signing, witness generation, transaction construction, or signature verification.

### `diagnostic.rs`

Defines the diagnostic model:

* `Status`
* `Diagnostic`
* metadata
* child diagnostics
* status-combination logic
* diagnostic reasons
* rendering

The diagnostic tree is public so applications can consume it directly.

### `lib.rs`

Provides the public parsing and evaluation API.

### `main.rs`

Provides the CLI. It handles argument parsing and output while keeping evaluation logic in the library.

## Status Model

#### `SATISFIED`

The fragment can currently be satisfied with the supplied context.

#### `UNAVAILABLE`

The fragment is not currently satisfiable, but its required condition may become available later. Examples include an unavailable preimage or an immature timelock.

#### `IMPOSSIBLE`

The fragment cannot be satisfied. A missing signature required by `pk` is treated as impossible because signatures cannot be forged, matching `rust-miniscript`'s satisfaction semantics.

#### `UNSUPPORTED`

The fragment is outside the supported scope of the evaluator. This is a project-specific status and does not correspond to a `rust-miniscript` witness state.

## Evaluation

The evaluator handles supported leaves and combinators recursively. Child diagnostics are preserved so the result explains both the root status and the state of each branch. Combinator diagnostics also include reasons describing why the resulting status was reached.

### AND

All required branches must be satisfiable.

### OR

At least one branch must be satisfiable. Unsuccessful branches remain in the diagnostic tree.

### AND/OR combined

`andor(A, B, C)` is satisfied if both `A` and `B` are satisfiable, or if `C` is satisfiable. It is evaluated as `combine_or(combine_and(A, B), C)` and its diagnostic node preserves all three children directly rather than synthesizing an intermediate AND node, so the tree matches what the user wrote.

### Threshold

`thresh(k, ...)` reports whether at least `k` children can currently be satisfied.

It reports feasibility only and does not select branches or reproduce witness-cost optimization performed by a planner.

## Miniscript Boundary

The evaluator uses the public `rust-miniscript` AST and does not depend on private Miniscript implementation details. The project evaluates raw Miniscript expressions using its own `DiagnosticContext` rather than wallet or descriptor-specific state.

## Parsing

Parsing and type checking are delegated to `rust-miniscript`. The CLI uses `from_str_insane` so that well-formed Miniscript can still be diagnosed when it fails additional sanity checks.

## Scope

The current version does not implement:

* Taproot
* witness generation
* signing
* transaction construction
* PSBT handling
* witness optimization
* signature verification
