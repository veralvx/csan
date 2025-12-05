import argparse
import logging
from typing import cast

from csan.cutter import cutter_call_number, cutter_number
from csan.naming import compose_name, process_name

logger = logging.getLogger(__name__)


def get_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="csan",
        description="Cutter-Sanborn identifier generator.",
    )

    parser.add_argument("-f", "--first-name")
    parser.add_argument("-l", "--last-name", required=True)
    parser.add_argument("-t", "--title")
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
    )

    return parser.parse_args()


def verbalize_args(**kwargs: str | bool | None) -> None:
    name: str = cast(str, kwargs["Name"]).replace(" ", "")
    if not name.isascii() or not name.isalpha():
        logger.warning("works better with ascii and alphabetic name.")

    for k, v in kwargs.items():
        print(f"{k}: {v}")


def main():
    args = get_args()
    first_name, last_name = process_name(args.first_name, args.last_name)
    composed_name, composed_name_abbr = compose_name(first_name, last_name)
    log_level = logging.DEBUG if args.verbose else logging.INFO
    logging.basicConfig(level=log_level, format="%(message)s")

    if args.verbose:
        verbalize_args(
            **{
                "Name": args.first_name + " " + args.last_name
                if first_name is not None
                else args.last_name,
                "First name": first_name,
                "Last name": last_name,
                "Composed name": composed_name,
                "Composed name abbr": composed_name_abbr,
                "Title": args.title,
                "Verbose": args.verbose,
            }
        )

    cutter_n = cutter_number(
        first_name,
        last_name,
        composed_name,
        composed_name_abbr,
    )

    if cutter_n:
        print(
            cutter_call_number(
                last_name,
                cutter_n,
                args.title,
            )
        )
    else:
        raise ValueError("cutter_number returned None")


if __name__ == "__main__":
    main()
