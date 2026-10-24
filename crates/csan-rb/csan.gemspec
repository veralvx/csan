# frozen_string_literal: true

require_relative "lib/csan/version"

Gem::Specification.new do |spec|
  spec.name = "csan"
  spec.version = Csan::VERSION
  spec.authors = ["VERALVX"]
  spec.email = ["veralvx@veralvx.com"]

  spec.summary = "Cutter-Sanborn and Library of Congress cutter numbers for library cataloging"
  spec.description = <<~DESCRIPTION
    Generate Cutter-Sanborn three-figure and Library of Congress cutter
    numbers from author names and titles, with the csan command-line tool.
    The numbers come from the csan Rust library (a native extension), so they
    match the csan CLI and its Python, JavaScript, and Typst packages.
  DESCRIPTION
  spec.homepage = "https://github.com/veralvx/csan"
  # RubyGems takes a dual licence as a list of SPDX identifiers.
  spec.licenses = ["MIT", "Apache-2.0"]
  spec.required_ruby_version = ">= 3.3"

  spec.metadata = {
    "homepage_uri" => spec.homepage,
    "source_code_uri" => "https://github.com/veralvx/csan/tree/main/crates/csan-rb",
    "changelog_uri" => "https://github.com/veralvx/csan/blob/main/CHANGELOG.md",
    "bug_tracker_uri" => "https://github.com/veralvx/csan/issues"
  }

  # The source gem's crate is `cargo package`'s (rake source_gem), not the
  # workspace's manifest; a platform gem drops the Rust files altogether.
  spec.files = Dir[
    "lib/**/*.rb", "exe/*", "sig/**/*.rbs", "ext/**/*.{rb,rs,toml,lock}",
    "README.md", "LICENSE-APACHE", "LICENSE-MIT"
  ]
  spec.bindir = "exe"
  spec.executables = ["csan"]
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/csan/extconf.rb"]

  # Builds the extension on install from the source gem; the platform gems
  # need no compiler and drop it.
  spec.add_dependency "rb_sys", "~> 0.9.130"
end
