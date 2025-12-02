import logging
from bisect import bisect_left, bisect_right

from .data import CUTTER_DATA
from .naming import compose_name, process_name

logger = logging.getLogger(__name__)


def cutter_number(
    first_name: str | None,
    last_name: str,
    composed_name: str | None = None,
    composed_name_abbr: str | None = None,
) -> int:
    """
    Generate/Retrieve a cutter-sanborn number, given a first and last name.

    Args:
        first_name (str): a person's first name (e.g. "Jane");
        last_name (str): a person's last name (e.g. "Doe");
        composed_name (str|None): a composition in the format "Last, First";
        composed_name_abbr (str|None): an abbreviated compositon, format "Last, F."

    Returns:
        An integer, retrieved from the Cutter-Sanborn table.

    Raises:
        ValueError: if no Cutter-Sanborn number is found.

    Examples:
        >>> cutter_number("First", "Last")
        349
    """
    first_name, last_name = process_name(first_name, last_name)

    if len(last_name) == 1:
        logger.debug("last_name has len == 1. Retrieving first entry with this letter.")
        cutter_bisect = bisect_right(tuple_cutter := tuple(CUTTER_DATA), last_name)
        return CUTTER_DATA[tuple_cutter[cutter_bisect]]

    if (composed_name is None) or (composed_name_abbr is None):
        composed_name, composed_name_abbr = compose_name(first_name, last_name)

    cutter_s = None

    obvious_attempts = [
        composed_name,
        last_name,
    ]

    if first_name is not None:
        obvious_attempts.insert(1, last_name + f", {first_name[:3]}.")

    if composed_name != composed_name_abbr:
        obvious_attempts.insert(
            -1,
            composed_name_abbr,
        )

    if attempt := next((a for a in obvious_attempts if a in CUTTER_DATA), None):
        logger.debug(
            "Obvious attempts: %s\nReturning match '%s' from obvious attempts",
            obvious_attempts,
            attempt,
        )
        return CUTTER_DATA[attempt]

    tuple_data = tuple(CUTTER_DATA)
    bisect_entrypoint = bisect_left(tuple_data, last_name[0:2])
    bisect_endpoint = bisect_left(
        tuple_data,
        last_name[0] + chr(ord(last_name[1]) + 1),
        lo=bisect_entrypoint,
    )

    sieved_data = tuple_data[bisect_entrypoint:bisect_endpoint]

    # alternative to bisect (a bit slower)
    # sieved_data = [
    #    k for k in tuple_data[bisect_entrypoint - 1 :] if k.startswith(last_name[0:2])
    # ]

    composed_name_decrescent = [
        composed_name[: i + 1]
        for i in range(len(composed_name), 0, -1)
        if not composed_name[: i + 1].endswith((",", " "))
    ]

    logger.debug("\nDecrescent last name list: %s", composed_name_decrescent)

    for pos, name in enumerate(composed_name_decrescent[1:]):
        sub_sieved = [
            k
            for k in sieved_data
            if (k.startswith(name) and (len(k) <= len(composed_name)))
        ]

        if not sub_sieved:
            continue

        miss_lttrs = list(composed_name_decrescent[0][len(name) :])

        logger.debug("Name: %s\nPos: %s\nSieved List: %s", name, pos, sub_sieved)

        for candidate in sub_sieved:
            logger.debug("Candidate: %s", candidate)

            if candidate == name:
                cutter_s = candidate
            else:
                xtra_lttrs = list(candidate[len(name) :].replace(".", ""))
                logger.debug("Missing Letters: %s", miss_lttrs)
                logger.debug("Extra Letters: %s", xtra_lttrs)

                if not (pairs := list(zip(miss_lttrs, xtra_lttrs, strict=False))):
                    logger.debug("Pairs are empty. Continuing...")
                    continue

                logger.debug("Pairs: %s\nMatch: %s.", pairs, cutter_s)

                cutter_s = (
                    candidate
                    if all(x >= y and (x.isalpha() == y.isalpha()) for x, y in pairs)
                    else cutter_s
                )

            logger.debug("Match: %s\n", cutter_s)

        if cutter_s is not None:
            return CUTTER_DATA[cutter_s]

        logger.debug("\n")

    if cutter_s is None:
        logger.debug(
            "Last resource: bisect on last name, first letter; get adjacent left entry"
        )
        cutter_bisect = bisect_right(tuple_data, composed_name_decrescent[-1])
        return CUTTER_DATA[tuple_data[cutter_bisect - 1]]
        # raise ValueError("Unable to retrieve Cutter-Sanborn number.")

    return 0


# first_letters = last_name[0] if last_name[1] in "aeiou" else last_name[0:2].upper()
def cutter_identifier(
    last_name: str, cutter_number: int, title: str | None = None
) -> str:
    """
    Generate an identifier based on last name, cutter-sanborn number and a title.

    Args:
        last_name (str): a person's last name;
        cutter_number (int): the cutter-sanborn number;
        title (str): a work's title.

    Returns:
        A string identifier

    Examples:
        >>> cutter_identifier("Doe", 649, "Title")
        'D649t'
    """
    title = "" if title is None else title[0].lower()
    return last_name[0] + str(cutter_number) + title
