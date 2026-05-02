## Context

The project is a Rust CLI tool (transcriber) using `cargo fmt` for Rust formatting. Currently:
- CI runs `cargo fmt --all` + `git diff` check in `ci.yml`
- No local pre-commit hook exists
- No TOML formatting is enforced
- Hooks are not version-controlled

## Goals / Non-Goals

**Goals:**
- Enforce Rust and TOML formatting before every local `git commit`
- Version-control hooks so they're shared across the team
- Add TOML formatting to CI pipeline
- Auto-format on commit (fix minor issues without developer action)

**Non-Goals:**
- Clippy enforcement in pre-commit (already in CI)
- Commit message validation
- Any non-formatting git hooks

## Decisions

### 1. `.githooks/` over `.git/hooks/`
- **Choice**: Place version-controlled hook scripts in `.githooks/` directory
- **Rationale**: `.git/hooks/` is not committed to the repository. `.githooks/` (or any custom dir via `core.hooksPath`) is tracked by git and shared across clones. This ensures all developers use the same hooks without manual setup per checkout.
- **Alternatives considered**: Symlink from `.git/hooks/` — not portable across OS; script in `scripts/` — less conventional path

### 2. Auto-format over format-check in pre-commit
- **Choice**: `cargo fmt --all && taplo fmt` in pre-commit (auto-fix), rather than `--check` (block only)
- **Rationale**: If formatting is slightly off, the developer doesn't need to manually re-run and re-stage. The hook fixes it in-place and stages the changes automatically.
- **Alternative**: `--check` + abort — more strict but worse developer experience

### 3. `taplo` for TOML formatting
- **Choice**: Use `taplo fmt` for `Cargo.toml` and other TOML files
- **Rationale**: Rust ecosystem standard for TOML formatting. No custom config needed — default rules work well for `Cargo.toml`.
- **Alternative**: Manual formatting — unenforceable and inconsistent

### 4. CI retains `--check` semantics
- **Choice**: CI keeps `cargo fmt --all` + `git diff` approach (not just `--check`)
- **Rationale**: Existing pattern works well. CI shows what the correctly formatted output should look like when it fails.
- **Alternative**: `taplo fmt --check` — added alongside existing cargo fmt step

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `taplo` not installed locally → hook fails | Hook checks for `taplo` presence and skips gracefully with a warning |
| Hook modifies staged files unexpectedly | Hook stages formatted files after `cargo fmt --all` (uses `git add -u`) |
| Developer bypasses hook via `--no-verify` | CI catches formatting issues at PR time — safe net |
