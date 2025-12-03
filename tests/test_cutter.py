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
        ("Jorge", "De la Cruz", 332),
        ("Mary", "Abbot", 128),
        ("M", "Abbott", 133),
        ("M.", "Abbott", 133),
        ("Mary", "Abbott", 133),
        ("Ivan", "Smith", 649),
        ("Ivan.", "Smith", 649),
        ("I", "Smith", 649),
        ("I.", "Smith", 649),
        ("John", "Smith", 652),
        ("Johnny", "Smith", 652),
        ("Jos.", "Smith", 653),
        ("Joseph", "Smith", 653),
        ("Roberto", "Smith", 658),
        ("Robert", "Smith", 658),
        ("Sol", "Smith", 661),
        ("Sophomore", "Smith", 659),
        ("Solstice", "Smith", 661),
        ("Will", "Smith", 663),
        ("William", "Smith", 664),
        ("Wm", "Smith", 664),
        ("Wmayo", "Smith", 664),
        ("VERA", "LVX", 979),
        ("Lord", "Sith", 622),
        ("Emile", "Zola", 86),
        (None, "Aarabc", 113),
        (None, "Aa", 111),
        (None, "Aabbcc", 111),
        (None, "Ba", 111),
        (None, "Baab", 111),
        (None, "Cca", 386),
        (None, "Xz", 9),
        (None, "Za", 11),
        (None, "Zaaz", 11),
        (None, "Zy", 99),
        (None, "Zyz", 99),
        (None, "Zz", 99),
        (None, "A", 111),
        (None, "B", 111),
        (None, "X", 1),
        (None, "Y", 11),
        (None, "Z", 11),
    ],
)
def test_cutter_number_basic(
    first_name: str | None, last_name: str, expected: int
) -> None:
    assert cutter_number(first_name, last_name) == expected


def test_case_insensitivity() -> None:
    assert cutter_number("charles", "dickens") == cutter_number("CHARLES", "DICKENS")


def test_invalid_names() -> None:
    with pytest.raises(ValueError):
        cutter_number("", "Doe")
    with pytest.raises(ValueError):
        cutter_number("Jane", "")
    with pytest.raises(ValueError):
        cutter_number("   ", "   ")
