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
