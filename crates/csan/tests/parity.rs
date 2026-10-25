//! The captured interface ([`csan::execute`], behind the npm package's
//! `csan` command) must print exactly what the native binary prints.
//!
//! Each case runs the built `csan` binary with piped (non-terminal)
//! streams, so neither side styles its text, and compares stdout, stderr,
//! and the exit code byte for byte.

// `unused_crate_dependencies` is checked per target; these are the
// library's dependencies, not this test's.
use {clap as _, csan_core as _, serde_json as _};

#[cfg(test)]
mod tests {
    use csan::{Styling, execute};
    use std::io::Read;
    use std::iter;
    use std::process::{Command, Stdio};

    /// The native binary, with no color-forcing environment inherited.
    fn native() -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_csan"));
        command.env_remove("CLICOLOR_FORCE");
        command
    }

    #[test]
    #[cfg_attr(miri, ignore = "spawns processes")]
    fn execute_prints_exactly_what_the_binary_prints() {
        let cases: &[&[&str]] = &[
            &["--surname", "Dickens", "--title", "Oliver Twist"],
            &["--method", "cutter-loc", "--surname", "Carter"],
            &[
                "--method=cutter-loc",
                "--surname",
                "Macdonald",
                "--max-len",
                "4",
            ],
            &["--json", "--surname", "Borges"],
            &[
                "--json",
                "--method",
                "cutter-loc",
                "--title",
                "100 best recipes",
            ],
            &[
                "--prefix-len",
                "2",
                "--suffix-len",
                "3",
                "--surname",
                "Dickens",
                "--title",
                "Oliver Twist",
            ],
            &["--surname", "Čapek"],
            &[
                "--lang",
                "en",
                "--strip-article",
                "--surname",
                "Dickens",
                "--title",
                "A Christmas Carol",
            ],
            &["--strip-article", "--surname", "X"],
            &["--lang", "xx", "--surname", "X"],
            &["--help"],
            &["-h"],
            &["--version"],
            &["--method", "dewey"],
            &["--method"],
            &["--cutter-loc", "--surname", "X"],
            &["--surnam", "X"],
            &["--prefix-len", "4294967296", "--surname", "X"],
            &["--surname", "   "],
            &["--surname", "!!!"],
            &[],
        ];
        for args in cases {
            let output = native().args(*args).output().unwrap();
            let expected = execute(
                iter::once("csan").chain(args.iter().copied()),
                Styling::default(),
            );
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                expected.stdout,
                "stdout of {args:?}"
            );
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                expected.stderr,
                "stderr of {args:?}"
            );
            assert_eq!(
                output.status.code(),
                Some(i32::from(expected.code)),
                "exit code of {args:?}"
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "spawns processes")]
    fn a_closed_pipe_is_a_quiet_exit() {
        // The reader goes away before csan writes, as with `csan --help | true`.
        let mut child = native()
            .arg("--help")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        drop(child.stdout.take());
        let mut stderr = String::new();
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        assert_eq!(child.wait().unwrap().code(), Some(0));
        assert_eq!(stderr, "");
    }

    #[test]
    #[cfg(target_os = "linux")]
    #[cfg_attr(miri, ignore = "spawns processes")]
    fn an_unwritable_stdout_is_reported_and_exits_one() {
        use std::fs::OpenOptions;

        let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
        let output = native()
            .args(["--surname", "Carter"])
            .stdout(full)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.starts_with("error: failed to write output:"),
            "{stderr}"
        );
    }
}
