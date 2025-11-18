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

For instance:

- `csan -f John -l Doe` -> D649
- `csan -f John -l Doe -t "My Book"` -> D649m
- `csan -f First -l Last -v` -> L349, with log output to the console
- `csan -f Jorge -l "De la Cruz"` -> D332

## Examples

The following cutter numbers are expected, for their respective names. This is achieved with `cutter_number` function from `csan.cutter`. When run via CLI, the output is the cutter identifier (`cutter_identifier` function), which also includes the cutter number.

| First Name | Last Name    | Cutter Number |
|------------|--------------|:------------------------:|
| Charles    | Dickens      | 548                    |
| Jane       | Austen       | 933                    |
| Mark       | Twain        | 969                    |
| Samuel     | Clemens      | 625                    |
| George     | Orwell       | 79                     |
| Eric       | Blair        | 635                    |
| Virginia   | Woolf        | 913                    |
| Leo        | Tolstoy      | 654                    |
| Fyodor     | Dostoyevsky  | 724                    |
| Herman     | Melville     | 531                    |
| Emily      | Dickinson    | 553                    |
| William    | Shakespeare  | 527                    |
| Agatha     | Christie     | 555                    |
| Stephen    | King         | 52                     |
| Jorge      | De la Cruz   | 332                    |
| Ivan       | Smith        | 649
| Lord       | Sith         | 622                    |