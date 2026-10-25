"""Cutter call numbers for library cataloging, powered by Rust.

Generate Cutter-Sanborn three-figure numbers and Library of Congress
cutter numbers from author names or titles::

    import csan

    csan.cutter_sanborn(forename="Charles", surname="Dickens",
                        title="Oliver Twist")  # 'D548o'
    csan.cutter_loc(surname="Carter")  # 'C378'
    csan.cutter_sanborn_detailed(surname="Borges")
    # {'alpha_prefix': 'B', 'alpha_suffix': None, 'entry': 'Borg',
    #  'formatted': 'B732', 'numeric': 732}

At least one of ``forename``/``surname``/``title`` must be
non-blank; blank strings count as absent. With ``strip_article=True``, the
title is filed without its initial article in the language ``lang`` names
(``"en"``, ``"pt-BR"``, ...)::

    csan.cutter_sanborn(surname="Dickens", title="A Christmas Carol",
                        lang="en", strip_article=True)  # 'D548c'

A blank ``lang`` counts as absent, and author names are never stripped.
Unusable input, an unknown ``lang`` (even with ``strip_article`` off), or
``strip_article`` without ``lang`` raises ``ValueError``.

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
