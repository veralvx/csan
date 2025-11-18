import pytest

from csan.cutter import cutter_number


@pytest.mark.parametrize(
    ("first_name", "last_name", "expected"),
    [
        ("Charles", "Dickens", 548),
        ("Jane", "Austen", 933),
        ("Mark", "Twain", 969),
        ("Samuel", "Clemens", 625),
        ("George", "Orwell", 79),
        ("Eric", "Blair", 635),
        ("Virginia", "Woolf", 913),
        ("Leo", "Tolstoy", 654),
        ("Fyodor", "Dostoyevsky", 724),
        ("Herman", "Melville", 531),
        ("Emily", "Dickinson", 553),
        ("William", "Shakespeare", 527),
        ("Agatha", "Christie", 555),
        ("Stephen", "King", 52),
        ("jorge", "De la Cruz", 332),
        ("Lord", "Sith", 622),
    ],
)
def test_cutter_number_basic(first_name: str, last_name: str, expected: int) -> None:
    assert cutter_number(first_name, last_name) == expected


def test_case_insensitivity() -> None:
    assert cutter_number("charles", "dickens") == cutter_number("CHARLES", "DICKENS")


def test_raises_value_error() -> None:
    """Names not in the Cutter-Sanborn table must raise ValueError."""
    with pytest.raises(ValueError, match="Cutter-Sanborn"):
        cutter_number("xz", "xz")


def test_invalid_names() -> None:
    with pytest.raises(ValueError):
        cutter_number("", "Doe")
    with pytest.raises(ValueError):
        cutter_number("Jane", "")
    with pytest.raises(ValueError):
        cutter_number("   ", "   ")

