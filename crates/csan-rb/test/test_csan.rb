# frozen_string_literal: true

# Tests for the csan gem, through the same Ruby interface users call,
# against the built extension (the crate's Rust test harness cannot link it):
#
#   bundle exec rake compile test   (from crates/csan-rb)

require "minitest/autorun"
require "open3"
require "rbconfig"
require "csan"

class TestCutterSanborn < Minitest::Test
  def test_with_author_and_title
    assert_equal "D548o", Csan.cutter_sanborn(forename: "Charles", surname: "Dickens", title: "Oliver Twist")
  end

  def test_surname_only
    assert_equal "B732", Csan.cutter_sanborn(surname: "Borges")
  end

  def test_forename_only_matches_surname_only
    # A lone forename is cuttered like a lone surname.
    assert_equal Csan.cutter_sanborn(surname: "Prince"), Csan.cutter_sanborn(forename: "Prince")
  end

  def test_prefix_and_suffix_len
    assert_equal "Di548oli", Csan.cutter_sanborn(surname: "Dickens", title: "Oliver Twist", prefix_len: 2, suffix_len: 3)
  end

  def test_title_only
    assert_equal "B481", Csan.cutter_sanborn(title: "Beowulf")
  end

  def test_detailed
    assert_equal(
      {alpha_prefix: "D", numeric: 548, alpha_suffix: "o", entry: "Dicke", formatted: "D548o"},
      Csan.cutter_sanborn_detailed(forename: "Charles", surname: "Dickens", title: "Oliver Twist")
    )
  end

  def test_detailed_without_work_mark
    result = Csan.cutter_sanborn_detailed(surname: "Borges")
    assert_nil result.fetch(:alpha_suffix)
    assert_equal "Borg", result.fetch(:entry)
    assert_instance_of Integer, result.fetch(:numeric)
  end
end

class TestCutterLoc < Minitest::Test
  def test_known_value
    assert_equal "C378", Csan.cutter_loc(surname: "Carter")
  end

  def test_title_numeral_rule
    assert_equal "A153", Csan.cutter_loc(title: "100 best recipes")
  end

  def test_max_len_is_clamped
    assert_equal "M3336", Csan.cutter_loc(surname: "Macdonald", max_len: 64)
    assert_equal "M3", Csan.cutter_loc(surname: "Macdonald", max_len: 1)
  end

  def test_detailed
    assert_equal({alpha_prefix: "C", numeric: 378, formatted: "C378"}, Csan.cutter_loc_detailed(surname: "Carter"))
  end
end

class TestErrors < Minitest::Test
  FUNCTIONS = %i[cutter_sanborn cutter_sanborn_detailed cutter_loc cutter_loc_detailed].freeze

  def test_missing_input
    FUNCTIONS.each { |function| assert_raises(ArgumentError) { Csan.public_send(function) } }
  end

  def test_blank_input_names_the_fields
    FUNCTIONS.each do |function|
      error = assert_raises(ArgumentError) { Csan.public_send(function, surname: "   ") }
      assert_includes error.message, "surname"
    end
  end

  def test_untransliterable_name
    # A name that normalizes to nothing is an ArgumentError, not a crash.
    assert_raises(ArgumentError) { Csan.cutter_sanborn(surname: "!!!") }
  end

  def test_unknown_keyword
    # A misspelled or foreign keyword is rejected, not ignored.
    assert_raises(ArgumentError) { Csan.cutter_sanborn(given_name: "Charles") }
    assert_raises(ArgumentError) { Csan.cutter_loc(surname: "Carter", prefix_len: 2) }
  end

  def test_wrong_type
    assert_raises(TypeError) { Csan.cutter_sanborn(surname: 42) }
    assert_raises(TypeError) { Csan.cutter_loc(surname: "Carter", max_len: "4") }
  end

  def test_negative_length
    # A length is a usize: a negative one cannot be converted.
    assert_raises(RangeError) { Csan.cutter_loc(surname: "Carter", max_len: -1) }
  end
end

class TestArticles < Minitest::Test
  FUNCTIONS = TestErrors::FUNCTIONS
  ENGLISH = {lang: "en", strip_article: true}.freeze
  DICKENS = {surname: "Dickens", title: "A Christmas Carol"}.freeze

  def test_strip_article_files_the_title_without_its_article
    assert_equal "D548a", Csan.cutter_sanborn(**DICKENS)
    assert_equal "D548c", Csan.cutter_sanborn(**DICKENS, **ENGLISH)
    # Any accepted code works, in any case.
    assert_equal "D548c", Csan.cutter_sanborn(**DICKENS, lang: "EN-us", strip_article: true)
    assert_equal "T443", Csan.cutter_loc(title: "The Bear")
    assert_equal "B437", Csan.cutter_loc(title: "The Bear", **ENGLISH)
    sanborn = Csan.cutter_sanborn_detailed(title: "The Bear", **ENGLISH)
    assert_equal "Bear", sanborn.fetch(:entry)
    assert_equal "B368", sanborn.fetch(:formatted)
    assert_equal "B437", Csan.cutter_loc_detailed(title: "The Bear", **ENGLISH).fetch(:formatted)
  end

  def test_author_names_are_never_stripped
    assert_equal "L166f", Csan.cutter_sanborn(surname: "La Fontaine", title: "Les Fables", lang: "fr", strip_article: true)
  end

  def test_lang_without_strip_article_changes_nothing
    ["en", "", "  ", nil].each do |lang|
      assert_equal "D548a", Csan.cutter_sanborn(**DICKENS, lang: lang)
    end
  end

  def test_strip_article_is_read_by_truthiness
    # nil is off, and any other value but false is on, in every method.
    unstripped = {cutter_sanborn: "T375", cutter_sanborn_detailed: "T375", cutter_loc: "T443", cutter_loc_detailed: "T443"}
    stripped = {cutter_sanborn: "B368", cutter_sanborn_detailed: "B368", cutter_loc: "B437", cutter_loc_detailed: "B437"}
    FUNCTIONS.each do |function|
      formatted = ->(switch) { formatted_number(Csan.public_send(function, title: "The Bear", lang: "en", strip_article: switch)) }
      assert_equal unstripped.fetch(function), formatted.call(nil)
      [1, "yes"].each { |switch| assert_equal stripped.fetch(function), formatted.call(switch) }
    end
  end

  def test_strip_article_requires_lang
    FUNCTIONS.product([nil, "", "  "]).each do |function, lang|
      error = assert_raises(ArgumentError) { Csan.public_send(function, title: "The Bear", lang: lang, strip_article: true) }
      assert_equal "strip_article requires lang", error.message
    end
  end

  def test_unknown_lang_is_an_error_even_with_the_switch_off
    FUNCTIONS.product([false, true]).each do |function, strip_article|
      error = assert_raises(ArgumentError) { Csan.public_send(function, title: "The Bear", lang: "xx", strip_article: strip_article) }
      assert_match(/\Aunknown language "xx" \(expected one of: af, br,/, error.message)
    end
  end

  def test_language_errors_come_before_missing_input
    FUNCTIONS.each do |function|
      assert_match(/\Aunknown language/, assert_raises(ArgumentError) { Csan.public_send(function, lang: "xx") }.message)
      assert_equal "strip_article requires lang",
        assert_raises(ArgumentError) { Csan.public_send(function, strip_article: true) }.message
      assert_match(/\Aprovide at least one/,
        assert_raises(ArgumentError) { Csan.public_send(function, **ENGLISH) }.message)
    end
  end

  def test_lang_must_be_a_string
    FUNCTIONS.product([:en, 1], [false, true]).each do |function, lang, strip_article|
      assert_raises(TypeError) { Csan.public_send(function, title: "The Bear", lang: lang, strip_article: strip_article) }
    end
  end

  def test_native_arities
    # The extension takes the keywords positionally, in their order.
    assert_equal({cutter_sanborn: 7, cutter_sanborn_detailed: 7, cutter_loc: 6, cutter_loc_detailed: 6},
      FUNCTIONS.to_h { |function| [function, Csan::Native.method(function).arity] })
  end

  private

  # The number as a string, from a method's String or detailed Hash result.
  def formatted_number(result)
    result.is_a?(Hash) ? result.fetch(:formatted) : result
  end
end

class TestRunCli < Minitest::Test
  def test_success
    out, = capture_subprocess_io { assert_equal 0, Csan.run_cli(["--surname", "Dickens", "--title", "Oliver Twist"]) }
    assert_equal "D548o", out.strip
  end

  def test_method_cutter_loc
    out, = capture_subprocess_io { assert_equal 0, Csan.run_cli(["--method=cutter-loc", "--surname", "Carter"]) }
    assert_equal "C378", out.strip
  end

  def test_unusable_input_returns_one
    _, err = capture_subprocess_io { assert_equal 1, Csan.run_cli(["--surname", "   "]) }
    assert_includes err, "surname"
  end

  def test_usage_error_returns_two
    capture_subprocess_io do
      assert_equal 2, Csan.run_cli(["--method", "dewey", "--surname", "Dickens"])
      assert_equal 2, Csan.run_cli(["--cutter-sanborn", "--surname", "Dickens"])
    end
  end

  def test_help_returns_zero
    capture_subprocess_io { assert_equal 0, Csan.run_cli(["--help"]) }
  end

  def test_strip_article_with_lang
    out, = capture_subprocess_io { assert_equal 0, Csan.run_cli(["--lang", "en", "--strip-article", "--title", "The Bear"]) }
    assert_equal "B368", out.strip
    capture_subprocess_io { assert_equal 2, Csan.run_cli(["--strip-article", "--title", "The Bear"]) }
  end
end

class TestGem < Minitest::Test
  EXECUTABLE = File.expand_path("../exe/csan", __dir__)
  LIB = File.expand_path("../lib", __dir__)

  def test_versions_agree
    # The gem's version, and the extension's (the crate's): one release.
    assert_equal Csan::VERSION, Csan::Native::VERSION
  end

  def test_executable
    out, status = Open3.capture2(RbConfig.ruby, "-I", LIB, EXECUTABLE, "--surname", "Dickens", "--title", "Oliver Twist")
    assert_predicate status, :success?
    assert_equal "D548o", out.strip
  end

  def test_executable_exit_code
    _, _, status = Open3.capture3(RbConfig.ruby, "-I", LIB, EXECUTABLE, "--method", "dewey")
    assert_equal 2, status.exitstatus
  end

  # Ruby's buffered output comes first, though the CLI writes to the file
  # descriptor directly.
  def test_run_cli_after_ruby_output
    script = 'require "csan"; print "before "; Csan.run_cli(["--surname", "Carter"])'
    out, status = Open3.capture2(RbConfig.ruby, "-I", LIB, "-e", script)
    assert_predicate status, :success?
    assert_equal "before C323\n", out
  end
end
