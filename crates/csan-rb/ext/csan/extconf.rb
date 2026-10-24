# frozen_string_literal: true

# Builds the gem's extension, lib/csan/csan_rb.so, from the crate beside this
# file (rb-sys runs Cargo).
require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("csan/csan_rb") do |r|
  # The source gem's manifest carries the workspace's lint levels, some of
  # them `deny`: a newer compiler must not fail a user's install over a lint.
  r.extra_rustflags = ["--cap-lints", "warn"]
end
