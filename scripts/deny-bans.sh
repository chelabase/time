#!/usr/bin/env bash
# `cargo deny check bans` refuses chrono and humantime in the library's graph, even though the
# oracle member (excluded from the deny graph) depends on both. Each case copies the workspace
# to a scratch directory, adds one dependency to chelabase-time's [dependencies] there, and
# expects the ban message (an advisory failure alone wouldn't prove the ban).
#
#   bash scripts/deny-bans.sh   (needs cargo-deny)
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

bad=0
# expect_banned <case> <dependency line> <crate the ban must name>
expect_banned() {
  local dir="$tmp/$1" dep="$2" crate="$3" out
  mkdir -p "$dir"
  cp -r "$root/Cargo.toml" "$root/Cargo.lock" "$root/deny.toml" "$root/src" "$root/oracle" "$dir/"
  # The first [dependencies] table is the library's.
  sed -i "0,/^\[dependencies\]$/s//[dependencies]\n$dep/" "$dir/Cargo.toml"
  if out="$(cd "$dir" && cargo deny --offline check bans 2>&1)"; then
    echo "deny-bans: cargo deny passed with $crate in chelabase-time's graph" >&2
    bad=1
  elif ! grep -qE "crate '$crate = [^']*' is explicitly banned" <<<"$out"; then
    echo "deny-bans: cargo deny failed, but not with a ban on $crate:" >&2
    echo "$out" >&2
    bad=1
  else
    echo "deny-bans: $1: $crate is banned"
  fi
}

# chrono named directly (the version the oracle locks, so no resolution is needed).
expect_banned direct 'chrono = "=0.4.45"' chrono
# humantime pulled in through another crate.
expect_banned transitive 'humantime-serde = "=1.1.1"' humantime

[[ $bad -eq 0 ]] && echo "deny-bans: ok"
exit $bad
