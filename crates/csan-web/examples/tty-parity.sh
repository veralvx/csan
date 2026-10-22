#!/usr/bin/env bash
# Checks that the npm package's `csan` command prints byte for byte what the
# native binary prints on a terminal, and exits the same: the same text and
# ANSI styles under the same color settings. The bin shim mirrors clap's
# terminal rules (anstream), so this catches a clap update that changes them.
#
#     tty-parity.sh <native csan binary> <pkg-node/bin/csan.cjs>
#
# Runs each case in a pseudo-terminal with `script` (util-linux), from an
# environment holding only the color settings under test.
set -euo pipefail

native="$(realpath "$1")"
npm_bin="$(realpath "$2")"
node="$(command -v node)"

cases=(
  "--help"
  "-h"
  "--method dewey"
  "--surnam X"
  "--prefix-len 4294967296"
  "--surname Carter"
  "--version"
  ""
)
environments=(
  "TERM=xterm-256color"
  "TERM=dumb"
  "TERM=xterm-256color NO_COLOR=1"
  "TERM=dumb CLICOLOR_FORCE=1"
  "TERM=xterm-256color CLICOLOR=0"
  "CI=true"
)

# Prints what `$1 $2` writes to a terminal under the environment `$3`,
# then its exit code (`script -e` passes the command's status through).
on_tty() {
  local status=0
  # shellcheck disable=SC2086 # the environment and arguments split on purpose
  env -i PATH="$PATH" HOME="${HOME:-/}" $3 script -qec "$1 $2" /dev/null || status=$?
  echo "[exit $status]"
}

failures=0
styled=0
for environment in "${environments[@]}"; do
  for args in "${cases[@]}"; do
    expected="$(on_tty "$native" "$args" "$environment" | od -c)"
    actual="$(on_tty "$node $npm_bin" "$args" "$environment" | od -c)"
    if [ "$expected" != "$actual" ]; then
      echo "DIFF: $environment csan $args" >&2
      failures=$((failures + 1))
    elif grep -q '033' <<<"$expected"; then
      styled=$((styled + 1))
    fi
  done
done

total=$((${#environments[@]} * ${#cases[@]}))
if [ "$failures" -ne 0 ]; then
  echo "tty parity: $failures of $total cases differ" >&2
  exit 1
fi
# Guard against a vacuous pass (e.g. no terminal, so nothing is styled).
if [ "$styled" -eq 0 ]; then
  echo "tty parity: no case produced styled output; is script(1) giving a terminal?" >&2
  exit 1
fi
echo "tty parity: $total cases identical ($styled styled)"
