"""Tests for the ``csan`` Python extension module.

Because the extension is built with pyo3's ``extension-module`` feature
(which omits libpython at link time), the binding functions cannot be
exercised by a Rust test harness. They are tested here instead, through the
same Python interface end users call, against the built module. Run with::

    maturin develop -m crates/csan-py/Cargo.toml
    pytest crates/csan-py/tests
"""

from __future__ import annotations

import pathlib
import subprocess
import sys

import pytest

import csan


# --------------------------------------------------------------------------
# cutter_sanborn
# --------------------------------------------------------------------------
def test_cutter_sanborn_with_author_and_title():
    assert (
        csan.cutter_sanborn(
            forename="Charles", surname="Dickens", title="Oliver Twist"
        )
        == "D548o"
    )


def test_cutter_sanborn_family_only():
    assert csan.cutter_sanborn(surname="Borges") == "B732"


def test_cutter_sanborn_given_only_matches_family_only():
    # A lone given name is cuttered like a lone family name.
    assert csan.cutter_sanborn(forename="Prince") == csan.cutter_sanborn(
        surname="Prince"
    )


def test_cutter_sanborn_honors_prefix_and_suffix_len():
    assert (
        csan.cutter_sanborn(
            surname="Dickens",
            title="Oliver Twist",
            prefix_len=2,
            suffix_len=3,
        )
        == "Di548oli"
    )


def test_cutter_sanborn_positional_arguments():
    # The signature is (forename, surname, title, prefix_len, suffix_len).
    assert csan.cutter_sanborn("Charles", "Dickens", "Oliver Twist") == "D548o"


# --------------------------------------------------------------------------
# cutter_sanborn_detailed
# --------------------------------------------------------------------------
def test_cutter_sanborn_detailed_shape():
    result = csan.cutter_sanborn_detailed(
        forename="Charles", surname="Dickens", title="Oliver Twist"
    )
    assert result == {
        "alpha_prefix": "D",
        "numeric": 548,
        "alpha_suffix": "o",
        "entry": "Dicke",  # the table key is the truncated form
        "formatted": "D548o",
    }
    # `numeric` must be an int, not a float (== would accept 548.0).
    assert type(result["numeric"]) is int


def test_cutter_sanborn_detailed_without_work_mark():
    result = csan.cutter_sanborn_detailed(surname="Borges")
    assert result["alpha_suffix"] is None
    assert result["entry"] == "Borg"
    assert result["numeric"] == 732
    assert result["formatted"] == "B732"


# --------------------------------------------------------------------------
# cutter_loc
# --------------------------------------------------------------------------
def test_cutter_loc_known_value():
    assert csan.cutter_loc(surname="Carter") == "C378"


def test_cutter_loc_title_numeral_rule():
    assert csan.cutter_loc(title="100 best recipes") == "A153"


def test_cutter_loc_max_len_is_clamped():
    # max_len above 4 is clamped rather than overflowing.
    assert csan.cutter_loc(surname="Macdonald", max_len=64) == "M3336"
    assert csan.cutter_loc(surname="Macdonald", max_len=1) == "M3"


# --------------------------------------------------------------------------
# cutter_loc_detailed
# --------------------------------------------------------------------------
def test_cutter_loc_detailed_shape():
    result = csan.cutter_loc_detailed(surname="Carter")
    assert result == {"alpha_prefix": "C", "numeric": 378, "formatted": "C378"}
    # `numeric` must be an int, not a float (== would accept 378.0).
    assert type(result["numeric"]) is int


# --------------------------------------------------------------------------
# error handling
# --------------------------------------------------------------------------
@pytest.mark.parametrize(
    "func", [csan.cutter_sanborn, csan.cutter_loc]
)
def test_missing_input_raises_value_error(func):
    with pytest.raises(ValueError):
        func()


@pytest.mark.parametrize(
    "func", [csan.cutter_sanborn, csan.cutter_loc]
)
def test_blank_input_raises_with_field_names(func):
    with pytest.raises(ValueError) as excinfo:
        func(surname="   ")
    assert "surname" in str(excinfo.value)


def test_untransliterable_name_raises_value_error():
    # A name that normalizes to nothing surfaces as ValueError, not a panic.
    with pytest.raises(ValueError):
        csan.cutter_sanborn(surname="!!!")


# --------------------------------------------------------------------------
# run_cli (the console-script engine)
# --------------------------------------------------------------------------
def test_run_cli_success_returns_zero(capfd):
    code = csan.run_cli(
        ["--cutter-sanborn", "--surname", "Dickens", "--title", "Oliver Twist"]
    )
    assert code == 0
    assert capfd.readouterr().out.strip() == "D548o"


def test_run_cli_both_methods_prints_two_lines(capfd):
    code = csan.run_cli(["--cutter-sanborn", "--cutter-loc", "--surname", "Carter"])
    assert code == 0
    assert capfd.readouterr().out.strip().splitlines() == ["C323", "C378"]


def test_run_cli_runtime_error_returns_one(capfd):
    code = csan.run_cli(["--cutter-loc", "--surname", "   "])
    assert code == 1
    assert "surname" in capfd.readouterr().err


def test_run_cli_usage_error_returns_two():
    # No method flag is a clap usage error.
    assert csan.run_cli(["--surname", "Dickens"]) == 2


def test_run_cli_help_returns_zero():
    assert csan.run_cli(["--help"]) == 0


# --------------------------------------------------------------------------
# package metadata and console script
# --------------------------------------------------------------------------
def test_version_is_exposed():
    assert csan.__version__ == "0.1.0"


def test_public_api_is_exported():
    for name in (
        "cutter_sanborn",
        "cutter_sanborn_detailed",
        "cutter_loc",
        "cutter_loc_detailed",
        "run_cli",
        "main",
        "__version__",
    ):
        assert name in csan.__all__
        assert hasattr(csan, name)


def test_main_exits_with_run_cli_code(monkeypatch):
    # main() wraps run_cli and raises SystemExit with its return code.
    monkeypatch.setattr(sys, "argv", ["csan", "--surname", "Dickens"])
    with pytest.raises(SystemExit) as excinfo:
        csan.main()
    assert excinfo.value.code == 2


def test_console_script_is_installed():
    # The `csan` entry point runs and produces the expected number. The
    # script is installed next to the running interpreter; resolve it there
    # so the test does not depend on the venv being on PATH.
    script = pathlib.Path(sys.executable).with_name("csan")
    if not script.exists():
        pytest.skip("csan console script is not installed in this environment")
    result = subprocess.run(
        [
            str(script),
            "--cutter-sanborn",
            "--surname",
            "Dickens",
            "--title",
            "Oliver Twist",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    assert result.stdout.strip() == "D548o"
