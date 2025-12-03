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
) -> int | None:
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

    # Last names with one letter must be the start of a series for a letter entry.
    # All entries, for each new starting letter, start with [A-z]a pattern.
    # Thus, the right side of bisection will have all entries where entry > last_name.
    # The first element of this list will have last_name's cutter number when retrieved.
    if len(last_name) == 1:
        logger.debug("last_name has len == 1. Retrieving first entry with this letter.")
        cutter_bisect = bisect_right(tuple_cutter := tuple(CUTTER_DATA), last_name)
        return CUTTER_DATA[tuple_cutter[cutter_bisect]]

    # There are only 5 entries in CUTTER_DATA with 2 or more letters in the first name.
    # All of them are associated to Smiths.
    # Matching these edge cases, composed_name_decrescent can be built from
    # composed_name_abbr instead of composed_name
    if last_name == "Smith" and first_name:
        match first_name:
            case f if f.startswith("John"):
                return CUTTER_DATA["Smith, John"]
            case f if f.startswith("Jos"):
                return CUTTER_DATA["Smith, Jos."]
            case f if f.startswith("Robert"):
                return CUTTER_DATA["Smith, Robert"]
            case f if f.startswith("Sol"):
                return CUTTER_DATA["Smith, Sol"]
            case f if f == "William" or f.startswith("Wm"):
                return CUTTER_DATA["Smith, Wm."]

    if (composed_name is None) or (composed_name_abbr is None):
        composed_name, composed_name_abbr = compose_name(first_name, last_name)

    composed_name_catdot = (
        composed_name + "."
        if "." not in composed_name
        else composed_name.replace(".", "")
    )

    obvious_attempts = [composed_name]

    # if first_name, then composed_name != last_name
    if first_name:
        obvious_attempts.append(composed_name_catdot)
        obvious_attempts.append(last_name)

    # There are two cases where composed_name == composed_name_abbr
    # 1: first_name is None, in which case both vars are equal to last_name
    # 2: len(first_name) == 1,
    # in which case both vars are equal to f"{last_name}, {first_name}" (no final dot)
    if composed_name != composed_name_abbr:
        insert_obv = [composed_name_abbr, composed_name_abbr.replace(".", "")]
        for i in insert_obv:
            obvious_attempts.insert(-1, i)

    logger.debug("Obvious attempts: %s", obvious_attempts)

    if attempt := next((a for a in obvious_attempts if a in CUTTER_DATA), None):
        logger.debug("Returning match '%s' from obvious attempts", attempt)
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
    #    k for k in CUTTER_DATA if k.startswith(last_name[0:2])
    # ]

    # Build composed_name_decrescent from composed_name_abbr instead of
    # composed_name, since 2+ letters in first name cases are covered in Smiths match.
    composed_name_decrescent = [
        composed_slice
        for i in range(len(composed_name_abbr), 1, -1)
        if not (composed_slice := composed_name_abbr[:i]).endswith((",", " "))
    ]

    if first_name and len(first_name) == 1:
        composed_name_decrescent.insert(1, composed_name_catdot)

    logger.debug("\nDecrescent last name list: %s", composed_name_decrescent)

    cutter_s = None
    max_len = len(max(composed_name, composed_name_catdot))

    for pos, name in enumerate(composed_name_decrescent):
        sub_sieved = [
            k for k in sieved_data if (k.startswith(name) and (len(k) <= max_len))
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

                logger.debug("Pairs: %s\nMatch: %s", pairs, cutter_s)

                if all(x >= y and (x.isalpha() == y.isalpha()) for x, y in pairs):
                    cutter_s = candidate

            logger.debug("Match: %s\n", cutter_s)

        if cutter_s is not None:
            return CUTTER_DATA[cutter_s]

        logger.debug("\n")

    if cutter_s is None:
        logger.debug("Last resource: bisect_right using first two letters of last name")
        cutter_bisect = bisect_right(tuple_data, composed_name_decrescent[-1])
        return CUTTER_DATA[tuple_data[max(0, cutter_bisect - 1)]]

    return None


def cutter_call_number(
    last_name: str, cutter_number: int, title: str | None = None
) -> str:
    """
    Generate call-number based on last name, cutter-sanborn numbertitle.

    Args:
        last_name (str): a person's last name;
        cutter_number (int): the cutter-sanborn number;
        title (str): a work's title.

    Returns:
        A call number identifier

    Examples:
        >>> cutter_call_number("Doe", 649, "Title")
        'D649t'
    """
    title = "" if title is None else title[0].lower()
    return last_name[0] + str(cutter_number) + title
