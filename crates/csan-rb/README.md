# csan (Ruby gem)

Generate Cutter-Sanborn three-figure and Library of Congress cutter numbers
for library cataloging from author names and titles. The gem is the Rust
[`csan-core`](https://crates.io/crates/csan-core) library as a native
extension, with the `csan` command-line tool. Precompiled gems cover Ruby
3.3, 3.4, and 4.0 on Linux (glibc and musl), macOS, and Windows, on x86_64
and arm64 (Windows on arm64: Ruby 3.4 and 4.0); elsewhere, the gem builds
from source, with a Rust toolchain.

```sh
gem install csan       # library + CLI
bundle add csan        # in a Bundler project
```

```ruby
require "csan"

Csan.cutter_sanborn(forename: "Charles", surname: "Dickens", title: "Oliver Twist")
# => "D548o"
Csan.cutter_loc(surname: "Carter")
# => "C378"

Csan.cutter_sanborn_detailed(surname: "Borges")
# => {alpha_prefix: "B", alpha_suffix: nil, entry: "Borg", formatted: "B732", numeric: 732}
Csan.cutter_loc_detailed(surname: "Carter")
# => {alpha_prefix: "C", formatted: "C378", numeric: 378}
```

Every function takes `forename`, `surname`, and `title` keyword arguments
(at least one must be non-blank; blank strings count as absent).
`cutter_sanborn*` also take `prefix_len` and `suffix_len` (default 1);
`cutter_loc*` take `max_len` (default 3, clamped to 4). The `*_detailed`
variants return a hash of the number's parts, with symbol keys, plus the
rendered number under `:formatted`. Unusable input, or an unknown keyword,
raises `ArgumentError`; a length that is not an Integer raises `TypeError`,
and a negative one `RangeError`. [`sig/csan.rbs`](https://github.com/veralvx/csan/blob/main/crates/csan-rb/sig/csan.rbs)
holds the type signatures.

Every function also takes `lang` and `strip_article`: with a truthy
`strip_article`, the title is filed without its initial article in the
language `lang` names (an ISO 639 code, a BCP 47 tag or a POSIX locale,
case-insensitive; `csan --help` lists them all). `lang` alone changes
nothing, a blank `lang` counts as absent, and author names are never
stripped. An unknown `lang` (even with `strip_article` off), or
`strip_article` without `lang`, raises `ArgumentError`.

```ruby
Csan.cutter_sanborn(surname: "Dickens", title: "A Christmas Carol", lang: "en", strip_article: true)
# => "D548c"
Csan.cutter_loc(title: "The Bear", lang: "en", strip_article: true)
# => "B437"
```

The executable is the same interface as the standalone Rust binary
(options, help, messages, and exit codes):

```sh
csan --forename Charles --surname Dickens --title "Oliver Twist"   # D548o
csan --method cutter-loc --surname Carter                         # C378
```

## Building from source

Built with [rb-sys](https://github.com/oxidize-rb/rb-sys) from
[`crates/csan-rb`](https://github.com/veralvx/csan/tree/main/crates/csan-rb)
in the main repository (Rust toolchain and libclang required):

```sh
cd crates/csan-rb
bundle install
bundle exec rake compile test   # the extension, then the test suite
bundle exec rake source_gem     # pkg/csan-<version>.gem
```

Each [GitHub release](https://github.com/veralvx/csan/releases) carries the
platform gems and the source gem that are published to RubyGems.

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/veralvx/csan/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/veralvx/csan/blob/main/LICENSE-MIT)
at your option.
