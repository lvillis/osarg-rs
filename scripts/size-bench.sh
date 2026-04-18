#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/osarg-size-bench-XXXXXX")
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT_DIR/target/size-bench}"
CLAP_VERSION="${CLAP_VERSION:-4.5}"
MAX_OSARG_DELTA_BYTES="${MAX_OSARG_DELTA_BYTES:-}"
MAX_OSARG_RATIO="${MAX_OSARG_RATIO:-}"
MIN_CLAP_DELTA_BYTES="${MIN_CLAP_DELTA_BYTES:-}"

cleanup() {
    rm -rf "$WORK_DIR"
}

trap cleanup EXIT

mkdir -p "$WORK_DIR/handwritten/src" "$WORK_DIR/osarg/src" "$WORK_DIR/clap/src"

cat > "$WORK_DIR/handwritten/Cargo.toml" <<'EOF'
[package]
name = "handwritten-bench"
version = "0.1.0"
edition = "2024"

[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
panic = "abort"
strip = "symbols"
EOF

cat > "$WORK_DIR/handwritten/src/main.rs" <<'EOF'
use std::ffi::OsStr;

fn parse_u16(value: &OsStr) -> Result<u16, &'static str> {
    value
        .to_str()
        .ok_or("invalid utf-8")?
        .parse::<u16>()
        .map_err(|_| "invalid value")
}

fn main() -> Result<(), &'static str> {
    let mut help = false;
    let mut version = false;
    let mut port = None;
    let mut path = None;
    let mut positional_only = false;

    let mut args = std::env::args_os().skip(1).peekable();

    while let Some(arg) = args.next() {
        let bytes = arg.as_encoded_bytes();

        if !positional_only && bytes == b"--" {
            positional_only = true;
            continue;
        }

        if !positional_only && bytes.len() > 2 && bytes.starts_with(b"--") {
            let body = &bytes[2..];
            let eq_index = body.iter().position(|byte| *byte == b'=');
            let name_end = eq_index.map_or(bytes.len(), |index| index + 2);
            let name = std::str::from_utf8(&bytes[2..name_end]).map_err(|_| "invalid option")?;

            match name {
                "help" => help = true,
                "version" => version = true,
                "port" => {
                    let next_value;
                    let value = match eq_index {
                        Some(index) => {
                            let start = index + 3;
                            let slice = &bytes[start..];
                            unsafe { OsStr::from_encoded_bytes_unchecked(slice) }
                        }
                        None => {
                            next_value = args.next().ok_or("missing value")?;
                            next_value.as_os_str()
                        }
                    };

                    port = Some(parse_u16(value)?);
                }
                _ => return Err("unexpected argument"),
            }

            continue;
        }

        if !positional_only && bytes.len() > 1 && bytes[0] == b'-' && bytes[1] != b'-' {
            let mut index = 1;

            while index < bytes.len() {
                match bytes[index] {
                    b'h' => help = true,
                    b'V' => version = true,
                    b'p' => {
                        let next_value;
                        let value = if index + 1 < bytes.len() {
                            let slice = &bytes[(index + 1)..];
                            unsafe { OsStr::from_encoded_bytes_unchecked(slice) }
                        } else {
                            next_value = args.next().ok_or("missing value")?;
                            next_value.as_os_str()
                        };

                        port = Some(parse_u16(value)?);
                        break;
                    }
                    _ => return Err("unexpected argument"),
                }

                index += 1;
            }

            continue;
        }

        path = Some(arg);
    }

    let _ = (help, version, port, path);

    Ok(())
}
EOF

cat > "$WORK_DIR/osarg/Cargo.toml" <<EOF
[package]
name = "osarg-bench"
version = "0.1.0"
edition = "2024"

[dependencies]
osarg = { path = "$ROOT_DIR" }

[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
panic = "abort"
strip = "symbols"
EOF

cat > "$WORK_DIR/osarg/src/main.rs" <<'EOF'
use std::ffi::OsString;

use osarg::{Arg, Parser};

fn main() -> Result<(), osarg::Error> {
    let mut help = false;
    let mut version = false;
    let mut port = None;
    let mut path: Option<OsString> = None;
    let mut parser = Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Arg::Short('h') | Arg::Long("help") => help = true,
            Arg::Short('V') | Arg::Long("version") => version = true,
            Arg::Short('p') | Arg::Long("port") => {
                port = Some(parser.value()?.parse::<u16>()?);
            }
            Arg::Value(value) => path = Some(value.as_os_str().to_os_string()),
            other => return Err(other.unexpected()),
        }
    }

    let _ = (help, version, port, path);

    Ok(())
}
EOF

cat > "$WORK_DIR/clap/Cargo.toml" <<EOF
[package]
name = "clap-bench"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "$CLAP_VERSION", default-features = false, features = ["std"] }

[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
panic = "abort"
strip = "symbols"
EOF

cat > "$WORK_DIR/clap/src/main.rs" <<'EOF'
use std::ffi::OsString;

use clap::builder::OsStringValueParser;
use clap::{Arg, ArgAction, Command};

fn main() -> Result<(), clap::Error> {
    let matches = Command::new("clap-bench")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .num_args(1)
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("path")
                .index(1)
                .value_parser(OsStringValueParser::new()),
        )
        .try_get_matches_from(std::env::args_os())?;

    let help = matches.get_flag("help");
    let version = matches.get_flag("version");
    let port = matches.get_one::<u16>("port").copied();
    let path: Option<OsString> = matches.get_one::<OsString>("path").cloned();

    let _ = (help, version, port, path);

    Ok(())
}
EOF

build_crate() {
    local manifest_path="$1"
    cargo build --release --manifest-path "$manifest_path" --target-dir "$TARGET_DIR"
}

binary_size() {
    local path="$1"
    stat -c '%s' "$path"
}

report_line() {
    local name="$1"
    local size="$2"
    local baseline="$3"
    local delta=$((size - baseline))
    local ratio
    ratio=$(awk -v size="$size" -v baseline="$baseline" 'BEGIN { printf "%.2fx", size / baseline }')

    printf '%-12s %12s %12s %10s\n' "$name" "$size" "$delta" "$ratio"
}

assert_thresholds() {
    if [[ -n "$MAX_OSARG_DELTA_BYTES" ]]; then
        local delta=$((OSARG_SIZE - HANDWRITTEN_SIZE))
        if (( delta > MAX_OSARG_DELTA_BYTES )); then
            echo "osarg delta too large: ${delta} > ${MAX_OSARG_DELTA_BYTES}" >&2
            exit 1
        fi
    fi

    if [[ -n "$MAX_OSARG_RATIO" ]]; then
        if ! awk -v osarg="$OSARG_SIZE" -v handwritten="$HANDWRITTEN_SIZE" -v max_ratio="$MAX_OSARG_RATIO" 'BEGIN { exit !((osarg / handwritten) <= max_ratio) }'; then
            echo "osarg ratio too large: $(awk -v osarg="$OSARG_SIZE" -v handwritten="$HANDWRITTEN_SIZE" 'BEGIN { printf "%.4f", osarg / handwritten }') > ${MAX_OSARG_RATIO}" >&2
            exit 1
        fi
    fi

    if [[ -n "$MIN_CLAP_DELTA_BYTES" ]]; then
        local clap_delta=$((CLAP_SIZE - OSARG_SIZE))
        if (( clap_delta < MIN_CLAP_DELTA_BYTES )); then
            echo "clap advantage too small: ${clap_delta} < ${MIN_CLAP_DELTA_BYTES}" >&2
            exit 1
        fi
    fi
}

build_crate "$WORK_DIR/handwritten/Cargo.toml"
build_crate "$WORK_DIR/osarg/Cargo.toml"
build_crate "$WORK_DIR/clap/Cargo.toml"

HANDWRITTEN_BIN="$TARGET_DIR/release/handwritten-bench"
OSARG_BIN="$TARGET_DIR/release/osarg-bench"
CLAP_BIN="$TARGET_DIR/release/clap-bench"

HANDWRITTEN_SIZE=$(binary_size "$HANDWRITTEN_BIN")
OSARG_SIZE=$(binary_size "$OSARG_BIN")
CLAP_SIZE=$(binary_size "$CLAP_BIN")

printf 'Size benchmark profile\n'
printf '  rustc:  %s\n' "$(rustc -V)"
printf '  host:   %s\n' "$(rustc -vV | awk '/host:/ { print $2 }')"
printf '  target: %s\n' "${CARGO_BUILD_TARGET:-native}"
printf '  clap:   %s\n' "$CLAP_VERSION"
printf '  flags:  opt-level=z, lto=true, codegen-units=1, panic=abort, strip=symbols\n'
printf '\n'
printf '%-12s %12s %12s %10s\n' 'binary' 'bytes' 'delta' 'ratio'
report_line "handwritten" "$HANDWRITTEN_SIZE" "$HANDWRITTEN_SIZE"
report_line "osarg" "$OSARG_SIZE" "$HANDWRITTEN_SIZE"
report_line "clap" "$CLAP_SIZE" "$HANDWRITTEN_SIZE"

assert_thresholds
