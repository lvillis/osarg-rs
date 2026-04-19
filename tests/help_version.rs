mod common;

use common::parser;
use osarg::{Arg, Error, help, standard};
use standard::Flag;

const HELP_SECTIONS: &[help::Section<'static>] = &[
    help::Section::new(
        "options:",
        "  -h, --help       show help\n  -V, --version    show version\n  -p, --port PORT  listen port",
    ),
    help::Section::new("examples:", "  demo -p8080 ./data"),
];
const HELP_DOC: help::Help<'static> = help::Help::new("demo [OPTIONS] <PATH>", HELP_SECTIONS);
const VERSION: &str = standard::text(Flag::Version, "unused", "1.2.3");

fn render_standard_output(args: &[&str]) -> Result<Option<String>, Error> {
    let mut parser = parser(args);

    while let Some(arg) = parser.next()? {
        let mut output = Vec::new();
        if standard::try_write(&mut output, arg, HELP_DOC, VERSION).expect("vec write succeeds") {
            return Ok(Some(
                String::from_utf8(output).expect("help/version is utf-8"),
            ));
        }

        match arg {
            Arg::Short('p') | Arg::Long("port") => {
                let _ = parser.parse::<u16>()?;
            }
            other => return Err(other.unexpected()),
        }
    }

    Ok(None)
}

#[test]
fn help_flag_can_drive_caller_owned_help_output() {
    let output = render_standard_output(&["--help"]).unwrap().unwrap();

    assert_eq!(
        output,
        "usage: demo [OPTIONS] <PATH>\n\noptions:\n  -h, --help       show help\n  -V, --version    show version\n  -p, --port PORT  listen port\n\nexamples:\n  demo -p8080 ./data\n"
    );
}

#[test]
fn version_flag_can_drive_caller_owned_version_output() {
    let output = render_standard_output(&["-V"]).unwrap().unwrap();
    assert_eq!(output, "1.2.3");
}
