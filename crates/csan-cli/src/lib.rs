#![forbid(unsafe_code)]

//! Argument parsing and execution for the `csan` command-line tool.
//!
//! The binary itself ([`main.rs`](../src/main.rs)) is a thin shell around
//! [`Cli`] and [`run`], so the whole surface is unit-testable without
//! spawning processes.

use clap::error::ErrorKind;
use clap::{ArgGroup, Parser};
use csan::{
    Attribution, CutterConfig, CutterGenerator, CutterLocMethod, CutterSanbornMethod, detailed_json,
};
use std::ffi::OsString;

/// Generate Cutter call numbers for library cataloging.
///
/// Provide at least one method flag (`--cutter-sanborn`, `--cutter-loc`)
/// and at least one non-blank input (`--forename`, `--surname`,
/// `--title`). With both method flags, the Cutter-Sanborn number is
/// printed first, each on its own line.
#[derive(Debug, Parser)]
#[command(name = "csan", version, about, group(
    ArgGroup::new("method")
        .required(true)
        .multiple(true)
        .args(["cutter_sanborn", "cutter_loc"])
))]
pub struct Cli {
    /// Generate a Cutter-Sanborn three-figure number
    #[arg(long)]
    pub cutter_sanborn: bool,

    /// Generate a Library of Congress cutter number
    #[arg(long)]
    pub cutter_loc: bool,

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
/// Returns a human-readable message when no method was selected (only
/// possible for a hand-constructed [`Cli`]; `clap` enforces this at parse
/// time), when no usable input was provided, or when Cutter generation
/// fails (e.g. a name that normalizes to nothing).
pub fn run(cli: &Cli) -> Result<String, String> {
    if !cli.cutter_sanborn && !cli.cutter_loc {
        return Err(String::from(
            "select at least one method: --cutter-sanborn or --cutter-loc",
        ));
    }

    let attribution = Attribution::from_parts(
        cli.forename.as_deref(),
        cli.surname.as_deref(),
        cli.title.as_deref(),
    )
    .ok_or_else(|| {
        String::from(
            "provide at least one non-blank input: --forename, --surname, or --title",
        )
    })?;

    let config = CutterConfig {
        prefix_len: cli.prefix_len,
        suffix_len: cli.suffix_len,
        max_len: cli.max_len,
    };

    let sanborn = cli
        .cutter_sanborn
        .then(|| CutterSanbornMethod::generate(&attribution, config))
        .transpose()
        .map_err(|e| format!("cutter-sanborn generation failed: {e}"))?;
    let loc = cli
        .cutter_loc
        .then(|| CutterLocMethod::generate(&attribution, config))
        .transpose()
        .map_err(|e| format!("cutter-loc generation failed: {e}"))?;

    if cli.json {
        let mut root = serde_json::Map::new();
        if let Some(cutter) = &sanborn {
            root.insert(
                "cutter_sanborn".to_owned(),
                detailed_json(cutter).map_err(|e| e.to_string())?,
            );
        }
        if let Some(cutter) = &loc {
            root.insert(
                "cutter_loc".to_owned(),
                detailed_json(cutter).map_err(|e| e.to_string())?,
            );
        }
        serde_json::to_string_pretty(&serde_json::Value::Object(root))
            .map_err(|e| format!("failed to serialize JSON output: {e}"))
    } else {
        let lines: Vec<String> = sanborn
            .iter()
            .map(ToString::to_string)
            .chain(loc.iter().map(ToString::to_string))
            .collect();
        Ok(lines.join("\n"))
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
            run_from_args(["csan", "--cutter-loc", "--surname", "Carter"]),
            0
        );
        assert_eq!(run_from_args(["csan", "--help"]), 0);
        assert_eq!(run_from_args(["csan", "--version"]), 0);
        assert_eq!(run_from_args(["csan", "--no-such-flag"]), 2);
        assert_eq!(run_from_args(["csan"]), 2, "missing method group");
        assert_eq!(
            run_from_args(["csan", "--cutter-loc", "--surname", "  "]),
            1,
            "blank input is a runtime failure"
        );
    }

    #[test]
    fn clap_definition_is_internally_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn run_rejects_a_methodless_cli() {
        // `clap` enforces the method group at parse time; `run` guards the
        // programmatic path too.
        let cli = Cli {
            cutter_sanborn: false,
            cutter_loc: false,
            forename: None,
            surname: Some("Dickens".to_owned()),
            title: None,
            prefix_len: 1,
            suffix_len: 1,
            max_len: 3,
            json: false,
        };
        assert!(run(&cli).unwrap_err().contains("--cutter-sanborn"));
    }

    #[test]
    fn a_method_flag_is_required() {
        let err = parse(&["csan", "--surname", "Dickens"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn single_method_prints_bare_value() {
        let cli = parse(&[
            "csan",
            "--cutter-sanborn",
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
    fn both_methods_print_sanborn_then_loc() {
        let cli = parse(&[
            "csan",
            "--cutter-sanborn",
            "--cutter-loc",
            "--surname",
            "Carter",
        ])
        .unwrap();
        assert_eq!(run(&cli).unwrap(), "C323\nC378");
    }

    #[test]
    fn config_flags_are_honored() {
        let cli = parse(&[
            "csan",
            "--cutter-sanborn",
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
            "--cutter-loc",
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
        let cli = parse(&["csan", "--cutter-loc", "--title", "100 best recipes"]).unwrap();
        assert_eq!(run(&cli).unwrap(), "A153");
    }

    #[test]
    fn blank_inputs_error_with_guidance() {
        let cli = parse(&["csan", "--cutter-loc", "--surname", "   "]).unwrap();
        let err = run(&cli).unwrap_err();
        assert!(err.contains("--surname"), "unhelpful message: {err}");
    }

    #[test]
    fn json_output_carries_parts_and_formatted() {
        let cli = parse(&[
            "csan",
            "--cutter-sanborn",
            "--cutter-loc",
            "--json",
            "--forename",
            "Charles",
            "--surname",
            "Dickens",
            "--title",
            "Oliver Twist",
        ])
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&run(&cli).unwrap()).unwrap();
        assert_eq!(value["cutter_sanborn"]["formatted"], "D548o");
        assert_eq!(value["cutter_sanborn"]["numeric"], 548);
        assert_eq!(value["cutter_sanborn"]["alpha_suffix"], "o");
        assert_eq!(value["cutter_loc"]["formatted"], "D535");
        assert_eq!(value["cutter_loc"]["alpha_prefix"], "D");
    }
}
