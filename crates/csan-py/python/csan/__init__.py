"""Cutter call numbers for library cataloging, powered by Rust.

Generate Cutter-Sanborn three-figure numbers and Library of Congress
cutter numbers from author names or titles::

    import csan

    csan.cutter_sanborn(given_name="Charles", family_name="Dickens",
                          title="Oliver Twist")   # 'D548o'
    csan.cutter_loc(family_name="Carter")       # 'C378'
    csan.cutter_sanborn_detailed(family_name="Borges")
    # {'alpha_prefix': 'B', 'numeric': 732, 'alpha_suffix': None,
    #  'entry': 'Borg', 'formatted': 'B732'}

At least one of ``given_name``/``family_name``/``title`` must be
non-blank; blank strings count as absent. Unusable input raises
``ValueError``.

Installing this package also provides the ``csan`` command-line tool
(see ``csan --help``).
"""

from __future__ import annotations

import sys

from ._csan import (
    __version__,
    cutter_loc,
    cutter_loc_detailed,
    cutter_sanborn,
    cutter_sanborn_detailed,
    run_cli,
)

__all__ = [
    "__version__",
    "cutter_loc",
    "cutter_loc_detailed",
    "cutter_sanborn",
    "cutter_sanborn_detailed",
    "main",
    "run_cli",
]


def main() -> None:
    """Entry point for the ``csan`` console script."""
    raise SystemExit(run_cli(sys.argv[1:]))
