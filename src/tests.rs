use crate::{Arg, Error, ErrorKind, Parser, help, standard};
use std::ffi::OsString;

const _: Option<char> = Arg::Short('h').as_short();
const _: Option<&str> = Arg::Long("help").as_long();
const _: bool = Arg::Short('h').is_short('h');
const _: &str = standard::text(standard::Flag::Help, "HELP", "VERSION");

fn parser(args: &[&str]) -> Parser<std::vec::IntoIter<OsString>> {
    Parser::new(
        args.iter()
            .copied()
            .map(OsString::from)
            .collect::<Vec<_>>()
            .into_iter(),
    )
}

#[test]
fn parses_short_long_and_positional_arguments() {
    let mut parser = parser(&["-v", "--port", "8080", "app"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Short('v')));
    assert_eq!(parser.next().unwrap(), Some(Arg::Long("port")));
    assert_eq!(parser.value().unwrap().parse::<u16>().unwrap(), 8080);

    match parser.next().unwrap() {
        Some(Arg::Value(value)) => assert_eq!(value.to_str().unwrap(), "app"),
        other => panic!("unexpected argument: {other:?}"),
    }

    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn parses_grouped_short_options_one_by_one() {
    let mut parser = parser(&["-abc"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Short('a')));
    assert_eq!(parser.next().unwrap(), Some(Arg::Short('b')));
    assert_eq!(parser.next().unwrap(), Some(Arg::Short('c')));
    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn parses_long_option_with_attached_value() {
    let mut parser = parser(&["--port=8080"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Long("port")));
    assert_eq!(parser.value().unwrap().parse::<u16>().unwrap(), 8080);
    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn long_option_can_use_empty_attached_value() {
    let mut parser = parser(&["--color="]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Long("color")));
    assert_eq!(parser.value().unwrap().to_str().unwrap(), "");
    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn last_short_in_group_can_consume_following_value() {
    let mut parser = parser(&["-vp", "8080"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Short('v')));
    assert_eq!(parser.next().unwrap(), Some(Arg::Short('p')));
    assert_eq!(parser.value().unwrap().parse::<u16>().unwrap(), 8080);
    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn short_option_can_use_attached_tail_as_value() {
    let mut parser = parser(&["-p8080"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Short('p')));
    assert_eq!(parser.value().unwrap().parse::<u16>().unwrap(), 8080);
    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn last_short_in_cluster_can_use_attached_tail_as_value() {
    let mut parser = parser(&["-vp8080"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Short('v')));
    assert_eq!(parser.next().unwrap(), Some(Arg::Short('p')));
    assert_eq!(parser.value().unwrap().parse::<u16>().unwrap(), 8080);
    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn short_tail_value_is_explicit_and_does_not_consume_the_next_argument() {
    let mut parser = parser(&["-pv", "8080"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Short('p')));
    assert_eq!(parser.value().unwrap().to_str().unwrap(), "v");

    match parser.next().unwrap() {
        Some(Arg::Value(value)) => assert_eq!(value.to_str().unwrap(), "8080"),
        other => panic!("unexpected argument: {other:?}"),
    }
}

#[test]
fn sentinel_stops_option_parsing() {
    let mut parser = parser(&["--", "--help"]);

    match parser.next().unwrap() {
        Some(Arg::Value(value)) => assert_eq!(value.to_str().unwrap(), "--help"),
        other => panic!("unexpected argument: {other:?}"),
    }

    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn value_opt_uses_attached_value() {
    let mut parser = parser(&["--color=auto"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Long("color")));
    assert_eq!(
        parser.value_opt().unwrap().unwrap().to_str().unwrap(),
        "auto"
    );
}

#[test]
fn value_opt_uses_short_tail() {
    let mut parser = parser(&["-Cauto"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Short('C')));
    assert_eq!(
        parser.value_opt().unwrap().unwrap().to_str().unwrap(),
        "auto"
    );
    assert_eq!(parser.next().unwrap(), None);
}

#[test]
fn value_opt_leaves_following_option_unconsumed() {
    let mut parser = parser(&["--color", "--help"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Long("color")));
    assert!(parser.value_opt().unwrap().is_none());
    assert_eq!(parser.next().unwrap(), Some(Arg::Long("help")));
}

#[test]
fn value_parse_reports_invalid_value() {
    let mut parser = parser(&["--port", "nope"]);

    assert_eq!(parser.next().unwrap(), Some(Arg::Long("port")));
    let error = parser.value().unwrap().parse::<u16>().unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "nope");
}

#[test]
fn empty_long_option_name_is_rejected() {
    let mut parser = parser(&["--=value"]);
    let error = parser.next().unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidOptionName);
    assert_eq!(
        error.to_string(),
        "option name is invalid or not valid UTF-8: --=value"
    );
}

#[test]
fn unexpected_helpers_build_structured_errors() {
    assert_eq!(
        Arg::Short('v').unexpected().kind(),
        ErrorKind::UnexpectedArgument
    );

    let mut parser = parser(&["path"]);
    let arg = parser.next().unwrap().unwrap();
    assert_eq!(arg.unexpected().kind(), ErrorKind::UnexpectedPositional);
}

#[test]
fn arg_helpers_match_standard_flags() {
    let help = Arg::Short('h');
    assert_eq!(help.as_short(), Some('h'));
    assert_eq!(help.as_long(), None);
    assert!(help.is_short('h'));
    assert!(help.matches('h', "help"));
    assert!(standard::is_help(help));
    assert!(!standard::is_version(help));
    assert_eq!(standard::classify(help), Some(standard::Flag::Help));
    assert!(standard::Flag::Help.matches(help));
    assert_eq!(standard::Flag::Help.short(), 'h');
    assert_eq!(standard::Flag::Help.long(), "help");

    let version = Arg::Long("version");
    assert_eq!(version.as_short(), None);
    assert_eq!(version.as_long(), Some("version"));
    assert!(version.is_long("version"));
    assert!(version.matches('V', "version"));
    assert!(standard::is_version(version));
    assert!(!standard::is_help(version));
    assert_eq!(standard::classify(version), Some(standard::Flag::Version));
    assert!(standard::Flag::Version.matches(version));
}

#[test]
fn standard_helpers_return_and_write_text() {
    assert_eq!(
        standard::text(standard::Flag::Help, "HELP", "VERSION"),
        "HELP"
    );
    assert_eq!(
        standard::text(standard::Flag::Version, "HELP", "VERSION"),
        "VERSION"
    );

    let mut output = Vec::new();
    standard::write(&mut output, standard::Flag::Version, "HELP", "VERSION").unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), "VERSION");
}

#[test]
fn arg_and_value_display_are_human_readable() {
    assert_eq!(Arg::Short('p').to_string(), "-p");
    assert_eq!(Arg::Long("port").to_string(), "--port");

    let mut parser = parser(&["path"]);
    let arg = parser.next().unwrap().unwrap();
    assert_eq!(arg.to_string(), "path");
}

#[test]
fn help_module_writes_usage_and_sections() {
    let sections = &[
        help::Section::new(
            "options:",
            "  -h, --help       show help\n  -V, --version    show version",
        ),
        help::Section::new("notes:", "  caller-owned text"),
    ];
    let doc = help::Help::new("demo [OPTIONS] <PATH>", sections);

    let mut output = Vec::new();
    doc.write(&mut output).unwrap();

    assert_eq!(doc.usage(), "demo [OPTIONS] <PATH>");
    assert_eq!(doc.sections(), sections);
    assert_eq!(
        String::from_utf8(output).unwrap(),
        "usage: demo [OPTIONS] <PATH>\n\noptions:\n  -h, --help       show help\n  -V, --version    show version\n\nnotes:\n  caller-owned text\n"
    );
}

#[test]
fn help_module_can_write_individual_sections() {
    let section = help::Section::new("positional arguments:", "  PATH    target path");
    let mut output = Vec::new();

    help::write_section(&mut output, section).unwrap();

    assert_eq!(section.heading(), "positional arguments:");
    assert_eq!(section.body(), "  PATH    target path");
    assert_eq!(
        String::from_utf8(output).unwrap(),
        "positional arguments:\n  PATH    target path\n"
    );
}

#[test]
fn value_can_be_cloned_into_os_string() {
    let mut parser = parser(&["path"]);
    let arg = parser.next().unwrap().unwrap();

    match arg {
        Arg::Value(value) => assert_eq!(value.to_os_string(), OsString::from("path")),
        other => panic!("unexpected argument: {other:?}"),
    }
}

#[test]
fn arg_can_extract_value_without_matching() {
    let mut parser = parser(&["path"]);
    let arg = parser.next().unwrap().unwrap();

    assert!(arg.is_value());
    assert_eq!(arg.as_value().unwrap().to_str().unwrap(), "path");
}

#[test]
fn value_supports_display_and_try_from() {
    let mut parser = parser(&["path"]);
    let arg = parser.next().unwrap().unwrap();
    let value = arg.as_value().unwrap();

    assert_eq!(value.to_string(), "path");
    assert_eq!(<&str>::try_from(value).unwrap(), "path");
    assert_eq!(OsString::from(value), OsString::from("path"));
}

#[test]
fn value_can_build_invalid_value_error() {
    let mut parser = parser(&["MODE"]);
    let arg = parser.next().unwrap().unwrap();
    let value = arg.as_value().unwrap();
    let error = value.invalid();

    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(error.argument().unwrap().to_string_lossy(), "MODE");
}

#[test]
fn public_error_constructors_are_structured() {
    let missing_argument = Error::missing_argument_for("<PATH>".into());
    assert_eq!(missing_argument.kind(), ErrorKind::MissingArgument);
    assert_eq!(
        missing_argument.to_string(),
        "missing required argument: <PATH>"
    );

    let missing = Error::missing_value_for("--port".into());
    assert_eq!(missing.kind(), ErrorKind::MissingValue);
    assert_eq!(missing.argument().unwrap().to_string_lossy(), "--port");

    let positional = Error::unexpected_positional("path".into());
    assert_eq!(positional.kind(), ErrorKind::UnexpectedPositional);
    assert_eq!(
        positional.to_string(),
        "unexpected positional argument: path"
    );

    let invalid_utf8 = Error::invalid_utf8("value".into());
    assert_eq!(invalid_utf8.kind(), ErrorKind::InvalidUtf8);

    let unavailable = Error::value_unavailable_for("--flag".into());
    assert_eq!(unavailable.kind(), ErrorKind::ValueUnavailable);
}

#[test]
fn parser_can_parse_typed_values_directly() {
    let mut parser = parser(&["--port", "8080"]);
    assert_eq!(parser.next().unwrap(), Some(Arg::Long("port")));
    assert_eq!(parser.parse::<u16>().unwrap(), 8080);
}

#[test]
fn parser_can_read_utf8_and_owned_os_values_directly() {
    let mut bind_parser = parser(&["--bind", "0.0.0.0"]);
    assert_eq!(bind_parser.next().unwrap(), Some(Arg::Long("bind")));
    assert_eq!(bind_parser.string().unwrap(), "0.0.0.0");

    let mut path_parser = parser(&["--path", "./data"]);
    assert_eq!(path_parser.next().unwrap(), Some(Arg::Long("path")));
    assert_eq!(path_parser.os_string().unwrap(), OsString::from("./data"));
}

#[test]
fn parser_can_parse_optional_typed_values() {
    let mut color_parser = parser(&["--color", "7"]);
    assert_eq!(color_parser.next().unwrap(), Some(Arg::Long("color")));
    assert_eq!(color_parser.parse_opt::<u8>().unwrap(), Some(7));

    let mut missing_parser = parser(&["--color", "--help"]);
    assert_eq!(missing_parser.next().unwrap(), Some(Arg::Long("color")));
    assert_eq!(missing_parser.parse_opt::<u8>().unwrap(), None);
    assert_eq!(missing_parser.next().unwrap(), Some(Arg::Long("help")));
}

#[test]
fn parser_can_read_optional_utf8_and_owned_os_values_directly() {
    let mut string_parser = parser(&["--color", "auto"]);
    assert_eq!(string_parser.next().unwrap(), Some(Arg::Long("color")));
    assert_eq!(string_parser.string_opt().unwrap(), Some("auto"));

    let mut missing_string_parser = parser(&["--color", "--help"]);
    assert_eq!(
        missing_string_parser.next().unwrap(),
        Some(Arg::Long("color"))
    );
    assert_eq!(missing_string_parser.string_opt().unwrap(), None);
    assert_eq!(
        missing_string_parser.next().unwrap(),
        Some(Arg::Long("help"))
    );

    let mut os_parser = parser(&["--path", "./data"]);
    assert_eq!(os_parser.next().unwrap(), Some(Arg::Long("path")));
    assert_eq!(
        os_parser.os_string_opt().unwrap(),
        Some(OsString::from("./data"))
    );
}

#[test]
fn into_remaining_preserves_unread_arguments() {
    let mut parser = parser(&["--port", "8080", "cmd", "--tail"]);
    assert_eq!(parser.next().unwrap(), Some(Arg::Long("port")));

    let remaining = parser.into_remaining();
    assert_eq!(remaining.size_hint(), (3, Some(3)));
    assert_eq!(remaining.len(), 3);

    let remaining = remaining.collect::<Vec<_>>();
    assert_eq!(
        remaining,
        vec![
            OsString::from("8080"),
            OsString::from("cmd"),
            OsString::from("--tail")
        ]
    );
}

#[test]
fn into_remaining_reconstructs_grouped_short_tail() {
    let mut parser = parser(&["-abc", "tail"]);
    assert_eq!(parser.next().unwrap(), Some(Arg::Short('a')));

    let remaining = parser.into_remaining();
    assert_eq!(remaining.size_hint(), (2, Some(2)));
    assert_eq!(remaining.len(), 2);

    let remaining = remaining.collect::<Vec<_>>();
    assert_eq!(
        remaining,
        vec![OsString::from("-bc"), OsString::from("tail")]
    );
}

#[test]
fn into_remaining_keeps_optional_value_lookahead() {
    let mut parser = parser(&["--color", "--help", "tail"]);
    assert_eq!(parser.next().unwrap(), Some(Arg::Long("color")));
    assert_eq!(parser.parse_opt::<u8>().unwrap(), None);

    let remaining = parser.into_remaining();
    assert_eq!(remaining.size_hint(), (2, Some(2)));
    assert_eq!(remaining.len(), 2);

    let remaining = remaining.collect::<Vec<_>>();
    assert_eq!(
        remaining,
        vec![OsString::from("--help"), OsString::from("tail")]
    );
}

#[cfg(unix)]
#[test]
fn preserves_non_utf8_values() {
    use std::ffi::OsString;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    let value = OsString::from_vec(vec![0xff, b'a', b't', b'h']);
    let mut parser = Parser::new(vec![value].into_iter());

    match parser.next().unwrap() {
        Some(Arg::Value(raw)) => {
            assert_eq!(raw.as_os_str().as_bytes(), &[0xff, b'a', b't', b'h']);
            assert_eq!(raw.to_str().unwrap_err().kind(), ErrorKind::InvalidUtf8);
        }
        other => panic!("unexpected argument: {other:?}"),
    }
}

#[cfg(unix)]
#[test]
fn rejects_non_utf8_long_option_names() {
    use std::os::unix::ffi::OsStringExt;

    let mut parser = Parser::new(vec![OsString::from_vec(vec![b'-', b'-', 0xff])].into_iter());
    let error = parser.next().unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidOptionName);
}
