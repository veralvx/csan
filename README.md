# CSAN: Cutter-Sanborn Call Number Generator

`csan` generates a Cutter-Sanborn call number, given a last name and an optional first name. It can be used as a CLI tool or as a Python library.

The Cutter-Sanborn call number, commonly called "Cutter number", is an alphanumeric code that forms part of the call number in library classification systems in order to arrange books alphabetically by author. It consists of the first letter of the author's last name followed by three-digit number derived from a predefined [table](https://github.com/veralvx/cutter-sanborn-table). This system was originally developed by Charles Cutter, and revised by Kate Sanborn.

## Installation

You can install this tool using [`uv`](https://docs.astral.sh/uv/) or `pip`:

```console
uv tool install csan
```

```console
pip install csan
```

## CLI Usage 


```console
usage: csan [-h] [-f FIRST_NAME] -l LAST_NAME [-t TITLE] [-v]

Cutter-Sanborn identifier generator.

options:
  -h, --help            show this help message and exit
  -f, --first-name FIRST_NAME
  -l, --last-name LAST_NAME
  -t, --title TITLE
  -v, --verbose
```

For instance:



### Examples

- `csan -f John -l Doe` -> D649
- `csan -f John -l Doe -t "My Book"` -> D649m
- `csan -f First -l Last -v` -> L349, with log output to the console
- `csan -f Jorge -l "De la Cruz"` -> D332

The following table shows expected Cutter call numbers for their respective names, using CLI

| First Name | Last Name    | Cutter Call Number |
|------------|--------------|:------------------:|
| Jane       | Austen       | A933               |
| Eric       | Blair        | B635               |
| Agatha     | Christie     | C555               |
| Samuel     | Clemens      | C625               |
| Jorge      | De la Cruz   | D332               |
| Charles    | Dickens      | D548               |
| Emily      | Dickinson    | D553               |
| Fyodor     | Dostoyevsky  | D724               |
| Stephen    | King         | K52                |
| VERA       | LVX          | L979               |
| Herman     | Melville     | M531               |
| George     | Orwell       | O79                |
| William    | Shakespeare  | S527               |
| Lord       | Sith         | S622               |
| Ivan       | Smith        | S649               |
| William    | Smith        | S664               |
| Leo        | Tolstoy      | T654               |
| Mark       | Twain        | T969               |
| Virginia   | Woolf        | W913               |
| Emile      | Zola         | Z86                |


## Library Usage

There are two relevant functions in `csan` package:

- `csan.cutter.cutter_number`: return the integer part of the cutter call number
- `csan.cutter.cutter_call_number`: return the call number

Also, the entire Cutter-Sanborn table can be retrieved as a Python `dict`:

- `csan.table.CUTTER_TABLE`


### Example

```python
from csan.cutter import cutter_call_number, cutter_number
from csan.table import CUTTER_TABLE


def main():
    print(CUTTER_TABLE)

    cutter_num = cutter_number("First", "Last")
    cutter_call_num = cutter_call_number("Last", cutter_num)

    print(cutter_num)
    print(cutter_call_num)


if __name__ == "__main__":
    main()
```