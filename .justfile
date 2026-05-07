set shell := ["bash", "-euo", "pipefail", "-c"]

patch:
    cargo release patch --no-publish --execute

publish:
    cargo publish

ci:
  cargo fmt --all --check
  cargo check --all-features --locked
  cargo clippy --all-targets --all-features --locked -- -D warnings
  cargo nextest run --all-features --locked
  cargo test --doc --all-features --locked
  RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked
  cargo package --allow-dirty --locked

bench:
    cargo bench --bench parse --

bench-quick:
    cargo bench --bench parse -- --quick
