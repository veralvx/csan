# csan

`csan` generates Cutter-Sanborn and Library of Congress cutter numbers for
library cataloging from author names and titles. It is a command-line tool
and a library for Rust, Python, Ruby, JavaScript (npm), and Typst.

```console
$ csan --forename Charles --surname Dickens --title "Oliver Twist"
D548o
$ csan --method cutter-loc --forename Charles --surname Dickens
D535
```

It implements two methods:

- **Cutter-Sanborn three-figure:** a lookup against the bundled
  Cutter-Sanborn table (12,000+ entries), with an optional call number
  prefix and a lowercase work mark derived from the title.
  Its table comes from https://github.com/veralvx/cutter-sanborn-table
- **Library of Congress:** the LoC character-branching rules
  ([Cutter table](https://www.loc.gov/aba/pcc/053/table.html),
  [CSM G 63](https://www.loc.gov/aba/publications/FreeCSM/G063.pdf)),
  producing an initial plus a digit sequence.

Input names are normalized first: whitespace and punctuation are stripped,
diacritics and ligatures are folded to ASCII (`Čapek` → `Capek`, `Strauß` →
`Strauss`), `Mc`/`M'` prefixes expand to `Mac`, and a surname of
`St`/`Ste` (as in Saint Patrick) expands to `Saint`/`Sainte`, matching the
conventions of the underlying tables.
Anonymous works are cuttered by title. On request, titles are filed without
their initial article (_The Bear_ under _Bear_), with 25 languages
supported: see [Initial articles](#initial-articles).

## Install

- **Command-line tool:** `cargo install csan`, or prebuilt with
  `cargo binstall csan` or the installers and archives on each
  [GitHub release](https://github.com/veralvx/csan/releases). With Nix
  (Linux, aarch64 macOS): `nix run github:veralvx/csan -- --help`. Without
  installing: `npx @veralvx/csan --help`.
- **Rust library:** `cargo add csan-core`.
- **Python:** `pip install csan` (library and CLI), or
  `uv tool install csan` for the CLI alone.
- **Ruby:** `gem install csan` (library and CLI).
- **JavaScript (Node):** `npm install @veralvx/csan` (library and CLI).
- **Typst:** see [Typst](#typst).

## Command line

```sh
csan --surname Dickens --title "Oliver Twist"            # D548o
csan --method cutter-loc --surname Carter                # C378
csan --method cutter-loc --title "100 best recipes"      # A153
csan --surname Borges --json                             # {"cutter_sanborn": {"formatted": "B732", ...}}
csan --lang en --strip-article --title "The Bear"        # B368 (T375 without)
```

`--method` selects the method, `cutter-sanborn` (the default) or
`cutter-loc`, and at least one non-blank input (`--forename`, `--surname`,
`--title`) is required. `--prefix-len`/`--suffix-len` tune the
Cutter-Sanborn output, `--max-len` the LoC digits, and `--json` emits the
parts as a JSON object keyed by method (`cutter_sanborn` or
`cutter_loc`). `--strip-article` disregards the title's initial article in
the language that `--lang` names (`en`, `pt-BR`, …; `csan --help` lists
them all; see [Initial articles](#initial-articles)). Errors exit non-zero
with a message on stderr.

With [Nix](https://nixos.org), the flake builds the binary with the
toolchain that `rust-toolchain.toml` pins:

```sh
nix run github:veralvx/csan -- --help
nix build github:veralvx/csan     # ./result/bin/csan
nix profile install github:veralvx/csan
```

## Rust

The library is the [`csan-core`](https://crates.io/crates/csan-core) crate
(up to 0.5.x it was published as `csan`, a name that now belongs to the
command-line tool):

```rust
use csan_core::{
    Attribution, Author, CutterConfig, CutterGenerator, CutterLocMethod,
    CutterSanbornMethod, Opus,
};

fn main() -> Result<(), csan_core::NormalizationError> {
    let entry = Attribution::Both {
        author: Author::new("Charles", "Dickens"),
        opus: Opus::new("Oliver Twist"),
    };

    let sanborn = CutterSanbornMethod::generate(&entry, CutterConfig::default())?;
    assert_eq!(sanborn.to_string(), "D548o");

    let loc = CutterLocMethod::generate(&entry, CutterConfig::default())?;
    assert_eq!(loc.to_string(), "D535");
    Ok(())
}
```

`CutterConfig` tunes the output: `prefix_len` and `suffix_len` control the
Cutter-Sanborn prefix and work mark, and `max_len` controls the LoC digit
count (clamped to 4). Works without an author use
`Attribution::OpusOnly(...)` and are cuttered by title, including the LoC
numeral rule (`"100 best recipes"` → `A153`). A title is filed without its
initial article with `with_article_language` on the `Opus` or the
`Attribution` (see [Initial articles](#initial-articles)):

```rust
use csan_core::{
    Attribution, CutterConfig, CutterGenerator, CutterLocMethod, Language, Opus,
};

fn main() -> Result<(), csan_core::NormalizationError> {
    let entry = Attribution::OpusOnly(Opus::new("The Bear"))
        .with_article_language(Some(Language::English));
    let loc = CutterLocMethod::generate(&entry, CutterConfig::default())?;
    assert_eq!(loc.to_string(), "B437"); // T443 without
    Ok(())
}
```

The API reference is on [docs.rs](https://docs.rs/csan-core).

## Python

The PyPI package [`csan`](https://pypi.org/project/csan/) (CPython 3.9+,
abi3 wheels) provides the library and the `csan` command:

```python
import csan

csan.cutter_sanborn(surname="Dickens", title="Oliver Twist")  # 'D548o'
csan.cutter_loc_detailed(surname="Carter")
# {'alpha_prefix': 'C', 'formatted': 'C378', 'numeric': 378}
csan.cutter_loc(title="The Bear", lang="en", strip_article=True)  # 'B437'
```

See the [Python package README](https://github.com/veralvx/csan/blob/main/crates/csan-py/README.md)
for the full API.

## Ruby

The gem [`csan`](https://rubygems.org/gems/csan) (Ruby 3.3+, precompiled
for Linux, macOS, and Windows; elsewhere it builds from source with a Rust
toolchain) provides the library and the `csan` command:

```ruby
require "csan"

Csan.cutter_sanborn(surname: "Dickens", title: "Oliver Twist") # => "D548o"
Csan.cutter_loc_detailed(surname: "Carter")
# => {alpha_prefix: "C", formatted: "C378", numeric: 378}
Csan.cutter_loc(title: "The Bear", lang: "en", strip_article: true) # => "B437"
```

See the [gem README](https://github.com/veralvx/csan/blob/main/crates/csan-rb/README.md)
for the full API.

## JavaScript

The npm package [`@veralvx/csan`](https://www.npmjs.com/package/@veralvx/csan)
is the library compiled to WebAssembly, built for Node (CommonJS, with
TypeScript declarations):

```js
const { cutterLoc, cutterSanborn } = require("@veralvx/csan");

cutterSanborn({ surname: "Dickens", title: "Oliver Twist" }); // "D548o"
cutterLoc({ surname: "Carter" }); // "C378"
cutterLoc({ title: "The Bear", lang: "en", stripArticle: true }); // "B437"
```

It also provides the `csan` command, with the same interface as the native
binary. See the [npm package README](https://github.com/veralvx/csan/blob/main/crates/csan-web/README.md)
for the full API and the browser build.

## Typst

The Typst package exposes two functions:

```typ
// Replace <version> with the installed release.
#import "@local/csan:<version>": cutter-sanborn, cutter-loc

#cutter-sanborn(
  forename: "Charles",
  surname: "Dickens",
  title: "Oliver Twist",
) // -> "D548o"

#cutter-loc(surname: "Carter") // -> "C378"

#cutter-loc(title: "The Bear", lang: "en", strip-article: true) // -> "B437"
```

Both accept `forename`, `surname`, and `title` (at least one must be
non-blank; blank strings count as absent). `cutter-sanborn` also takes
`prefix-len` and `suffix-len`; `cutter-loc` takes `max-len`. Both take
the CLI's `--lang` and `--strip-article` as `lang` and `strip-article`:
with `strip-article: true`, the title is filed without its initial article
in the language that `lang` names (see
[Initial articles](#initial-articles)). Pass `detailed: true` to receive a
dictionary of the parts (`alpha_prefix`, `numeric`, …) instead of the
formatted string. Requires Typst 0.14.2 or later.

Until the package is published to Typst Universe, install it as a
[local package](https://github.com/typst/packages#local-packages): extract
the `csan_typst-<version>.tar.gz` attached to each
[GitHub release](https://github.com/veralvx/csan/releases) into
`{data-dir}/typst/packages/local/csan/<version>/`, then import
`@local/csan:<version>`. The plugin binary alone is attached as
`csan_typst.wasm`.

## Initial articles

Library filing ignores a title's initial article. Cutter's _Rules for a
Dictionary Catalogue_ (§241) say to "take account of every word except
initial articles", and LC's shelflisting manual
([G 100](https://www.loc.gov/aba/publications/FreeCSM/G100.pdf) §9) says to
"ignore initial articles in the nominative case". So Dickens's _A Christmas
Carol_ gets the work mark `c`, not `a`, and a work without an author is
cuttered by the word after its article, in both methods.

csan does this on request, for the language the title is in, because a word
that is an article in one language is an ordinary word in another (English
_Die Hard_). Every frontend takes the language and a switch:

| Frontend   | Language and switch                               |
| ---------- | ------------------------------------------------- |
| CLI        | `--lang en --strip-article`                       |
| Rust       | `.with_article_language(Some(Language::English))` |
| Python     | `lang="en", strip_article=True`                   |
| Ruby       | `lang: "en", strip_article: true`                 |
| JavaScript | `{ lang: "en", stripArticle: true }`              |
| Typst      | `lang: "en", strip-article: true`                 |

```console
$ csan --surname Dickens --title "A Christmas Carol"
D548a
$ csan --lang en --strip-article --surname Dickens --title "A Christmas Carol"
D548c
```

The language alone changes nothing, so it can be set once (a shell alias,
Python's `functools.partial`, Typst's `.with`). An unknown code is an error,
even with the switch off, and so is the switch without a language. In the
libraries a blank language counts as absent, as blank names and titles do;
the CLI rejects an empty or blank `--lang`, a flag given without its value. The
language is named by an ISO 639 code, a BCP 47 tag or a POSIX locale, in any
case (`en`, `EN`, `eng`, `pt-BR`, `ca@valencia`, `pt_BR.UTF-8`).

**Author names are never stripped.** G 100 §10 treats "initial articles that
form a part of place names and personal names as an integral part of the
heading", so _La Fontaine_ stays under L (`L166f` for _Les Fables_, in
French). Where a rule enters a name under the word after its prefix
(Cutter's §24c: _Gama, Vasco da_), the heading already reflects it, and csan
takes the heading as given.

The article lists start from MARC 21's
[Appendix F](https://www.loc.gov/marc/bibliographic/bdapndxf.html), keeping
only the nominative forms that G 100 §9 asks for, so German _dem_ and _des_
are filed on. Other sources fill its gaps: Breton from the list the JSC
approved for RDA in 2006, Luxembourgish from its grammar's nominative forms
as the official orthography spells them (_d'_, and _de_ and _e_ by its
n-rule), and standard forms that fill a gap in a paradigm Appendix F
otherwise gives (Galician _os_ beside _as_, _un_ beside _unha_; Provençal
_lei_, _leis_; the assimilated Maltese forms; Irish without the hyphen).
Left out are forms that G 100 or the grammars show aren't initial articles
(Catalan's personal _en_, Romanian's genitival _a_ and _al_, and Albanian's
_njē_, as G 100 §9 lists Albanian as having no initial articles), forms with
no grammatical basis (Icelandic _'r_), and archaic or dialect forms that are
mostly other words (English _ye_, German _'s_, usually the pronoun _es_).
Appendix F gives no plural indefinite for Spanish, Portuguese, Catalan,
Galician or French, only for Occitan, so _unos_, _uns_ and the like aren't
added.

Numerals that are also articles (Hungarian _egy_, Swedish _en_) are
stripped, since G 100 makes "no attempt … to differentiate cardinal numbers
from articles". Latin and Albanian, which G 100 §9 lists as having no initial
articles, have none: their codes are accepted, and nothing is stripped.
Matching ignores case but not diacritics, and an apostrophe matches its
typographic forms (_L’Étranger_). A word article must be followed by a
space (_The, Bear_ keeps its _The_); after that, everything up to the next
letter or ASCII digit goes with it (_The “Bear”_ files under _Bear_), and
only one article is removed.

| Language             | Codes                                          | Articles                                                                                                                                 |
| -------------------- | ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Afrikaans            | `af`, `afr`                                    | `die`, `'n`                                                                                                                              |
| Albanian             | `sq`, `sqi`, `alb`, `als`, `aln`, `aae`, `aat` | none                                                                                                                                     |
| Breton               | `br`, `bre`                                    | `an`, `al`, `ar`, `un`, `ul`, `ur`, `eun`, `eul`, `eur`                                                                                  |
| Catalan (Valencian)  | `ca`, `cat`                                    | `el`, `els`, `la`, `les`, `l'`, `un`, `una`                                                                                              |
| Danish               | `da`, `dan`                                    | `de`, `den`, `det`, `en`, `et`                                                                                                           |
| Dutch (Flemish)      | `nl`, `nld`, `dut`                             | `de`, `het`, `een`, `eene`, `'t`, `'n`                                                                                                   |
| English              | `en`, `eng`                                    | `a`, `an`, `the`                                                                                                                         |
| Esperanto            | `eo`, `epo`                                    | `la`                                                                                                                                     |
| French               | `fr`, `fra`, `fre`                             | `le`, `la`, `les`, `l'`, `un`, `une`                                                                                                     |
| Galician             | `gl`, `glg`                                    | `o`, `a`, `os`, `as`, `un`, `unha`                                                                                                       |
| German               | `de`, `deu`, `ger`                             | `der`, `die`, `das`, `ein`, `eine`                                                                                                       |
| Hungarian            | `hu`, `hun`                                    | `a`, `az`, `egy`                                                                                                                         |
| Icelandic            | `is`, `isl`, `ice`                             | `hinn`, `hin`, `hið`, `hinir`, `hinar`                                                                                                   |
| Irish                | `ga`, `gle`                                    | `an`, `na`, and the `t`/`h` prefix after them (_An tOileánach_, _na hÉireann_)                                                           |
| Italian              | `it`, `ita`                                    | `il`, `lo`, `la`, `l'`, `i`, `gli`, `gl'`, `le`, `un`, `uno`, `una`, `un'`                                                               |
| Latin                | `la`, `lat`                                    | none                                                                                                                                     |
| Luxembourgish        | `lb`, `ltz`                                    | `den`, `de`, `d'`, `deen`, `dat`, `déi`, `en`, `e`, `eng`                                                                                |
| Maltese              | `mt`, `mlt`                                    | `il-`, `l-`, `iċ-`, `id-`, `in-`, `ir-`, `is-`, `it-`, `ix-`, `iż-`, `iz-`, `ic-`                                                        |
| Norwegian            | `no`, `nor`, `nb`, `nob`, `nn`, `nno`          | `de`, `dei`, `den`, `det`, `e`, `ei`, `ein`, `eit`, `en`, `et`                                                                           |
| Occitan (Provençal)  | `oc`, `oci`, `pro`                             | `il`, `l'`, `la`, `las`, `le`, `les`, `lh`, `lhi`, `li`, `lis`, `lo`, `los`, `lou`, `lu`, `lei`, `leis`, `un`, `una`, `uno`, `uns`, `us` |
| Portuguese           | `pt`, `por`                                    | `o`, `a`, `os`, `as`, `um`, `uma`                                                                                                        |
| Romanian (Moldavian) | `ro`, `ron`, `rum`, `mo`, `mol`                | `un`, `o`                                                                                                                                |
| Spanish              | `es`, `spa`                                    | `el`, `la`, `los`, `las`, `lo`, `un`, `una`                                                                                              |
| Swedish              | `sv`, `swe`                                    | `de`, `den`, `det`, `en`, `ett`                                                                                                          |
| Welsh                | `cy`, `cym`, `wel`                             | `y`, `yr`                                                                                                                                |

Some titles start with a word that matches an article but is filed on, which
is why stripping is off by default: a letter used as a word (_A to Z drug
facts_), a pronoun (Spanish _Lo dijo_, German _Das ist…_), an article that is
part of a name (_El Cid_, _Los Angeles_), the Irish interrogative _An_, or an
Irish loanword that begins with `t` or `h` (_An tsunami_ files under
_sunami_). The `catalog::language` module documentation on
[docs.rs](https://docs.rs/csan-core) gives the details, with the false
negatives.

## Expected values

Generated by `cargo run --example showcase` with the default configuration:

| Author              | Cutter-Sanborn | Library of Congress |
| ------------------- | -------------- | ------------------- |
| Jane Austen         | `A933`         | `A978`              |
| Eric Blair          | `B635`         | `B535`              |
| Agatha Christie     | `C555`         | `C475`              |
| Samuel Clemens      | `C625`         | `C546`              |
| Jorge De la Cruz    | `D332`         | `D453`              |
| Charles Dickens     | `D548`         | `D535`              |
| Emily Dickinson     | `D553`         | `D535`              |
| Fyodor Dostoyevsky  | `D724`         | `D678`              |
| Stephen King        | `K54`          | `K564`              |
| VERA LVX            | `L979`         | `L89`               |
| Herman Melville     | `M531`         | `M458`              |
| George Orwell       | `O79`          | `O794`              |
| William Shakespeare | `S527`         | `S535`              |
| Lord Sith           | `S623`         | `S584`              |
| Ivan Smith          | `S649`         | `S658`              |
| William Smith       | `S664`         | `S658`              |
| Leo Tolstoy         | `T654`         | `T657`              |
| Mark Twain          | `T969`         | `T835`              |
| Virginia Woolf      | `W913`         | `W665`              |
| Emile Zola          | `Z86`          | `Z653`              |

## Contributing

| Path                    | Contents                                                    |
| ----------------------- | ----------------------------------------------------------- |
| `crates/csan-core`      | The library: normalization, models, generators, the table.  |
| `crates/csan`           | The `csan` command-line tool (clap).                        |
| `crates/csan-py`        | Python bindings and the PyPI package (pyo3 + maturin).      |
| `crates/csan-rb`        | Ruby bindings and the gem (magnus + rb-sys).                |
| `crates/csan-web`       | JavaScript/npm bindings via WebAssembly (wasm-bindgen).     |
| `crates/csan-typst`     | The Typst WASM plugin, and the package's `just` harness.    |
| `crates/csan-typst/pkg` | The Typst package (`typst.toml`, `lib.typ`, plugin binary). |

[CONTRIBUTING.md](https://github.com/veralvx/csan/blob/main/CONTRIBUTING.md)
covers the development setup, the checks CI runs, building the Typst
plugin, and the release process. Changes are listed in the
[changelog](https://github.com/veralvx/csan/blob/main/CHANGELOG.md).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/veralvx/csan/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/veralvx/csan/blob/main/LICENSE-MIT)
at your option.
