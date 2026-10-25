# frozen_string_literal: true

require "rbconfig"
require_relative "csan/version"

# A precompiled (platform) gem carries one extension per Ruby minor version,
# in lib/csan/<major.minor>/; the source gem, and a local `rake compile`,
# build one in lib/csan/. (Chosen by file, so that an extension that fails to
# load reports its own error.)
versioned = File.join(__dir__, "csan", RUBY_VERSION[/\A\d+\.\d+/], "csan_rb.#{RbConfig::CONFIG["DLEXT"]}")
if File.exist?(versioned)
  require versioned
else
  require_relative "csan/csan_rb"
end

# Cutter-Sanborn three-figure and Library of Congress cutter numbers for
# library cataloging, from author names and titles.
#
# Every function takes the author's `forename` and `surname` and the work's
# `title` as keyword arguments, at least one of them non-blank (blank strings
# count as absent), and raises `ArgumentError` on unusable input. A length
# that is not an Integer raises `TypeError` (a Float is truncated), and a
# negative one `RangeError`.
#
# Every function also takes `lang` and `strip_article`: with a truthy
# `strip_article`, the title is filed without its initial article in the
# language `lang` names, an ISO 639 code, a BCP 47 tag or a POSIX locale
# (`"en"`, `"pt-BR"`), case-insensitive; `csan --help` lists them all.
# `lang` alone changes nothing, a blank `lang` counts as absent, and author
# names are never stripped. An unknown `lang` (even with `strip_article`
# off), or `strip_article` without `lang`, raises `ArgumentError`, and a
# `lang` that is not a String `TypeError`.
#
# @example
#   Csan.cutter_sanborn(forename: "Charles", surname: "Dickens", title: "Oliver Twist")
#   # => "D548o"
#   Csan.cutter_loc(surname: "Carter") # => "C378"
#   Csan.cutter_sanborn(surname: "Dickens", title: "A Christmas Carol", lang: "en", strip_article: true)
#   # => "D548c"
module Csan
  # The Cutter-Sanborn three-figure number.
  #
  # @param forename [String, nil] the author's forename
  # @param surname [String, nil] the author's surname
  # @param title [String, nil] the work's title: the work mark, or the whole
  #   number when there is no author
  # @param prefix_len [Integer, nil] letters of the name kept before the
  #   number (default 1)
  # @param suffix_len [Integer, nil] letters of the title in the work mark
  #   (default 1)
  # @param lang [String, nil] the work's language, for `strip_article`
  # @param strip_article [Boolean, nil] file the title without its initial
  #   article in `lang` when truthy (default false)
  # @return [String] the number, e.g. "D548o"
  # @raise [ArgumentError] on unusable input, an unknown `lang`, or
  #   `strip_article` without `lang`
  def self.cutter_sanborn(forename: nil, surname: nil, title: nil, prefix_len: nil, suffix_len: nil,
    lang: nil, strip_article: false)
    Native.cutter_sanborn(forename, surname, title, prefix_len, suffix_len, lang, strip_article)
  end

  # The Cutter-Sanborn number's parts.
  #
  # Takes the arguments of {cutter_sanborn}.
  #
  # @return [Hash{Symbol => String, Integer, nil}] `:alpha_prefix`,
  #   `:numeric`, `:alpha_suffix` (nil without a work mark), `:entry` (the
  #   table entry that matched), and `:formatted`
  # @raise [ArgumentError] as {cutter_sanborn} does
  def self.cutter_sanborn_detailed(forename: nil, surname: nil, title: nil, prefix_len: nil, suffix_len: nil,
    lang: nil, strip_article: false)
    Native.cutter_sanborn_detailed(forename, surname, title, prefix_len, suffix_len, lang, strip_article)
  end

  # The Library of Congress cutter number.
  #
  # @param forename [String, nil] the author's forename
  # @param surname [String, nil] the author's surname
  # @param title [String, nil] the work's title, cuttered when there is no
  #   author
  # @param max_len [Integer, nil] digits in the number (default 3, at most 4)
  # @param lang [String, nil] the work's language, for `strip_article`
  # @param strip_article [Boolean, nil] file the title without its initial
  #   article in `lang` when truthy (default false)
  # @return [String] the number, e.g. "C378"
  # @raise [ArgumentError] on unusable input, an unknown `lang`, or
  #   `strip_article` without `lang`
  def self.cutter_loc(forename: nil, surname: nil, title: nil, max_len: nil,
    lang: nil, strip_article: false)
    Native.cutter_loc(forename, surname, title, max_len, lang, strip_article)
  end

  # The Library of Congress number's parts.
  #
  # Takes the arguments of {cutter_loc}.
  #
  # @return [Hash{Symbol => String, Integer}] `:alpha_prefix`, `:numeric`,
  #   and `:formatted`
  # @raise [ArgumentError] as {cutter_loc} does
  def self.cutter_loc_detailed(forename: nil, surname: nil, title: nil, max_len: nil,
    lang: nil, strip_article: false)
    Native.cutter_loc_detailed(forename, surname, title, max_len, lang, strip_article)
  end

  # Runs the `csan` command-line interface, the engine of the gem's `csan`
  # executable: its options, help, messages, and exit codes are the
  # standalone binary's. It writes to the process's standard output and
  # error directly, after flushing `$stdout` and `$stderr`.
  #
  # @param args [Array<String>] the arguments, without the program name
  # @return [Integer] the exit code: 0, 1 for an unusable input, 2 for a
  #   usage error
  def self.run_cli(args)
    $stdout.flush
    $stderr.flush
    Native.run_cli(args)
  end
end
