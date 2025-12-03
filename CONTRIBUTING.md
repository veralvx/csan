# Contributing

## Requirements

- [`uv`](https://docs.astral.sh/uv/)
- [`just`](https://github.com/casey/just)


## Setup

Fork this repository and clone your fork:

```console
git clone https://github.com/<your-username>/csan.git
cd csan
```

Sync dependencies:

```console
uv sync --locked
```

Create a branch for your changes:

```console
git switch -c <short-desc>
```


## Commands

| Command | Description |
| :--- | :--- |
| `just format` | Format files |
| `just format-check` | Check if files are well formatted |
| `just lint` | Lint files |
| `just ruff-checks` | Run both `just format-check` and `just lint` |
| `just test` | Run tests |
| `just static` | Static Typing Checks |
| `just check` | All Checks. Almost equivalent to what will be run in CI |


**Important**: run `just check` before submitting a pull request.


## Commit messages

Use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#specification).


## Legal

By contributing, you agree that your contributions will be licensed under the repository license.

