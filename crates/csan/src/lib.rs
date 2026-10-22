#![forbid(unsafe_code)]

//! Argument parsing and execution for the `csan` command-line tool.
//!
//! The binary itself ([`main.rs`](../src/main.rs)) is a thin shell around
//! [`Cli`] and [`run`], so the whole surface is unit-testable without
//! spawning processes.

use clap::builder::RangedU64ValueParser;
use clap::error::ErrorKind;
use clap::{Parser, ValueEnum};
use csan_core::{
    Attribution, CutterConfig, CutterGenerator, CutterLocMethod, CutterSanbornMethod, detailed_json,
};
use std::ffi::OsString;
use std::io::{self, Write};

/// The Cutter method to generate a number with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum Method {
    /// Cutter-Sanborn three-figure number
    #[default]
    CutterSanborn,
    /// Library of Congress cutter number
    CutterLoc,
}

impl Method {
    /// The key of this method's object in `--json` output.
    const fn json_key(self) -> &'static str {
        match self {
            Self::CutterSanborn => "cutter_sanborn",
            Self::CutterLoc => "cutter_loc",
        }
    }
}

/// Parses a length option (`--prefix-len` and friends) as a `usize` no
/// larger than `u32::MAX`.
///
/// `usize` is 32 bits on wasm32 and 64 bits on the native targets; the
/// shared bound makes the native binary and the npm package's command
/// accept, and reject, exactly the same values.
fn length_parser() -> RangedU64ValueParser<usize> {
    RangedU64ValueParser::new().range(..=u64::from(u32::MAX))
}

/// Generate Cutter call numbers for library cataloging.
///
/// Provide at least one non-blank input (`--forename`, `--surname`,
/// `--title`). `--method` selects the method (Cutter-Sanborn by default).
#[derive(Debug, Parser)]
// `bin_name` keeps the usage line `csan` everywhere (not `csan.exe` on
// Windows), as the Python and npm commands print it.
#[command(name = "csan", bin_name = "csan", version, about)]
pub struct Cli {
    /// The method to generate the number with
    #[arg(long, value_enum, value_name = "METHOD", default_value_t = Method::default())]
    pub method: Method,

    /// The author's given (first) name
    #[arg(long, value_name = "NAME")]
    pub forename: Option<String>,

    /// The author's family (last) name
    #[arg(long, value_name = "NAME")]
    pub surname: Option<String>,

    /// The work's title (used for the work mark, or for title-derived
    /// cutters when no author is given)
    #[arg(long, value_name = "TITLE")]
    pub title: Option<String>,

    /// Letters taken from the name for the Cutter-Sanborn call number
    /// prefix
    #[arg(long, value_name = "N", default_value_t = CutterConfig::default().prefix_len, value_parser = length_parser())]
    pub prefix_len: usize,

    /// Letters taken from the title for the Cutter-Sanborn work mark
    #[arg(long, value_name = "N", default_value_t = CutterConfig::default().suffix_len, value_parser = length_parser())]
    pub suffix_len: usize,

    /// Digits in the Library of Congress number (clamped to 4)
    #[arg(long, value_name = "N", default_value_t = CutterConfig::default().max_len, value_parser = length_parser())]
    pub max_len: usize,

    /// Emit a JSON object with the number's parts instead of plain text
    #[arg(long)]
    pub json: bool,
}

/// Executes a parsed [`Cli`] invocation, returning the text to print.
///
/// # Errors
/// Returns a human-readable message when no usable input was provided, or
/// when Cutter generation fails (e.g. a name that normalizes to nothing).
pub fn run(cli: &Cli) -> Result<String, String> {
    let attribution = Attribution::from_parts(
        cli.forename.as_deref(),
        cli.surname.as_deref(),
        cli.title.as_deref(),
    )
    .ok_or_else(|| {
        String::from("provide at least one non-blank input: --forename, --surname, or --title")
    })?;

    let config = CutterConfig {
        prefix_len: cli.prefix_len,
        suffix_len: cli.suffix_len,
        max_len: cli.max_len,
    };

    let (formatted, detailed) = match cli.method {
        Method::CutterSanborn => {
            let cutter = CutterSanbornMethod::generate(&attribution, config)
                .map_err(|e| format!("cutter-sanborn generation failed: {e}"))?;
            (cutter.to_string(), detailed_json(&cutter))
        }
        Method::CutterLoc => {
            let cutter = CutterLocMethod::generate(&attribution, config)
                .map_err(|e| format!("cutter-loc generation failed: {e}"))?;
            (cutter.to_string(), detailed_json(&cutter))
        }
    };

    if cli.json {
        let mut root = serde_json::Map::new();
        root.insert(
            cli.method.json_key().to_owned(),
            detailed.map_err(|e| e.to_string())?,
        );
        serde_json::to_string_pretty(&serde_json::Value::Object(root))
            .map_err(|e| format!("failed to serialize JSON output: {e}"))
    } else {
        Ok(formatted)
    }
}

/// The result of a CLI invocation: what to print and the exit code.
///
/// Returned by [`execute`] for hosts that capture the output instead of
/// writing to the process streams (the WebAssembly build has none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The process exit code (see [`run_from_args`]).
    pub code: u8,
    /// Text for standard output, newline-terminated when non-empty.
    pub stdout: String,
    /// Text for standard error, newline-terminated when non-empty.
    pub stderr: String,
}

/// Which output streams [`execute`] styles with ANSI escapes.
///
/// The native binary styles clap's help and usage errors on a stream that
/// is a color-capable terminal; a host that prints an [`Outcome`] decides
/// that for its own streams and passes the answer here.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Styling {
    /// Style text destined for standard output.
    pub stdout: bool,
    /// Style text destined for standard error.
    pub stderr: bool,
}

/// The outcome of running a successfully parsed [`Cli`].
fn outcome_of(cli: &Cli) -> Outcome {
    match run(cli) {
        Ok(output) => Outcome {
            code: 0,
            stdout: format!("{output}\n"),
            stderr: String::new(),
        },
        Err(message) => Outcome {
            code: 1,
            stdout: String::new(),
            stderr: format!("error: {message}\n"),
        },
    }
}

/// The exit code for a parse "error": `--help`/`--version` land here too;
/// those displays exit 0 and usage errors exit 2.
fn parse_error_code(parse_error: &clap::Error) -> u8 {
    let displays_only = matches!(
        parse_error.kind(),
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
    );
    if displays_only { 0 } else { 2 }
}

/// Parses `argv` (including the program name) and executes it without
/// touching the process streams, returning what to print and the exit code.
///
/// This is [`run_from_args`] for hosts that print the [`Outcome`]
/// themselves (the npm package's `csan` command); both share the same
/// parsing, messages, and exit codes. `styling` selects, per stream,
/// whether clap's help and usage errors carry ANSI styles, as the native
/// binary decides for a terminal.
pub fn execute<I, T>(argv: I, styling: Styling) -> Outcome
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(argv) {
        Ok(cli) => outcome_of(&cli),
        Err(parse_error) => {
            let code = parse_error_code(&parse_error);
            let to_stderr = parse_error.use_stderr();
            let styled = if to_stderr {
                styling.stderr
            } else {
                styling.stdout
            };
            let rendered = parse_error.render();
            let text = if styled {
                rendered.ansi().to_string()
            } else {
                rendered.to_string()
            };
            if to_stderr {
                Outcome {
                    code,
                    stdout: String::new(),
                    stderr: text,
                }
            } else {
                Outcome {
                    code,
                    stdout: text,
                    stderr: String::new(),
                }
            }
        }
    }
}

/// Applies the write-failure policy to the exit code of an invocation.
///
/// A closed pipe (`csan --help | head -1`) is not a failure: the reader
/// has what it wanted, so the exit code stands. Any other failure (e.g. a
/// full disk) is reported in one line and exits 1. The npm package's
/// `csan` command applies the same policy to its streams.
fn finish(code: u8, written: io::Result<()>) -> u8 {
    match written {
        Ok(()) => code,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => code,
        Err(error) => {
            // If stderr is unwritable too, there is nowhere left to report.
            writeln!(io::stderr(), "error: failed to write output: {error}").unwrap_or_default();
            1
        }
    }
}

/// Writes an [`Outcome`]'s text to the process streams.
fn write_outcome(outcome: &Outcome) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(outcome.stdout.as_bytes())?;
    stdout.flush()?;
    io::stderr().lock().write_all(outcome.stderr.as_bytes())
}

/// Parses `argv` (including the program name), executes it, prints the
/// result or error, and returns the process exit code.
///
/// This is the single exit-code policy point shared by the standalone
/// binary, the Python console script, and (through [`execute`]) the npm
/// package's `csan` command: `0` on success (including
/// `--help`/`--version`), `1` for runtime failures or output that cannot
/// be written, and `2` for argument errors, with results on stdout and
/// diagnostics on stderr.
pub fn run_from_args<I, T>(argv: I) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(argv) {
        Ok(cli) => {
            let outcome = outcome_of(&cli);
            finish(outcome.code, write_outcome(&outcome))
        }
        // clap prints help and version to stdout and errors to stderr,
        // styled when the stream is a color-capable terminal.
        Err(parse_error) => finish(parse_error_code(&parse_error), parse_error.print()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    /// Parses a full argv (including the program name).
    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(args)
    }

    #[test]
    fn run_from_args_maps_outcomes_to_exit_codes() {
        assert_eq!(
            run_from_args(["csan", "--method", "cutter-loc", "--surname", "Carter"]),
            0
        );
        assert_eq!(run_from_args(["csan", "--surname", "Carter"]), 0);
        assert_eq!(run_from_args(["csan", "--help"]), 0);
        assert_eq!(run_from_args(["csan", "--version"]), 0);
        assert_eq!(run_from_args(["csan", "--no-such-flag"]), 2);
        assert_eq!(
            run_from_args(["csan", "--method", "dewey", "--surname", "Carter"]),
            2,
            "unknown method is a usage error"
        );
        assert_eq!(run_from_args(["csan"]), 1, "no input is a runtime failure");
        assert_eq!(
            run_from_args(["csan", "--method", "cutter-loc", "--surname", "  "]),
            1,
            "blank input is a runtime failure"
        );
    }

    #[test]
    fn execute_captures_output_and_exit_codes() {
        let ok = execute(
            ["csan", "--surname", "Dickens", "--title", "Oliver Twist"],
            Styling::default(),
        );
        assert_eq!(
            ok,
            Outcome {
                code: 0,
                stdout: "D548o\n".to_owned(),
                stderr: String::new(),
            }
        );

        let runtime = execute(["csan", "--surname", "  "], Styling::default());
        assert_eq!(runtime.code, 1);
        assert!(runtime.stdout.is_empty());
        assert!(runtime.stderr.starts_with("error: provide at least one"));

        // Usage errors go to stderr, the --help/--version displays to
        // stdout, as clap prints them for the native binary.
        let usage = execute(["csan", "--method", "dewey"], Styling::default());
        assert_eq!(usage.code, 2);
        assert!(usage.stdout.is_empty());
        assert!(
            usage
                .stderr
                .contains("possible values: cutter-sanborn, cutter-loc")
        );

        let help = execute(["csan", "--help"], Styling::default());
        assert_eq!(help.code, 0);
        assert!(help.stdout.contains("--method <METHOD>"));
        assert!(help.stderr.is_empty());

        let version = execute(["csan", "--version"], Styling::default());
        assert_eq!(
            version.stdout,
            format!("csan {}\n", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn execute_text_is_unstyled() {
        for args in [&["csan", "--help"][..], &["csan", "--method", "dewey"][..]] {
            let outcome = execute(args, Styling::default());
            for text in [&outcome.stdout, &outcome.stderr] {
                assert!(!text.contains('\u{1b}'), "ANSI escape in {args:?}");
            }
        }
    }

    #[test]
    fn execute_styles_only_the_requested_stream() {
        let both = Styling {
            stdout: true,
            stderr: true,
        };
        let usage = execute(["csan", "--method", "dewey"], both);
        assert!(usage.stderr.contains('\u{1b}'), "usage error not styled");
        let help = execute(["csan", "--help"], both);
        assert!(help.stdout.contains('\u{1b}'), "help not styled");

        // Only the stream the text goes to matters.
        let stdout_only = Styling {
            stdout: true,
            stderr: false,
        };
        let unstyled_usage = execute(["csan", "--method", "dewey"], stdout_only);
        assert!(!unstyled_usage.stderr.contains('\u{1b}'));
        // Results and runtime errors are never styled (as when printed).
        let result = execute(["csan", "--surname", "Carter"], both);
        assert_eq!(result.stdout, "C323\n");
        let runtime = execute(["csan"], both);
        assert!(!runtime.stderr.contains('\u{1b}'));
    }

    #[test]
    fn length_options_share_one_range_on_every_target() {
        let largest = parse(&["csan", "--prefix-len", "4294967295"]).unwrap();
        assert_eq!(largest.prefix_len, usize::try_from(u32::MAX).unwrap());
        for flag in ["--prefix-len", "--suffix-len", "--max-len"] {
            let err = parse(&["csan", flag, "4294967296"]).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::ValueValidation, "{flag}");
        }
    }

    #[test]
    fn write_failures_follow_the_policy() {
        let broken = io::Error::from(io::ErrorKind::BrokenPipe);
        assert_eq!(finish(0, Err(broken)), 0, "a closed pipe keeps the code");
        let full = io::Error::from(io::ErrorKind::StorageFull);
        assert_eq!(finish(0, Err(full)), 1, "other failures exit 1");
        assert_eq!(finish(2, Ok(())), 2);
    }

    #[test]
    fn clap_definition_is_internally_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn method_defaults_to_cutter_sanborn() {
        let cli = parse(&["csan", "--surname", "Dickens"]).unwrap();
        assert_eq!(cli.method, Method::CutterSanborn);

        let explicit = parse(&["csan", "--method", "cutter-sanborn", "--surname", "Dickens"]);
        assert_eq!(run(&cli).unwrap(), run(&explicit.unwrap()).unwrap());
    }

    #[test]
    fn method_accepts_both_value_forms() {
        for args in [
            ["csan", "--method", "cutter-loc"],
            ["csan", "--method=cutter-loc", "--json"],
        ] {
            assert_eq!(parse(&args).unwrap().method, Method::CutterLoc);
        }
        assert_eq!(
            parse(&["csan", "--method=cutter-sanborn"]).unwrap().method,
            Method::CutterSanborn
        );
    }

    #[test]
    fn method_rejects_unknown_and_repeated_values() {
        let unknown = parse(&["csan", "--method", "loc"]).unwrap_err();
        assert_eq!(unknown.kind(), ErrorKind::InvalidValue);

        let repeated = parse(&[
            "csan",
            "--method",
            "cutter-sanborn",
            "--method",
            "cutter-loc",
        ])
        .unwrap_err();
        assert_eq!(repeated.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn the_former_method_flags_are_gone() {
        for flag in ["--cutter-sanborn", "--cutter-loc"] {
            let err = parse(&["csan", flag, "--surname", "Dickens"]).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::UnknownArgument, "{flag}");
        }
    }

    #[test]
    fn cutter_sanborn_prints_bare_value() {
        let cli = parse(&[
            "csan",
            "--method",
            "cutter-sanborn",
            "--forename",
            "Charles",
            "--surname",
            "Dickens",
            "--title",
            "Oliver Twist",
        ])
        .unwrap();
        assert_eq!(run(&cli).unwrap(), "D548o");
    }

    #[test]
    fn each_method_prints_its_own_number() {
        let sanborn = parse(&["csan", "--surname", "Carter"]).unwrap();
        assert_eq!(run(&sanborn).unwrap(), "C323");
        let loc = parse(&["csan", "--method", "cutter-loc", "--surname", "Carter"]).unwrap();
        assert_eq!(run(&loc).unwrap(), "C378");
    }

    #[test]
    fn config_flags_are_honored() {
        let cli = parse(&[
            "csan",
            "--forename",
            "Charles",
            "--surname",
            "Dickens",
            "--title",
            "Oliver Twist",
            "--prefix-len",
            "2",
            "--suffix-len",
            "3",
        ])
        .unwrap();
        assert_eq!(run(&cli).unwrap(), "Di548oli");

        let loc = parse(&[
            "csan",
            "--method",
            "cutter-loc",
            "--surname",
            "Macdonald",
            "--max-len",
            "4",
        ])
        .unwrap();
        assert_eq!(run(&loc).unwrap(), "M3336");
    }

    #[test]
    fn title_only_input_works() {
        let cli = parse(&[
            "csan",
            "--method",
            "cutter-loc",
            "--title",
            "100 best recipes",
        ])
        .unwrap();
        assert_eq!(run(&cli).unwrap(), "A153");
    }

    #[test]
    fn blank_inputs_error_with_guidance() {
        let cli = parse(&["csan", "--method", "cutter-loc", "--surname", "   "]).unwrap();
        let err = run(&cli).unwrap_err();
        assert!(err.contains("--surname"), "unhelpful message: {err}");
    }

    #[test]
    fn json_output_carries_parts_and_formatted() {
        let dickens = [
            "--forename",
            "Charles",
            "--surname",
            "Dickens",
            "--title",
            "Oliver Twist",
        ];

        let sanborn = parse(&[&["csan", "--json"][..], &dickens].concat()).unwrap();
        let sanborn_json: serde_json::Value =
            serde_json::from_str(&run(&sanborn).unwrap()).unwrap();
        assert_eq!(sanborn_json["cutter_sanborn"]["formatted"], "D548o");
        assert_eq!(sanborn_json["cutter_sanborn"]["numeric"], 548);
        assert_eq!(sanborn_json["cutter_sanborn"]["alpha_suffix"], "o");
        assert_eq!(
            sanborn_json.as_object().unwrap().len(),
            1,
            "only the chosen method"
        );

        let loc =
            parse(&[&["csan", "--json", "--method", "cutter-loc"][..], &dickens].concat()).unwrap();
        let loc_json: serde_json::Value = serde_json::from_str(&run(&loc).unwrap()).unwrap();
        assert_eq!(loc_json["cutter_loc"]["formatted"], "D535");
        assert_eq!(loc_json["cutter_loc"]["alpha_prefix"], "D");
        assert_eq!(
            loc_json.as_object().unwrap().len(),
            1,
            "only the chosen method"
        );
    }
}
