# CSAN: Cutter-Sanborn Number Generator

`csan` is a CLI tool that generates a Cutter-Sanborn identifier, given a first and a last name.

The Cutter-Sanborn identifier, commonly called "Cutter number", is an alphanumeric code that forms part of the call number in library classification systems in order to arrange books alphabetically by author. It consists of the first letter of the author's last name followed by three-digit number derived from a predefined [table](https://github.com/veralvx/cutter-sanborn-table). This system was originally developed by Charles Cutter, and revised by Kate Sanborn.

## Installation

You can install this tool using `pip` or `uv`:

```console
uv tool install csan
```

```console
pip install csan
```

## Usage  


```console
usage: csan [-h] -f FIRST_NAME -l LAST_NAME [-t TITLE] [-v]

Cutter-Sanborn identifier generator.

options:
  -h, --help            show this help message and exit
  -f FIRST_NAME, --first-name FIRST_NAME
  -l LAST_NAME, --last-name LAST_NAME
  -t TITLE, --title TITLE
  -v, --verbose
```

## Examples  

- `csan -f John -l Doe` -> D649
- `csan -f John -l Doe -t "My Book"` -> D649m
- `csan -f First -l Last -v` -> L349, with log output to the console
- `csan -f Jorge -l "De la Cruz"` -> D332
