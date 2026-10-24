# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-10-24

### Added

- Add a harness for the Typst package

### Changed

- Update README
- Rewrite the package README for Typst Universe
- Import the package by its @preview spec in the example
- Align the README's table as dprint formats it
- Merge pull request #20 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Merge pull request #21 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Name the author's parts forename and surname
- Merge pull request #22 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Publish csan as a Ruby gem
- Accept rb_sys's licence in dependency review
- Merge pull request #23 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Merge pull request #24 from veralvx/claude/youthful-brahmagupta-6rcrfr

### Fixed

- Qualify the harness's recipes after the first from the root
- Drop the apostrophes from the gem bumps' messages

## [0.7.1] - 2026-10-22

### Changed

- Bump actions/setup-node from 6 to 7
- Merge pull request #14 from veralvx/dependabot/github_actions/actions/setup-node-7
- Run the required checks on every pull request
- Merge pull request #19 from veralvx/claude/youthful-brahmagupta-6rcrfr

## [0.7.0] - 2026-10-22

### Added

- Add the csan command to the npm package
- Add a flake for the csan binary
- Support cargo binstall

### Changed

- Merge pull request #9 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Select the method with --method, defaulting to cutter-sanborn
- Say which npm token the publish fallback needs
- Merge pull request #10 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Merge pull request #11 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Update flake.lock weekly with Dependabot and auto-merge it
- Merge pull request #12 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Merge pull request #16 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Rework the READMEs for their registry pages
- Correct normalization, JSON and Python output in the READMEs
- Wrap the npm README's require as dprint formats it
- Merge pull request #17 from veralvx/claude/youthful-brahmagupta-6rcrfr
- Remove version badges from README.md
- Update dist to 0.33.1 and keep Dependabot off release.yml
- Merge pull request #18 from veralvx/claude/youthful-brahmagupta-6rcrfr

### Fixed

- Update tokio to 1.54.1 (RUSTSEC-2026-0384)
- Publish to npm and crates.io from the tag push
- Match the native binary in the npm command's colors and edge cases
- Optional options in the typings, unused-stream parity, tty check
- Pin and separate the type check; widen the tty parity matrix

## [0.6.1] - 2026-10-22

### Changed

- Name the Node package @veralvx/csan
- Publish @veralvx/csan to npm after each successful release
- Publish csan-core and csan to crates.io after each release
- Pin the npm major used for publishing
- Merge pull request #8 from veralvx/claude/youthful-brahmagupta-6rcrfr

## [0.6.0] - 2026-10-22

### Changed

- Apply fmt
- Rename csan to csan-core and csan-cli to csan
- Attach Typst, Python, and npm artifacts to releases
- Publish to PyPI after each successful release
- Describe the renamed crates, release artifacts, and publishing
- Run the extra-artifacts build when its workflow changes
- Skip the standalone extra-artifacts run for tag pushes
- Declare rust-version, use snake_case lint names, ignore bytecode
- Harden PyPI publishing and the extra-artifacts build
- Run the standalone extra-artifacts build on branch pushes
- Order the rust-version keys as dprint formats them
- Merge pull request #7 from veralvx/claude/youthful-brahmagupta-6rcrfr

### Fixed

- Accept forename/surname and reject unknown option keys
- Check __version__ against the installed metadata
- Sync the package version and rebuild the plugin
- Bump the versions that exist in just release
- Run the npm checks through just and fix dependabot auto-merge
- Prepend the new changelog section instead of regenerating
- Refuse to cut a release with no unreleased commits
- Stop when a version edit fails
- Refuse a bump too small for breaking changes
- Reject non-object options with a clear error
- Stop doubling the Error prefix in option errors

## [0.5.10] - 2026-10-22

### Changed

- Rename given-name and family-name
- Rename given_ and family_ prefixes
- Ignore .ropeproject
- Apply fmt
- Move csan crate declaration to root Cargo.toml

### Fixed

- Override env for rustup toolchain
- Rename wasm file for typst

## [0.5.9] - 2026-10-21

### Changed

- Bump to 0.5.9
- Update CHANGELOG

### Removed

- Remove musl

## [0.5.8] - 2026-10-21

### Changed

- Bump to 0.5.8
- Bump lock to 0.5.8

## [0.5.7] - 2026-10-21

### Changed

- Bump version
- Bump version
- Bump version
- Format with dprint
- V0.5.7

## [0.5.6] - 2026-10-21

### Changed

- V0.5.6

### Fixed

- Correct members for release

## [0.5.5] - 2026-10-21

### Changed

- Update dprint
- Update cargo versions
- Update csan-cli version
- V0.5.5

### Fixed

- Ignore crates for release

## [0.5.4] - 2026-10-21

### Changed

- Format with dprint
- V0.5.4

### Fixed

- Specify member for release

## [0.5.3] - 2026-10-21

### Changed

- V0.5.3

### Removed

- Remove msi

## [0.5.2] - 2026-10-21

### Added

- Add release workflow

### Changed

- V0.5.2

## [0.5.1] - 2026-10-21

### Changed

- Compliant dependabot
- Format with dprint
- Format with dprint
- Bump actions/setup-python from 5 to 7
- Merge pull request #6 from veralvx/dependabot/github_actions/actions/setup-python-7
- Bump dprint/check
- Merge pull request #5 from veralvx/dependabot/github_actions/actions-dependencies-456eeecc7b
- Merge branch 'main' of https://github.com/veralvx/csan
- Toml versions
- V0.5.1

### Fixed

- Extra space in catergory
- Rectify category slugs

## [0.5.0] - 2026-10-21

### Changed

- Rewrite in rust
- V0.5.0

## [0.4.2] - 2025-12-08

### Changed

- Early returns in cutter function main loop
- Use absolute imports
- Update gitignore
- Bump version

### Fixed

- Henry, g. typo in table

## [0.4.1] - 2025-12-05

### Added

- Add exports to __init__.py
- Add license information to README

### Changed

- Correct git cliff recipe in justfile
- Improve justfile recipes
- Replace commands description for a table
- Include library installation instruction in README
- Correct description for release recipe
- Update CONTRIBUTING
- Update .gitignore
- Update release and clean recipes for Justfile
- Bump version
- Update CHANGELOG

### Fixed

- Fix typos and remove extra spaces in README

### Removed

- Remove trailing whitespaces in Justfile

## [0.4.0] - 2025-12-03

### Added

- Add 'console' language to pre element
- Add edge case

### Changed

- Update README
- Build descending name list from abbreviated composed name
- Rename data file with cutter table
- Update README
- Bump version
- Update README with historic context
- Update CHANGELOG

### Removed

- Remove exports

## [0.3.0] - 2025-12-03

### Added

- Add contribution instructions
- Add aliases for  justfile recipes
- Add edge cases

### Changed

- Reorder justfile recipes
- Create a tuple from cutter keys instead of a list
- Allow single character last name
- Allow single character last name
- Bump pyrefly
- Ignore 'Too many statements' ruff rule
- Update README
- Exclude log files in .gitignore
- Bump version
- Update CHANGELOG

### Fixed

- Handle first two letters absent from the table
- Broad rewrite and handle edge cases

### Removed

- Remove debugging entry

## [0.2.0] - 2025-12-02

### Added

- Add justfile recipe for dev-deps
- Add envs to gitignore
- Add more cases to cutter test
- Add autoformat in check recipe
- Add missing command word to check recipe

### Changed

- Bump uv and checkout actions
- Change CHANGELOG format
- Rename commit message in justfile
- Improve justfile documentation
- Update pyrefly
- Bump deps
- Use bisection for sieving cutter keys
- Make first name optional
- Update README
- Bump version
- Update CHANGELOG
- Update CHANGELOG

### Fixed

- Clean recipe does not alter .venv
- Include uv sync in check recipe
- Logging config only in main file

### Removed

- Remove underscores from functions in main file
- Remove f-string variable from raised error

## [0.1.1] - 2025-11-24

### Added

- Add CHANGELOG
- Add basic case

### Changed

- Initial commit
- Update boilerplate files
- Rename args function
- Update README
- Update README
- Update gitignore
- Bump version
- Bump version

### Fixed

- Change error type if cutter function fails
- Initial tests
- Passing checks for release

[0.5.10]: https://github.com/veralvx/csan/compare/v0.5.9..0.5.10
[0.5.9]: https://github.com/veralvx/csan/compare/v0.5.8..v0.5.9
[0.5.8]: https://github.com/veralvx/csan/compare/v0.5.7..v0.5.8
[0.5.7]: https://github.com/veralvx/csan/compare/v0.5.6..v0.5.7
[0.5.6]: https://github.com/veralvx/csan/compare/v0.5.5..v0.5.6
[0.5.5]: https://github.com/veralvx/csan/compare/v0.5.4..v0.5.5
[0.5.4]: https://github.com/veralvx/csan/compare/v0.5.3..v0.5.4
[0.5.3]: https://github.com/veralvx/csan/compare/v0.5.2..v0.5.3
[0.5.2]: https://github.com/veralvx/csan/compare/v0.5.1..v0.5.2
[0.5.1]: https://github.com/veralvx/csan/compare/v0.5.0..v0.5.1
[0.5.0]: https://github.com/veralvx/csan/compare/v0.4.2..v0.5.0
[0.4.2]: https://github.com/veralvx/csan/compare/v0.4.1..v0.4.2
[0.4.1]: https://github.com/veralvx/csan/compare/v0.4.0..v0.4.1
[0.4.0]: https://github.com/veralvx/csan/compare/v0.3.0..v0.4.0
[0.3.0]: https://github.com/veralvx/csan/compare/v0.2.0..v0.3.0
[0.2.0]: https://github.com/veralvx/csan/compare/v0.1.1..v0.2.0
[0.1.1]: https://github.com/veralvx/csan/compare/v0.1.0..v0.1.1

<!-- generated by git-cliff -->
