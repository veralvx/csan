"""Type stubs for the compiled ``csan._csan`` extension module."""

__version__: str

def cutter_sanborn(
    given_name: str | None = None,
    family_name: str | None = None,
    title: str | None = None,
    prefix_len: int | None = None,
    suffix_len: int | None = None,
) -> str: ...
def cutter_sanborn_detailed(
    given_name: str | None = None,
    family_name: str | None = None,
    title: str | None = None,
    prefix_len: int | None = None,
    suffix_len: int | None = None,
) -> dict[str, str | int | None]: ...
def cutter_loc(
    given_name: str | None = None,
    family_name: str | None = None,
    title: str | None = None,
    max_len: int | None = None,
) -> str: ...
def cutter_loc_detailed(
    given_name: str | None = None,
    family_name: str | None = None,
    title: str | None = None,
    max_len: int | None = None,
) -> dict[str, str | int]: ...
def run_cli(args: list[str]) -> int: ...
