#![forbid(unsafe_code)]

//! Argument parsing and execution for the `csan` command-line tool.
//!
//! The binary itself ([`main.rs`](../src/main.rs)) is a thin shell around
//! [`Cli`] and [`run`], so the whole surface is unit-testable without
//! spawning processes.

use clap::error::ErrorKind;
use clap::{Parser, ValueEnum};
use csan_core::{
    Attribution, CutterConfig, CutterGenerator, CutterLocMethod, CutterSanbornMethod, detailed_json,
};
use std::ffi::OsString;

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

/// Generate Cutter call numbers for library cataloging.
///
/// Provide at least one non-blank input (`--forename`, `--surname`,
/// `--title`). `--method` selects the method (Cutter-Sanborn by default).
#[derive(Debug, Parser)]
#[command(name = "csan", version, about)]
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
    #[arg(long, value_name = "N", default_value_t = CutterConfig::default().prefix_len)]
    pub prefix_len: usize,

    /// Letters taken from the title for the Cutter-Sanborn work mark
    #[arg(long, value_name = "N", default_value_t = CutterConfig::default().suffix_len)]
    pub suffix_len: usize,

    /// Digits in the Library of Congress number (clamped to 4)
    #[arg(long, value_name = "N", default_value_t = CutterConfig::default().max_len)]
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

/// Parses `argv` (including the program name), executes it, prints the
/// result or error, and returns the process exit code.
///
/// This is the single exit-code policy point shared by the standalone
/// binary and the Python console script: `0` on success (including
/// `--help`/`--version`), `1` for runtime failures, and `2` for argument
/// errors, with results on stdout and diagnostics on stderr.
pub fn run_from_args<I, T>(argv: I) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(argv) {
        Ok(cli) => match run(&cli) {
            Ok(output) => {
                println!("{output}");
                0
            }
            Err(message) => {
                eprintln!("error: {message}");
                1
            }
        },
        Err(parse_error) => {
            // --help/--version land here too; clap renders them and the
            // conventional exit codes apply (0 for the displays, 2 for
            // usage errors).
            let displays_only = matches!(
                parse_error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            );
            // Nothing useful can be done if rendering fails (e.g. the
            // pipe is already closed, as with `csan --help | head`).
            parse_error.print().unwrap_or_default();
            if displays_only { 0 } else { 2 }
        }
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
