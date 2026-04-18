use osarg::{Arg, Parser};
use std::env;
use std::ffi::{OsStr, OsString};
use std::hint::black_box;
use std::time::{Duration, Instant};

const DEFAULT_ITERATIONS: u64 = 200_000;
const DEFAULT_WARMUP: u64 = 20_000;
const QUICK_ITERATIONS: u64 = 20_000;
const QUICK_WARMUP: u64 = 2_000;

struct BenchConfig {
    iterations: u64,
    warmup: u64,
    filter: Option<String>,
    list: bool,
}

struct Case {
    name: &'static str,
    args: Vec<OsString>,
    run: fn(&[OsString]) -> usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BenchConfig::from_env()?;
    let cases = cases();

    if config.list {
        for case in &cases {
            println!("{}", case.name);
        }
        return Ok(());
    }

    println!("runtime bench profile");
    println!("  rustc:      {}", rustc_version());
    println!("  iterations: {}", config.iterations);
    println!("  warmup:     {}", config.warmup);
    println!();
    println!(
        "{:<22} {:>14} {:>14} {:>14}",
        "benchmark", "ns/iter", "total ms", "checksum"
    );

    let mut matched = 0usize;

    for case in &cases {
        if config
            .filter
            .as_deref()
            .is_some_and(|filter| !case.name.contains(filter))
        {
            continue;
        }

        matched += 1;
        warm_up(case, config.warmup);
        let (elapsed, checksum) = run_case(case, config.iterations);
        let ns_per_iter = elapsed.as_secs_f64() * 1_000_000_000.0 / config.iterations as f64;
        let total_ms = elapsed.as_secs_f64() * 1_000.0;

        println!(
            "{:<22} {:>14.1} {:>14.3} {:>14}",
            case.name, ns_per_iter, total_ms, checksum
        );
    }

    if matched == 0 {
        return Err("no benchmark matched the requested filter".into());
    }

    Ok(())
}

impl BenchConfig {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self {
            iterations: DEFAULT_ITERATIONS,
            warmup: DEFAULT_WARMUP,
            filter: None,
            list: false,
        };

        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bench" => {}
                "--quick" => {
                    config.iterations = QUICK_ITERATIONS;
                    config.warmup = QUICK_WARMUP;
                }
                "--list" => {
                    config.list = true;
                }
                "--iterations" => {
                    config.iterations = parse_u64_arg(args.next(), "--iterations")?;
                }
                "--warmup" => {
                    config.warmup = parse_u64_arg(args.next(), "--warmup")?;
                }
                "--filter" => {
                    config.filter = Some(parse_string_arg(args.next(), "--filter")?);
                }
                _ => {
                    if let Some(value) = arg.strip_prefix("--iterations=") {
                        config.iterations = value.parse()?;
                    } else if let Some(value) = arg.strip_prefix("--warmup=") {
                        config.warmup = value.parse()?;
                    } else if let Some(value) = arg.strip_prefix("--filter=") {
                        config.filter = Some(value.to_owned());
                    } else {
                        return Err(format!("unknown benchmark argument: {arg}").into());
                    }
                }
            }
        }

        if config.iterations == 0 {
            return Err("--iterations must be greater than zero".into());
        }

        Ok(config)
    }
}

fn parse_u64_arg(value: Option<String>, flag: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let value = parse_string_arg(value, flag)?;
    Ok(value.parse()?)
}

fn parse_string_arg(
    value: Option<String>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    value.ok_or_else(|| format!("missing value for {flag}").into())
}

fn warm_up(case: &Case, iterations: u64) {
    if iterations == 0 {
        return;
    }

    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box((case.run)(black_box(&case.args))));
    }
    black_box(checksum);
}

fn run_case(case: &Case, iterations: u64) -> (Duration, usize) {
    let start = Instant::now();
    let mut checksum = 0usize;

    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box((case.run)(black_box(&case.args))));
    }

    (start.elapsed(), checksum)
}

fn cases() -> [Case; 5] {
    [
        Case {
            name: "short-cluster",
            args: fixture(["-vv", "-p8080", "./app"]),
            run: parse_short_cluster,
        },
        Case {
            name: "long-values",
            args: fixture(["--bind=0.0.0.0", "--port", "8080", "--color=auto", "app"]),
            run: parse_long_values,
        },
        Case {
            name: "optional-none",
            args: fixture(["--color", "--verbose", "./file.txt"]),
            run: parse_optional_none,
        },
        Case {
            name: "repeated-options",
            args: fixture([
                "-vv",
                "-Iinclude",
                "-I",
                "generated",
                "-DMODE=release",
                "src/main.c",
            ]),
            run: parse_repeated_options,
        },
        Case {
            name: "passthrough",
            args: fixture([
                "--env",
                "RUST_LOG=debug",
                "cargo",
                "test",
                "--",
                "--nocapture",
            ]),
            run: parse_passthrough,
        },
    ]
}

fn fixture<const N: usize>(args: [&str; N]) -> Vec<OsString> {
    args.into_iter().map(OsString::from).collect()
}

fn parse_short_cluster(args: &[OsString]) -> usize {
    let mut parser = Parser::new(args.iter().cloned());
    let mut verbose = 0usize;
    let mut port = 0usize;
    let mut path_len = 0usize;

    while let Some(arg) = parser.next().expect("parse succeeds") {
        match arg {
            Arg::Short('v') => verbose += 1,
            Arg::Short('p') | Arg::Long("port") => {
                port = parser.parse::<u16>().expect("port parses") as usize;
            }
            Arg::Value(value) => {
                path_len = encoded_len(value.as_os_str());
            }
            other => panic!("unexpected benchmark argument: {other}"),
        }
    }

    verbose + port + path_len
}

fn parse_long_values(args: &[OsString]) -> usize {
    let mut parser = Parser::new(args.iter().cloned());
    let mut bind_len = 0usize;
    let mut port = 0usize;
    let mut color_len = 0usize;
    let mut path_len = 0usize;

    while let Some(arg) = parser.next().expect("parse succeeds") {
        match arg {
            Arg::Long("bind") => {
                bind_len = encoded_len(parser.value().expect("bind value").as_os_str());
            }
            Arg::Long("port") => {
                port = parser.parse::<u16>().expect("port parses") as usize;
            }
            Arg::Long("color") => {
                color_len = encoded_len(parser.value().expect("color value").as_os_str());
            }
            Arg::Value(value) => {
                path_len = encoded_len(value.as_os_str());
            }
            other => panic!("unexpected benchmark argument: {other}"),
        }
    }

    bind_len + port + color_len + path_len
}

fn parse_optional_none(args: &[OsString]) -> usize {
    let mut parser = Parser::new(args.iter().cloned());
    let mut used_default = false;
    let mut verbose = 0usize;
    let mut path_len = 0usize;

    while let Some(arg) = parser.next().expect("parse succeeds") {
        match arg {
            Arg::Long("color") => {
                used_default = parser.value_opt().expect("optional value parses").is_none();
            }
            Arg::Long("verbose") => {
                verbose += 1;
            }
            Arg::Value(value) => {
                path_len = encoded_len(value.as_os_str());
            }
            other => panic!("unexpected benchmark argument: {other}"),
        }
    }

    usize::from(used_default) + verbose + path_len
}

fn parse_repeated_options(args: &[OsString]) -> usize {
    let mut parser = Parser::new(args.iter().cloned());
    let mut verbose = 0usize;
    let mut include_bytes = 0usize;
    let mut define_bytes = 0usize;
    let mut input_len = 0usize;

    while let Some(arg) = parser.next().expect("parse succeeds") {
        match arg {
            Arg::Short('v') => verbose += 1,
            Arg::Short('I') | Arg::Long("include") => {
                include_bytes += encoded_len(parser.value().expect("include value").as_os_str());
            }
            Arg::Short('D') | Arg::Long("define") => {
                let value = parser.value().expect("define value");
                let text = value.to_str().expect("define utf-8");
                let (key, raw_value) = text.split_once('=').expect("KEY=VALUE");
                define_bytes += key.len() + raw_value.len();
            }
            Arg::Value(value) => {
                input_len = encoded_len(value.as_os_str());
            }
            other => panic!("unexpected benchmark argument: {other}"),
        }
    }

    verbose + include_bytes + define_bytes + input_len
}

fn parse_passthrough(args: &[OsString]) -> usize {
    let mut parser = Parser::new(args.iter().cloned());
    let mut env_bytes = 0usize;
    let mut command_len = 0usize;
    let mut forwarded = 0usize;

    while let Some(arg) = parser.next().expect("parse succeeds") {
        match arg {
            Arg::Long("env") => {
                env_bytes += encoded_len(parser.value().expect("env value").as_os_str());
            }
            Arg::Value(value) => {
                command_len = encoded_len(value.as_os_str());
                forwarded = parser
                    .remaining_vec()
                    .into_iter()
                    .map(|value| encoded_len(&value))
                    .sum();
                break;
            }
            other => panic!("unexpected benchmark argument: {other}"),
        }
    }

    env_bytes + command_len + forwarded
}

fn encoded_len(value: &OsStr) -> usize {
    value.as_encoded_bytes().len()
}

fn rustc_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map_or_else(
            || String::from("unknown"),
            |output| String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        )
}
