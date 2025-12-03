alias checks := check
alias linter := lint
alias lints := lint
alias lintr := lint
alias test := pytest
alias tests := pytest
alias type := static
alias typing := static

[doc("List Just recipes")]
default:
  @just --list

dev-deps:
  uv add --dev mypy basedpyright ty pyrefly ruff pytest


[group("Checks")]
[group("Ruff")]
format-check:
  @echo
  uv run ruff format --check


[group("Checks")]
[group("Ruff")]
lint:
  @echo
  uv run ruff check


[group("Checks")]
[group("Ruff")]
[doc("Run Ruff checks: formatter and linter")]
ruff:
  uv run ruff format
  @just format-check lint


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
[doc("Run all checks: static typing, Ruff linter/formatter and tests")]
check: 
  uv sync
  @just ruff pytest static 


[group("Ruff")]
format:
  uv run ruff format

[doc("semver: major|minor|patch")]
release tag semver:
  #!/usr/bin/sh
  just check
  uv version --bump {{ semver }}
  uv sync
  git cliff --unreleased --tag {{ tag }} --output CHANGELOG.md
  git add .
  git commit -m "chore(release): bump version for $tag"
  git tag "$tag"
  git push origin main --tags

[doc("Clean temporary files")]
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
            -name dist \
        \) \
        -print \
        -exec rm -rf {} +
