"""Type stubs for the compiled ``csan._csan`` extension module."""

# pyo3 lists the module's members, in the order they are added.
__all__ = [
    "cutter_sanborn",
    "cutter_sanborn_detailed",
    "cutter_loc",
    "cutter_loc_detailed",
    "run_cli",
    "__version__",
]

__version__: str

def cutter_sanborn(
    forename: str | None = None,
    surname: str | None = None,
    title: str | None = None,
    prefix_len: int | None = None,
    suffix_len: int | None = None,
    *,
    lang: str | None = None,
    strip_article: bool = False,
) -> str: ...
def cutter_sanborn_detailed(
    forename: str | None = None,
    surname: str | None = None,
    title: str | None = None,
    prefix_len: int | None = None,
    suffix_len: int | None = None,
    *,
    lang: str | None = None,
    strip_article: bool = False,
) -> dict[str, str | int | None]: ...
def cutter_loc(
    forename: str | None = None,
    surname: str | None = None,
    title: str | None = None,
    max_len: int | None = None,
    *,
    lang: str | None = None,
    strip_article: bool = False,
) -> str: ...
def cutter_loc_detailed(
    forename: str | None = None,
    surname: str | None = None,
    title: str | None = None,
    max_len: int | None = None,
    *,
    lang: str | None = None,
    strip_article: bool = False,
) -> dict[str, str | int]: ...
def run_cli(args: list[str]) -> int: ...
