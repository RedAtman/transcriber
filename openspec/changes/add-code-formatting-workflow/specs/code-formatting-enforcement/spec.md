## ADDED Requirements

### Requirement: Rust formatting enforced pre-commit
The system SHALL run `cargo fmt --all` on all staged Rust source files before every `git commit`.

#### Scenario: Pre-commit hook formats Rust files
- **WHEN** developer runs `git commit` with unformatted `.rs` files staged
- **THEN** the hook runs `cargo fmt --all` and re-stages the formatted files
- **AND THEN** the commit proceeds with properly formatted files

#### Scenario: Already formatted files pass through
- **WHEN** developer runs `git commit` with already-formatted `.rs` files staged
- **THEN** the hook runs `cargo fmt --all` with no changes
- **AND THEN** the commit proceeds normally

### Requirement: TOML formatting enforced pre-commit
The system SHALL run `taplo fmt` on `Cargo.toml` and all TOML files before every `git commit`.

#### Scenario: Pre-commit hook formats TOML files
- **WHEN** developer runs `git commit` with unformatted `.toml` files staged
- **THEN** the hook runs `taplo fmt` and re-stages the formatted files
- **AND THEN** the commit proceeds with properly formatted files

#### Scenario: taplo not installed skips gracefully
- **WHEN** developer runs `git commit` and `taplo` is not installed
- **THEN** the hook SHALL print a warning and skip TOML formatting
- **AND THEN** the commit proceeds without error

### Requirement: Hook is version-controlled
The pre-commit hook SHALL live in a version-controlled directory (`.githooks/`) tracked by git.

#### Scenario: Hook location is configurable via core.hooksPath
- **WHEN** a developer clones the repository
- **AND WHEN** they run `git config core.hooksPath .githooks`
- **THEN** git SHALL use `.githooks/pre-commit` as the pre-commit hook

### Requirement: CI enforces formatting for both Rust and TOML
The CI pipeline SHALL reject pull requests with unformatted Rust or TOML files.

#### Scenario: CI fails on unformatted Rust code
- **WHEN** a PR contains `.rs` files not passing `cargo fmt --all`
- **THEN** the `rustfmt-clippy` job SHALL fail with a diff of formatting issues

#### Scenario: CI fails on unformatted TOML files
- **WHEN** a PR contains `.toml` files not passing `taplo fmt --check`
- **THEN** the `rustfmt-clippy` job SHALL fail with a formatting error message

### Requirement: Auto-fix stages formatted files
After formatting, the hook SHALL re-stage any modified files so the commit includes the formatted versions.

#### Scenario: Hook re-stages formatted files
- **WHEN** `cargo fmt --all` or `taplo fmt` modifies any file
- **THEN** the hook SHALL run `git add -u` to stage the modifications
- **AND THEN** the commit proceeds with the formatted versions
