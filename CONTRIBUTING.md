# Contributing to miniscript-diagnostics

Thank you for your interest in contributing to `miniscript-diagnostics`.

## Getting Started

1. Fork the repository.
2. Clone your fork and create a branch for your change.
3. Make your changes and add or update tests where appropriate.
4. Run the project's checks locally.
5. Open a pull request describing your change.

## Reporting Issues

When reporting a bug, include:

- A description of the observed and expected behavior.
- A minimal Miniscript expression and spending context that reproduce the issue, where possible.
- The diagnostic result and relevant test output.

## Contributing Code

- Keep changes focused and consistent with the existing codebase.
- Add regression tests for bug fixes and tests for new functionality.
- Update documentation when changing public APIs or observable behavior.

## Development Setup

Install [Just](https://just.systems/) if it isn't already available.

Run the pre-push checks with:

```bash
just p
```

Individual commands are available for formatting (`just f`), checking (`just c`), testing (`just t`), and documentation (`just d`).

## Pull Requests

Please include a brief description of the change, its motivation, and the tests performed. Mention any known limitations where relevant.