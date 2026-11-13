# HTML tree construction fixture edition

These complete snapshots derive from html5lib-tests revision
`f994590f528ac8b6073665791ddb1ed85c66dfb2`, pinned by `rcdom/html5lib-tests`.
Upstream source: <https://github.com/html5lib/html5lib-tests/tree/f994590f528ac8b6073665791ddb1ed85c66dfb2/tree-construction>.
The pinned files and submodule are unchanged.

Original fixture copyright (c) 2006–2013 James Graham, Geoffrey Sneddon and other
contributors. These snapshots and their expectation updates use the upstream MIT
licence, reproduced without changes in [LICENSE](LICENSE).

Only the `#document` sections listed below differ. Every input, description, error
section, fragment context, script state and other field remains unchanged. Indices
are zero-based, as in the original test runner. The snapshots retain all 112 cases
in `tests1.dat`, 24 cases in `html5test-com.dat` and 81 cases in
`tests_innerHTML_1.dat`. The runner's scripting-mode variants remain intact.

## Processing instruction edition

The HTML Living Standard now tokenises these instructions as processing
instructions, rather than bogus comments. EOF in an unfinished instruction emits
no instruction or comment. Rules were reviewed on 13 November 2026 against
Moegoe's local `docs/specs/html.md`.

| File | Index | Input | Changed document expectation |
| --- | ---: | --- | --- |
| tests1.dat | 39 | `<?` | Remove `<!-- ? -->`; the generated html/head/body tree is unchanged. |
| tests1.dat | 43 | `<?COMMENT?>` | Replace the bogus comment with PI target `COMMENT`, empty data. |
| tests1.dat | 46 | `<?COM--MENT?>` | Replace the bogus comment with PI target `COM--MENT`, empty data. |
| html5test-com.dat | 11 | `<?import namespace="foo" implementation="#bar">` | Replace the bogus comment with PI target `import`, literal data `namespace="foo" implementation="#bar"`. |

Normative rules:

- [Processing instruction open state](https://html.spec.whatwg.org/multipage/parsing.html#processing-instruction-open-state): EOF emits only EOF.
- [Processing instruction target state](https://html.spec.whatwg.org/multipage/parsing.html#processing-instruction-target-state): letters and hyphens are retained in the target.
- [Processing instruction data state](https://html.spec.whatwg.org/multipage/parsing.html#processing-instruction-data-state) and [questionable state](https://html.spec.whatwg.org/multipage/parsing.html#processing-instruction-questionable-state): emit the instruction on its closing delimiter.
- [Initial insertion mode](https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode) and [insert a processing instruction](https://html.spec.whatwg.org/multipage/parsing.html#insert-a-processing-instruction): insert the PI at document level before the generated root.

The tree runner serialises instructions as `<?target data?>`, including the space
before empty data. These are tree expectations, not alterations to source input.
Historical `#errors` and `#new-errors` sections remain byte-for-byte unchanged;
this edition changes document expectations only.

## Earlier select-fragment edition drift

`tests_innerHTML_1.dat`, case 75, has input `<input><option>` with fragment context
`select`. Its old document expectation includes both elements. The current
[in-body input rule](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody)
requires a parse error, ignoring the input token and returning when the fragment
context is a select. The updated document therefore contains only `<option>`.

This behaviour predates the PI change: fork commit
`3584ededbc0d8a183ec3d9339021b73b4c205173` (`fix: ignore input tokens in select
fragments`, 11 November 2026) already implements that rule. This expectation update
records existing fixture drift; it is not a PI parser regression or a parser change.

## Source resolution

The shared enumerator selects these complete files through its generic
`data/spec-updates/<original relative path>` resolution, preserving original paths,
test names, every case and all scripting variants. No cases are skipped and no
runtime case-specific expectations are applied.

On an upstream revision update, compare complete case counts and all fields.
Remove a snapshot when upstream incorporates its edition changes.
