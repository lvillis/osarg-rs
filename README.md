# osarg

`osarg` is a tiny, zero-dependency, borrow-first CLI parser for Rust binaries.

It is built for size-sensitive tools that want explicit control flow, OS-native
argument handling, and enough ergonomics for real `--help`, `--version`,
options, values, and positionals.

`osarg` is not a CLI framework. It does not use derive macros, schemas, or
generated help text.

## Supported Forms

- `-v`
- `-abc`
- `--verbose`
- `--port=8080`
- `--port 8080`
- `-p 8080`
- `-p8080`
- `--`
- positional arguments
- repeated options
- passthrough via `Parser::into_remaining()` / `remaining_vec()`

## Example

```rust
use osarg::{Arg, Parser};

fn main() -> Result<(), osarg::Error> {
    let mut parser = Parser::new(
        ["-p", "8080", "./data"]
            .into_iter()
            .map(std::ffi::OsString::from),
    );

    let mut port = 8080;
    let mut path = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('p') | Arg::Long("port") => {
                port = parser.parse::<u16>()?;
            }
            Arg::Value(value) => {
                path = Some(value.to_os_string());
            }
            other => return Err(other.unexpected()),
        }
    }

    let path = path.ok_or_else(|| osarg::Error::unexpected_argument("<PATH>".into()))?;
    let _ = (port, path);
    Ok(())
}
```

## Examples

See `examples/` for:

- basic single-command parsing
- optional values
- repeated options
- passthrough wrappers

## Benchmarks

Size benchmark:

```bash
bash ./scripts/size-bench.sh
```

Runtime benchmark:

```bash
cargo bench --bench parse -- --quick
```
