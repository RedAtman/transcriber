## 1. Setup

- [ ] 1.1 Install `taplo-cli` (cargo install taplo-cli)
- [ ] 1.2 Create `.githooks/` directory at project root
- [ ] 1.3 Create `.githooks/pre-commit` script with:
  - Check for `taplo` availability
  - Run `cargo fmt --all` (auto-format Rust)
  - Run `taplo fmt` (auto-format TOML, skip if not installed)
  - Run `git add -u` to re-stage formatted files

## 2. Git Configuration

- [ ] 2.1 Track `.githooks/` in git (`git add .githooks/`)
- [ ] 2.2 Run `git config core.hooksPath .githooks` to activate hooks locally

## 3. CI Update

- [ ] 3.1 Add `taplo` installation step to `rustfmt-clippy` job in `.github/workflows/ci.yml`
- [ ] 3.2 Add `taplo fmt --check` step to the formatting job in CI

## 4. Verification

- [ ] 4.1 Test pre-commit hook: stage unformatted `.rs` file, verify `git commit` auto-formats and succeeds
- [ ] 4.2 Test pre-commit hook: stage unformatted `Cargo.toml`, verify `taplo fmt` formats it
- [ ] 4.3 Test pre-commit hook: `taplo` not installed → hook prints warning and proceeds
- [ ] 4.4 Run `cargo build --release` to ensure no formatting breakage
- [ ] 4.5 Run `cargo test` to ensure tests still pass
