# HTML processing instruction fixture edition

These are maintained snapshots of `tokenizer/test2.test` and `tokenizer/test3.test`
from html5lib-tests revision `f994590f528ac8b6073665791ddb1ed85c66dfb2`, the revision
pinned by `rcdom/html5lib-tests`. The original files and submodule are unchanged.
Upstream source: <https://github.com/html5lib/html5lib-tests/tree/f994590f528ac8b6073665791ddb1ed85c66dfb2/tokenizer>.

Original fixture copyright (c) 2006–2013 James Graham, Geoffrey Sneddon and other
contributors. These snapshots and their expectation updates use the upstream MIT
licence, reproduced without changes in [LICENSE](LICENSE).

Each snapshot retains every original case, description, input, state and other
field. Only the `output` and `errors` expectations of the eleven cases below change.
Indices are zero-based within the original `tests` array. `test2.test` retains 45
cases and `test3.test` retains 1,590 cases.

The edition follows the HTML Living Standard processing instruction tokenisation
rules, reviewed on 13 November 2026 against Moegoe's local `docs/specs/html.md`.
The original fixtures predate those rules and expect bogus comments instead.

| File | Index | Input | Updated output | Updated errors | HTML rule |
| --- | ---: | --- | --- | --- | --- |
| test2.test | 31 | `<?namespace>` | `ProcessingInstruction("namespace", "")` | None | Target → data → emit on `>` |
| test2.test | 32 | `<?foo-->` | `ProcessingInstruction("foo--", "")` | None | Target → data → emit on `>` |
| test3.test | 1158 | `<?` | No tokens | EOF error at line 1, column 3 | Open-state EOF |
| test3.test | 1180 | `<?A` | No tokens | EOF error at line 1, column 4 | Target-state EOF |
| test3.test | 1181 | `<?B` | No tokens | EOF error at line 1, column 4 | Target-state EOF |
| test3.test | 1182 | `<?Y` | No tokens | EOF error at line 1, column 4 | Target-state EOF |
| test3.test | 1183 | `<?Z` | No tokens | EOF error at line 1, column 4 | Target-state EOF |
| test3.test | 1185 | `<?a` | No tokens | EOF error at line 1, column 4 | Target-state EOF |
| test3.test | 1186 | `<?b` | No tokens | EOF error at line 1, column 4 | Target-state EOF |
| test3.test | 1187 | `<?y` | No tokens | EOF error at line 1, column 4 | Target-state EOF |
| test3.test | 1188 | `<?z` | No tokens | EOF error at line 1, column 4 | Target-state EOF |

All EOF errors above have code `eof-in-processing-instruction`; their column is
the input length plus one. State rules:

- [Processing instruction open](https://html.spec.whatwg.org/multipage/parsing.html#processing-instruction-open-state)
- [Processing instruction target](https://html.spec.whatwg.org/multipage/parsing.html#processing-instruction-target-state)
- [After processing instruction target](https://html.spec.whatwg.org/multipage/parsing.html#after-processing-instruction-target-state)
- [Processing instruction data](https://html.spec.whatwg.org/multipage/parsing.html#processing-instruction-data-state)

The shared test enumerator resolves each original source filename to a maintained
snapshot when one exists at `data/spec-updates/<original relative path>`. It keeps
the original test path and names, loads the complete chosen file and enumerates
every case once. There are no ignored cases or case-specific runtime overrides.

When updating the pinned upstream revision, compare complete case counts and all
fields against these snapshots. Remove a snapshot when upstream incorporates its
edition changes; do not layer further case rewrites into the runtime loader.
