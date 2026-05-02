## Why

Currently the codebase has no local formatting enforcement. While CI runs `cargo fmt --check`, bad formatting can still be committed locally and only caught at PR time, creating unnecessary back-and-forth. A local pre-commit hook together with CI enforcement ensures code is always properly formatted before it reaches review.

## What Changes

- Add `.githooks/pre-commit` hook that runs `cargo fmt --all` and `taplo fmt` before every commit
- Configure `core.hooksPath` to point at `.githooks/` so hooks are version-controlled and shared
- Add `taplo` to CI workflow alongside existing `cargo fmt` check
- Add `.githooks/` directory tracking to git (version-controlled hooks)

## Capabilities

### New Capabilities
- `code-formatting-enforcement`: Pre-commit hooks and CI checks that enforce code formatting standards for Rust and TOML files

### Modified Capabilities
<!-- No existing capabilities are affected -- formatting is an infrastructure concern, not a feature change -->

## Impact

- **New file**: `.githooks/pre-commit` — version-controlled git hook
- **Modified file**: `.github/workflows/ci.yml` — add `taplo fmt --check`
- **New dependency**: `taplo-cli` for TOML formatting
- **Setup required**: Developers need to run `git config core.hooksPath .githooks` once after clone
