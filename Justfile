alias check := checks
alias linter := lint
alias lints := lint
alias lintr := lint
alias ruff := format
alias ruff-check := ruff-checks
alias test := pytest
alias tests := pytest
alias typing := static

[group("Misc")]
[doc("List Just recipes")]
default:
  @just --list

[group("Misc")]
dev-deps:
  uv add --dev mypy basedpyright ty pyrefly ruff pytest

[group("Ruff")]
[doc("Format code (modifies files)")]
format:
  @echo
  uv run ruff format

[group("Checks")]
[group("Ruff")]
[doc("Check if files are formatted")]
format-check:
  @echo
  uv run ruff format --check


[group("Checks")]
[group("Ruff")]
[doc("Run linter (does not modify files)")]
lint:
  @echo
  uv run ruff check


[group("Checks")]
[group("Ruff")]
[doc("Run Ruff checks: format-check + lint")]
ruff-checks: format-check lint


[group("Checks")]
[group("Tests")]
pytest:
  @echo
  uv run pytest --doctest-modules


[group("Checks")]
[doc("Run static typing checks")]
static:
  @echo
  uv run basedpyright
  @echo
  uv run mypy .
  @echo
  uv run pyrefly check
  @echo
  uv run ty check


[group("Checks")]
[group("Tests")]
[group("Ruff")]
[doc("Run all checks: Ruff linter/formatter, static typing, tests")]
checks:
  uv lock --check
  @just ruff-checks pytest static


[doc("Template steps for a release. Usage: just release <major|minor|patch>")]
[group("Misc")]
release semver:
  #!/usr/bin/sh
  just checks
  uv version --bump {{ semver }}
  new_version="$(uv version --short)"
  uv sync
  git add pyproject.toml uv.lock
  git commit -m "chore(release): bump version"
  git cliff --tag "$new_version" --output CHANGELOG.md
  git add CHANGELOG.md
  git commit -m "chore(release): update CHANGELOG"
  git tag "$new_version"
  git push origin main --tags

[doc("Clean temporary files")]
[group("Misc")]
clean:
    @echo "Cleaning temporary files, except in .venv and .env"
    find . -type d \
        -name .venv -prune \
        -o -name .env -prune \
        -o \( \
            -name .mypy_cache -o \
            -name .pytest_cache -o \
            -name __pycache__ -o \
            -name .ruff_cache -o \
            -name .idea -o \
            -name dist -o \
            -name main.spec -o \
            -name build \
        \) \
        -print \
        -exec rm -rf {} +
