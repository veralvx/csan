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


release tag:
  #!/usr/bin/sh
  git cliff --unreleased --tag {{ tag }} --output CHANGELOG.md
  git add .
  git commit -m "chore(release): update CHANGELOG.md for $tag"
  git tag "$tag"
  git push origin main --tags


clean:
  #!/bin/sh
  list=(".mypy_cache" ".pytest_cache" "__pycache__" ".ruff_cache" ".idea" "dist")
  for dir in "${list[@]}"; do
    find . -type d -name "$dir" -print -exec rm -rf {} +
  done
