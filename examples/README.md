# Examples

These examples are intentionally small and explicit. They are meant to be
copy-pasteable, and each one demonstrates a specific `osarg` pattern.

- `basic.rs`: a normal single-command CLI with caller-owned help/version text
- `optional_values.rs`: `parse_opt()` and optional-value flag handling
- `repeated_options.rs`: repeated flags, repeated values, and custom validation
- `passthrough.rs`: wrapper-style parsing with `current_value_and_remaining()`

Run them with:

```bash
cargo run --example basic -- --help
cargo run --example optional_values -- --color=never ./file.txt
cargo run --example repeated_options -- -vv -Iinclude -D MODE=release src/main.c
cargo run --example passthrough -- --dry-run echo hello world
```
