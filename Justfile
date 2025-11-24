alias test := pytest
alias tests := pytest


default:
  @just --list


[group("Check")]
static:
  @echo
  uv run basedpyright
  @echo
  uv run mypy .
  @echo
  uv run pyrefly check
  @echo
  uv run ty check
  

[group("Check")]
[group("Ruff")]
format-check:
  @echo
  uv run ruff format --check


[group("Check")]
[group("Ruff")]
linter:
  @echo
  uv run ruff check


[group("Check")]
[group("Ruff")]
ruff: format-check linter


[group("Check")]
[group("Tests")]
pytest:
  @echo
  uv run pytest --doctest-modules


[group("Check")]
[group("Tests")]
[group("Ruff")]
check: static ruff pytest


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
