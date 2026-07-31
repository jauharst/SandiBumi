# Third-party licences

SandiBumi is distributed as a compiled desktop application that statically links a large number
of open-source Rust crates and bundles a JavaScript frontend. This file lists them and their
declared licences.

**Generated** by `tools/gen-third-party-licenses.mjs` — re-run it after any dependency change;
do not edit by hand. It is a **factual inventory, not legal advice.**

Scope note: only **normal** (distributed) dependencies are listed. Build-time and dev-only
packages — the compiler plugins, the bundler, the test harnesses — are not shipped to a user and
are excluded, so the obligations that DO apply stay visible.

Python packages (`numpy`, `dlisio`, `scikit-learn`, `xlsxwriter`, `python-docx`,
`python-pptx`, `matplotlib`, `Pillow`) are **not distributed with SandiBumi**. They are
prerequisites the user installs into their own interpreter, which SandiBumi invokes as a
subprocess. That is a materially lighter obligation than bundling them.

## Attention items

**Weak-copyleft licences present (6).** All are file-level (MPL-family): they
permit linking into a proprietary application, but require that the source of *those files*
remains available and that the licence notice is preserved. None is modified by this project;
all arrive transitively.

- smartstring v1.0.1 — MPL-2.0+
- option-ext v0.2.0 — MPL-2.0
- cssparser v0.36.0 — MPL-2.0
- cssparser-macros v0.6.1 — MPL-2.0
- dtoa-short v0.3.5 — MPL-2.0
- selectors v0.36.1 — MPL-2.0

**No package in the distributed set is missing a licence declaration.**

---

## Rust crates

289 packages.

| Licence | Count |
|---|---|
| MIT OR Apache-2.0 | 135 |
| MIT | 47 |
| Apache-2.0 OR MIT | 25 |
| MIT/Apache-2.0 | 18 |
| Unicode-3.0 | 18 |
| Apache-2.0 | 11 |
| MPL-2.0 | 5 |
| Unlicense OR MIT | 4 |
| Unlicense/MIT | 4 |
| Zlib | 3 |
| BSD-3-Clause | 2 |
| Apache-2.0 AND MIT | 2 |
| Zlib OR Apache-2.0 OR MIT | 2 |
| BSD-2-Clause OR Apache-2.0 OR MIT | 2 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| BSD-3-Clause AND MIT | 1 |
| BSD-3-Clause/MIT | 1 |
| CC0-1.0 OR MIT-0 OR Apache-2.0 | 1 |
| Apache-2.0 / MIT | 1 |
| MIT OR Zlib OR Apache-2.0 | 1 |
| MIT OR Apache-2.0 OR Zlib | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| MPL-2.0+ | 1 |
| CC0-1.0 | 1 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 1 |

### MIT OR Apache-2.0

- ahash v0.8.12
- anyhow v1.0.103
- arrayvec v0.7.8
- base64 v0.22.1
- bitflags v2.13.0
- block-buffer v0.10.4
- camino v1.2.4
- cargo-platform v0.1.9
- cast v0.3.0
- cfg-if v1.0.4
- chrono v0.4.45
- const-random v0.1.18
- const-random-macro v0.1.16
- cookie v0.18.1
- cpufeatures v0.2.17
- crc32fast v1.5.0
- crossbeam-channel v0.5.16
- crossbeam-deque v0.8.7
- crossbeam-epoch v0.9.20
- crossbeam-utils v0.8.22
- crypto-common v0.1.7
- deranged v0.5.8
- digest v0.10.7
- dirs v6.0.0
- dirs-sys v0.5.0
- displaydoc v0.2.6
- dtoa v1.0.11
- dyn-clone v1.0.20
- either v1.16.0
- erased-serde v0.4.10
- fdeflate v0.3.7
- flate2 v1.1.9
- form_urlencoded v1.2.2
- getrandom v0.2.17
- getrandom v0.3.4
- getrandom v0.4.3
- glob v0.3.3
- half v2.7.1
- hashbrown v0.12.3
- hashbrown v0.15.5
- hashbrown v0.17.1
- hashlink v0.10.0
- heck v0.5.0
- html5ever v0.38.0
- http v1.4.2
- idna v1.1.0
- itoa v1.0.18
- jsonptr v0.6.3
- keyboard-types v0.7.0
- libc v0.2.186
- lock_api v0.4.14
- log v0.4.33
- markup5ever v0.38.0
- mime v0.3.17
- num-bigint v0.4.8
- num-complex v0.4.6
- num-conv v0.2.2
- num-integer v0.1.46
- num-traits v0.2.19
- once_cell v1.21.4
- parking_lot v0.12.5
- parking_lot_core v0.9.12
- percent-encoding v2.3.2
- png v0.17.16
- powerfmt v0.2.0
- proc-macro2 v1.0.106
- quote v1.0.46
- rayon v1.12.0
- rayon-core v1.13.0
- regex v1.13.0
- regex-automata v0.4.15
- regex-syntax v0.8.11
- rhai v1.25.1
- rhai_codegen v3.2.0
- scopeguard v1.2.0
- semver v1.0.28
- serde v1.0.228
- serde-untagged v0.1.9
- serde_core v1.0.228
- serde_derive v1.0.228
- serde_derive_internals v0.29.1
- serde_json v1.0.150
- serde_repr v0.1.20
- serde_spanned v1.1.1
- serde_with v3.21.0
- serde_with_macros v3.21.0
- serialize-to-javascript v0.1.2
- serialize-to-javascript-impl v0.1.2
- servo_arc v0.4.3
- sha2 v0.10.9
- smallvec v1.15.2
- softbuffer v0.4.8
- stable_deref_trait v1.2.1
- static_assertions v1.1.0
- string_cache v0.9.0
- syn v2.0.119
- tendril v0.5.1
- thin-vec v0.2.18
- thiserror v1.0.69
- thiserror v2.0.18
- thiserror-impl v1.0.69
- thiserror-impl v2.0.18
- time v0.3.53
- time-core v0.1.9
- time-macros v0.2.31
- toml v1.1.3+spec-1.1.0
- toml_datetime v1.1.1+spec-1.1.0
- toml_parser v1.1.2+spec-1.1.0
- toml_writer v1.1.2+spec-1.1.0
- typeid v1.0.3
- typenum v1.20.1
- unicode-segmentation v1.13.3
- unicode-width v0.2.2
- url v2.5.8
- web_atoms v0.2.5
- windows v0.61.3
- windows-collections v0.2.0
- windows-core v0.61.2
- windows-future v0.2.1
- windows-implement v0.60.2
- windows-interface v0.59.3
- windows-link v0.1.3
- windows-link v0.2.1
- windows-numerics v0.2.0
- windows-result v0.3.4
- windows-strings v0.4.2
- windows-sys v0.59.0
- windows-sys v0.60.2
- windows-sys v0.61.2
- windows-targets v0.52.6
- windows-targets v0.53.5
- windows-threading v0.1.0
- windows-version v0.1.7
- windows_x86_64_msvc v0.52.6
- windows_x86_64_msvc v0.53.1

### MIT

- atoi v2.0.0
- bytes v1.12.1
- cargo_metadata v0.19.2
- cfb v0.7.3
- comfy-table v7.1.4
- crossterm v0.28.1
- crossterm_winapi v0.9.1
- crunchy v0.2.4
- darling v0.23.0
- darling_core v0.23.0
- darling_macro v0.23.0
- derive_more v2.1.1
- derive_more-impl v2.1.1
- dom_query v0.27.0
- duckdb v1.10504.0
- generic-array v0.14.7
- ico v0.5.0
- infer v0.19.0
- libduckdb-sys v1.10504.0
- libm v0.2.16
- new_debug_unreachable v1.0.6
- phf v0.13.1
- phf_generator v0.13.1
- phf_macros v0.13.1
- phf_shared v0.13.1
- plist v1.10.0
- precomputed-hash v0.1.1
- quick-xml v0.41.0
- rfd v0.16.0
- rust_decimal v1.42.1
- schemars v0.8.22
- schemars_derive v0.8.22
- simd-adler32 v0.3.10
- strsim v0.11.1
- strum v0.27.2
- strum_macros v0.27.2
- synstructure v0.13.2
- tokio v1.52.3
- tokio-macros v2.7.0
- tracing v0.1.44
- tracing-core v0.1.36
- urlpattern v0.3.0
- webview2-com v0.38.2
- webview2-com-macros v0.8.1
- webview2-com-sys v0.38.2
- winnow v1.0.4
- zmij v1.0.23

### Apache-2.0 OR MIT

- bit-set v0.8.0
- bit-vec v0.8.0
- ctor v0.8.0
- ctor-proc-macro v0.0.7
- equivalent v1.0.2
- fastrand v2.4.1
- idna_adapter v1.2.2
- indexmap v1.9.3
- indexmap v2.14.0
- muda v0.19.3
- pin-project-lite v0.2.17
- portable-atomic v1.13.1
- rustc-hash v2.1.3
- tauri v2.11.5
- tauri-codegen v2.6.3
- tauri-macros v2.6.3
- tauri-plugin-dialog v2.7.1
- tauri-plugin-fs v2.5.1
- tauri-runtime v2.11.3
- tauri-runtime-wry v2.11.4
- tauri-utils v2.9.3
- utf8_iter v1.0.4
- uuid v1.23.5
- window-vibrancy v0.6.0
- wry v0.55.1

### MIT/Apache-2.0

- bitflags v1.3.2
- fallible-iterator v0.3.0
- fallible-streaming-iterator v0.1.9
- ident_case v1.0.1
- json-patch v3.0.1
- lexical-core v1.0.6
- lexical-parse-float v1.0.6
- lexical-parse-integer v1.0.6
- lexical-util v1.0.7
- lexical-write-float v1.0.6
- lexical-write-integer v1.0.6
- siphasher v1.0.3
- unic-char-property v0.9.0
- unic-char-range v0.9.0
- unic-common v0.9.0
- unic-ucd-ident v0.9.0
- unic-ucd-version v0.9.0
- winapi v0.3.9

### Unicode-3.0

- icu_collections v2.2.0
- icu_locale_core v2.2.0
- icu_normalizer v2.2.0
- icu_normalizer_data v2.2.0
- icu_properties v2.2.0
- icu_properties_data v2.2.0
- icu_provider v2.2.0
- litemap v0.8.2
- potential_utf v0.1.5
- tinystr v0.8.3
- writeable v0.6.3
- yoke v0.8.3
- yoke-derive v0.8.2
- zerofrom v0.1.8
- zerofrom-derive v0.1.7
- zerotrie v0.2.4
- zerovec v0.11.6
- zerovec-derive v0.11.3

### Apache-2.0

- arrow v58.3.0
- arrow-arith v58.3.0
- arrow-buffer v58.3.0
- arrow-cast v58.3.0
- arrow-data v58.3.0
- arrow-ord v58.3.0
- arrow-row v58.3.0
- arrow-schema v58.3.0
- arrow-select v58.3.0
- arrow-string v58.3.0
- tao v0.35.3

### MPL-2.0

- cssparser v0.36.0
- cssparser-macros v0.6.1
- dtoa-short v0.3.5
- option-ext v0.2.0
- selectors v0.36.1

### Unlicense OR MIT

- aho-corasick v1.1.4
- byteorder v1.5.0
- memchr v2.8.3
- winapi-util v0.1.11

### Unlicense/MIT

- csv v1.4.0
- csv-core v0.1.13
- same-file v1.0.6
- walkdir v2.5.0

### Zlib

- foldhash v0.1.5
- foldhash v0.2.0
- zlib-rs v0.6.6

### BSD-3-Clause

- alloc-no-stdlib v2.0.4
- alloc-stdlib v0.2.4

### Apache-2.0 AND MIT

- arrow-array v58.3.0
- dpi v0.1.2

### Zlib OR Apache-2.0 OR MIT

- bytemuck v1.25.1
- bytemuck_derive v1.11.0

### BSD-2-Clause OR Apache-2.0 OR MIT

- zerocopy v0.8.54
- zerocopy-derive v0.8.54

### 0BSD OR MIT OR Apache-2.0

- adler2 v2.0.1

### BSD-3-Clause AND MIT

- brotli v8.0.4

### BSD-3-Clause/MIT

- brotli-decompressor v5.0.3

### CC0-1.0 OR MIT-0 OR Apache-2.0

- dunce v1.0.5

### Apache-2.0 / MIT

- fnv v1.0.7

### MIT OR Zlib OR Apache-2.0

- miniz_oxide v0.8.9

### MIT OR Apache-2.0 OR Zlib

- raw-window-handle v0.6.2

### Apache-2.0 OR BSL-1.0

- ryu v1.0.23

### MPL-2.0+

- smartstring v1.0.1

### CC0-1.0

- tiny-keccak v2.0.2

### (MIT OR Apache-2.0) AND Unicode-3.0

- unicode-ident v1.0.24

## JavaScript packages

154 packages.

| Licence | Count |
|---|---|
| MIT | 76 |
| BSD-3-Clause | 40 |
| ISC | 28 |
| Apache-2.0 OR MIT | 3 |
| Apache-2.0 | 3 |
| MIT OR Apache-2.0 | 1 |
| Unlicense | 1 |
| 0BSD | 1 |
| BSD-2-Clause | 1 |

### MIT

- @codemirror/autocomplete v6.20.3
- @codemirror/commands v6.10.4
- @codemirror/lang-python v6.2.1
- @codemirror/language v6.12.4
- @codemirror/lint v6.9.7
- @codemirror/search v6.7.1
- @codemirror/state v6.7.1
- @codemirror/view v6.43.6
- @esbuild/win32-x64 v0.25.12
- @jridgewell/gen-mapping v0.3.13
- @jridgewell/remapping v2.3.5
- @jridgewell/resolve-uri v3.1.2
- @jridgewell/sourcemap-codec v1.5.5
- @jridgewell/trace-mapping v0.3.31
- @lezer/common v1.5.2
- @lezer/highlight v1.2.3
- @lezer/lr v1.4.10
- @lezer/python v1.1.19
- @marijn/find-cluster-break v1.0.3
- @rollup/rollup-win32-x64-gnu v4.62.2
- @rollup/rollup-win32-x64-msvc v4.62.2
- @sveltejs/acorn-typescript v1.0.11
- @sveltejs/vite-plugin-svelte v5.1.1
- @sveltejs/vite-plugin-svelte-inspector v4.0.1
- @types/estree v1.0.9
- @types/geojson v7946.0.4
- @types/trusted-types v2.0.7
- acorn v8.17.0
- ansi-regex v5.0.1
- ansi-styles v4.3.0
- clsx v2.1.1
- codemirror v6.0.2
- color-convert v2.0.1
- color-name v1.1.4
- commander v2.20.3
- commander v7.2.0
- crelt v1.0.7
- debug v4.4.3
- deepmerge v4.3.1
- devalue v5.8.1
- dockview-core v7.0.2
- emoji-regex v8.0.0
- esbuild v0.25.12
- escalade v3.2.0
- esm-env v1.2.2
- esrap v2.3.0
- fast-json-patch v3.1.1
- fdir v6.5.0
- iconv-lite v0.6.3
- is-fullwidth-code-point v3.0.0
- is-reference v3.0.3
- json-stringify-pretty-compact v4.0.0
- kleur v4.1.5
- locate-character v3.0.0
- magic-string v0.30.21
- ms v2.1.3
- nanoid v3.3.16
- node-fetch v2.7.0
- picomatch v4.0.5
- postcss v8.5.19
- require-directory v2.1.1
- rollup v4.62.2
- safer-buffer v2.1.2
- string-width v4.2.3
- strip-ansi v6.0.1
- style-mod v4.1.3
- svelte v5.56.6
- tinyglobby v0.2.17
- tr46 v0.0.3
- vite v6.4.3
- vitefu v1.1.3
- w3c-keyname v2.2.8
- whatwg-url v5.0.0
- wrap-ansi v7.0.0
- yargs v17.7.3
- zimmerframe v1.1.4

### BSD-3-Clause

- @webgpu/types v0.1.71
- rw v1.3.3
- source-map-js v1.2.1
- vega v5.33.1
- vega-canvas v1.2.7
- vega-crossfilter v4.1.4
- vega-dataflow v5.7.8
- vega-embed v6.29.0
- vega-encode v4.10.3
- vega-event-selector v3.0.1
- vega-expression v5.1.2
- vega-expression v5.2.1
- vega-force v4.2.3
- vega-format v1.1.4
- vega-functions v5.18.1
- vega-geo v4.4.4
- vega-hierarchy v4.1.4
- vega-interpreter v1.2.1
- vega-label v1.3.2
- vega-lite v5.23.0
- vega-loader v4.5.4
- vega-parser v6.6.1
- vega-projection v1.6.3
- vega-regression v1.3.2
- vega-runtime v6.2.2
- vega-scale v7.4.3
- vega-scenegraph v4.13.2
- vega-schema-url-parser v2.2.0
- vega-selections v5.6.3
- vega-statistics v1.9.0
- vega-themes v2.15.0
- vega-time v2.1.4
- vega-tooltip v0.35.2
- vega-transforms v4.12.2
- vega-typings v1.5.1
- vega-util v1.17.4
- vega-view v5.16.1
- vega-view-transforms v4.6.2
- vega-voronoi v4.2.5
- vega-wordcloud v4.1.7

### ISC

- cliui v8.0.1
- d3-array v3.2.4
- d3-color v3.1.0
- d3-delaunay v6.0.4
- d3-dispatch v3.0.1
- d3-dsv v3.0.1
- d3-force v3.0.0
- d3-format v3.1.2
- d3-geo v3.1.1
- d3-geo-projection v4.0.0
- d3-hierarchy v3.1.2
- d3-interpolate v3.0.1
- d3-path v3.1.0
- d3-quadtree v3.0.1
- d3-scale v4.0.2
- d3-scale-chromatic v3.1.0
- d3-shape v3.2.0
- d3-time v3.1.0
- d3-time-format v4.1.0
- d3-timer v3.0.1
- delaunator v5.1.0
- get-caller-file v2.0.5
- internmap v2.0.3
- picocolors v1.1.1
- semver v7.8.5
- topojson-client v3.1.0
- y18n v5.0.8
- yargs-parser v21.1.1

### Apache-2.0 OR MIT

- @tauri-apps/api v2.11.1
- @tauri-apps/cli v2.11.4
- @tauri-apps/cli-win32-x64-msvc v2.11.4

### Apache-2.0

- aria-query v5.3.1
- axobject-query v4.1.0
- typescript v5.6.3

### MIT OR Apache-2.0

- @tauri-apps/plugin-dialog v2.7.1

### Unlicense

- robust-predicates v3.0.3

### 0BSD

- tslib v2.8.1

### BSD-2-Clause

- webidl-conversions v3.0.1

