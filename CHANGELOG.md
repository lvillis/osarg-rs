## 0.1.0 - 2026-04-18

- Added the initial zero-dependency `osarg` parser crate.
- Implemented borrow-first parsing for short options, grouped shorts, long options,
  attached values, separated values, `--`, and positional arguments.
- Added structured errors, UTF-8 conversion helpers, typed `FromStr` parsing, and
  minimal `help` / `version` usage examples.
- Added repository CI and a reproducible size benchmark against handwritten and
  `clap` reference binaries.
