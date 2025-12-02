import re
import unicodedata


def ascii_name(name: str) -> str:
    """
    Normalize (ASCII) a name.

    Args:
        name (str): The input name string.

    Returns:
        The normalized name.

    Raises:
        ValueError: if `name` contains characters other than letters and spaces.

    Examples:
        >>> ascii_name("café")
        'cafe'

        >>> ascii_name("Σailor")
        'ailor'

        >>> ascii_name("hôpital")
        'hopital'
    """
    if not name.isalpha():
        raise ValueError(f"Invalid string: {name}")

    if not name.isascii():
        name = (
            unicodedata.normalize("NFKD", name)
            .encode(encoding="ascii", errors="ignore")
            .decode()
        )

    return name


def compose_name(first_name: str | None, last_name: str) -> tuple[str, str]:
    """
    Compose a name, given a first and a last name.

    Args:
        first_name (str|None): a person's first name;
        last_name (str): a person's last name.

    Returns:
        a tuple with 2 strings, where the first string is a full composed form,
        and the second string is an abbreviated form.

    Examples:
        >>> compose_name("First", "Last")
        ('Last, First', 'Last, F.')
        >>> compose_name(None, "Last")
        ('Last', 'Last')
    """
    if first_name is not None:
        composed_name = f"{last_name}, {first_name}"
        composed_name_abbr = f"{last_name}, {first_name[0]}."
    else:
        composed_name, composed_name_abbr = last_name, last_name

    return (composed_name, composed_name_abbr)


def process_name(first_name: str | None, last_name: str) -> tuple[str | None, str]:
    """
    Process a name, returning normalized strings for usage in `cutter_number` function.

    Args:
        first_name (str|None): first name;
        last_name (str): last name.

    Returns:
        A tuple with 4 strings, where each element is, respectively:
        - The first name, after processing;
        - The last name, after proessing;
        - A composed string with first and last names;
        - The abbreviated composed string.

    Raises:
        ValueError: if any argument contains characters other than letters and spaces
        (from `ascii_name` function).

    Examples:
        >>> process_name("first", "last")
        ('First', 'Last')
    """
    if first_name is not None:
        first_name = ascii_name(re.split(r"[;,.\-' ]+", first_name)[0]).title()

    last_name = ascii_name(re.sub(r"[;,.\-' ]+", "", last_name)).title()

    #if len(last_name) < 2:
    #    raise ValueError("Last Name must have at least 2 letters")

    return first_name, last_name
