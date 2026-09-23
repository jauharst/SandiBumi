# G3-01 — the first release candidate, built from a clean clone (2026-09-23)

Receipt for increment **G3-01** of `docs/takeover/GATE3_PROGRAM.md`. It records one artifact and
the gate that preceded it, and it is written so that every number can be re-derived from the files
it names.

**What this evidences.** SB-CORE-041's fresh-clone clause, observed for the first time: a clone
taken from GitHub, with empty dependency caches, resolves every dependency from the lockfiles and
passes the full gate. SB-INS-001's *first half*: an MSI now exists, and its digest, version,
identifier, scope and build commit are recorded from the artifact itself.

**What it does not.** The MSI is **unsigned** (SB-INS-001's second half). It has not been installed
anywhere — the clean-machine install and standard-user launch are G3-02 (T-INS-02). The gate ran by
hand, not on a machine that runs it for every change (SB-CORE-042, awaiting Jauhar's CI decision).
T-INS-01's Pass is Jauhar's mark, not this file's.

## 1. The candidate

Field names in the first column are `installation::InstallerQualification`'s where the schema has
one, so G3-02 extends this record rather than re-typing it.

| Field | Value | Read from |
|---|---|---|
| `installer_file` | `SandiBumi_0.1.0_x64_en-US.msi` | `src-tauri\target\release\bundle\msi\` — the only file there |
| `installer_sha256` | `c197ca7948eae033de7aefe963785ae59b661236b959266af698613ba7c6b520` | `certutil -hashfile <msi> SHA256` and `Get-FileHash`, which agree |
| size | 299,181,159 bytes | file system |
| `build_commit` | `ac85d80fbc7a1e03b9fc63797be3b3759d727a9a` | `git rev-parse HEAD` in the clone |
| `package_type` | MSI | the file |
| `install_scope` | per-machine — `ALLUSERS = 1`, `MSIINSTALLPERUSER` not set | MSI `Property` table |
| platform | `x64;0` | MSI summary information, Template |
| ProductName | `SandiBumi` | MSI `Property` table; also the exe's version resource |
| ProductVersion | `0.1.0` | MSI `Property` table; the exe's ProductVersion and FileVersion; `package.json`, `tauri.conf.json` and `Cargo.toml` all say 0.1.0 |
| identifier | `com.sandibumi.petro` | not an MSI property; found 2 times in the built `sandibumi.exe` (`grep -c -a`) |
| UpgradeCode | `{AB826E11-4B0C-5BAD-9C80-1B0BBC3E3BE8}` | MSI `Property` table — matches `bundle.windows.wix.upgradeCode` |
| ProductCode | `{07CEA87D-8CE4-4F5D-B9C3-C4894B47650E}` | MSI `Property` table — generated per build |
| PackageCode | `{4D36FB03-47BC-4B3D-B700-3DE9CBEEEDB5}` | MSI summary information — generated per build |
| Manufacturer | `sandibumi` | MSI `Property` table — see finding 4 |
| `signature` | **NotSigned** — MSI and exe both | `Get-AuthenticodeSignature` |
| `sandibumi.exe` sha256 | `b278fef68280b7a876f52998252f326300abc743b9a726de73c2e9d05364ca93` (38,693,376 bytes) | `Get-FileHash` on `src-tauri\target\release\sandibumi.exe`; kept so the installed copy can be compared in G3-02 |

Summary-information WordCount is 2: compressed files, and elevation NOT waived — consistent with the
owner-decided deployment (a per-machine MSI installed by IT in system context).

**Where it is.** `C:\sb-clean\receipt\` on the reference machine holds the MSI (re-hashed after the
copy: same digest) and the three logs below. None of it is committed — the MSI is a 285 MB binary
and the logs are identified here by digest instead.

## 2. How it was built

Exactly T-INS-01's steps, plus two things that make "clean" mean empty caches rather than only a
new folder:

```
git clone https://github.com/jauharst/SandiBumi.git C:\sb-clean\repo      (from GitHub, not a local path)
git -C C:\sb-clean\repo checkout --detach ac85d80f
set CARGO_HOME=C:\sb-clean\cargo-home        (created empty: every crate fetched from crates.io)
set CARGO_BUILD_JOBS=16                      (of 32 logical CPUs; see below)
npm ci --cache C:\sb-clean\npm-cache         (created empty; `ci`, so package-lock.json is obeyed and never rewritten)
powershell -ExecutionPolicy Bypass -File tools\check.ps1
<vcvars64.bat -vcvars_ver=14.29> && npm run tauri build
```

- **`CARGO_HOME` empty**: 402 crates were downloaded into it (`ls registry\cache\* | wc -l`). There
  is no global cargo config on this machine to lose, and no repository cargo config either.
- **npm cache empty**: `npm ci` exited 0 in 66 s. npm 12 skipped esbuild's postinstall (its
  install-script policy), and the build did not need it: esbuild ships its binary as the
  `@esbuild/win32-x64` package, and `esbuild --version` answered `0.25.12`. No `allowScripts` edit was
  made, so nothing in the tree was touched to get past it.
- **`CARGO_BUILD_JOBS=16`** caps parallel `cl.exe` workers during the bundled DuckDB compile; an
  uncapped rebuild on this 32-thread machine died with `C1060: compiler is out of heap space` on
  2026-09-02. It changes scheduling, not output.
- **Not emptied**: tauri's own tool cache (`%LOCALAPPDATA%\tauri\WixTools314`, WiX 3.14.1.8722), the
  Rust toolchain, Node, MSVC and Python — these are the documented prerequisites (`CONTRIBUTING.md`
  §1), not dependency caches.

Environment:

| | |
|---|---|
| Windows | 11 Enterprise 25H2, build 26200.9457, x64 |
| Rust | rustc 1.97.0 / cargo 1.97.0 — no `rust-toolchain` pin in the repo; see finding 3 |
| Node / npm | 24.18.0 / 12.0.1 |
| MSVC | 14.29.30133 (vcvars `-vcvars_ver=14.29`), Windows SDK 10.0.26100.0 |
| tauri-cli | 2.11.4 |
| WiX | 3.14.1.8722 (tauri's cache) |
| Python | 3.12.10 with numpy 2.5.1 (a documented prerequisite; nothing in the gate requires it) |

## 3. The gate, from the clone

```
[1/4] repository gates green in 330s
[2/4] verification matrix green in 0s
[3/4] frontend green in 28s
[4/4] backend green in 401s

GATE GREEN in 759s
```

Literal lines from `02-gate.log` (2,241 lines; the last is line 2241). Totals, each from a command
over the whole log rather than a reading of it:

- Rust: **1289 passed, 0 failed, 45 ignored** — awk over every `test result:` line (5 of them: lib
  1282 / 45 ignored, main 0, `governance_contracts` 5, `verification_matrix` 2, doc-tests 0).
- node test runner: **119 passed, 0 failed** — awk over every `ℹ pass` / `ℹ fail` line.
- `Compiling libduckdb-sys` appears once: the bundled DuckDB really was compiled from nothing.
- The tree was clean after the gate (`git status --porcelain` empty).

The same gate took 231 s in total on the warm main tree on 2026-09-02 (PR #259's receipt). Most of
the difference is stage 1 paying for the empty caches: its `cargo check` fetched the index and all
402 crates and compiled DuckDB before anything else could run.

## 4. The build

`npm run tauri build` exited 0: `Finished release profile [optimized] target(s) in 15m 55s`, then
`Finished 1 bundle` naming the MSI above. The release compile reports 31 warnings — exactly the 31 in
`gate2-warning-inventory.json` (`expected_warning_count`), so nothing new.

## 5. Logs

| File (in `C:\sb-clean\receipt\`) | SHA-256 |
|---|---|
| `01-npm-ci.log` | `98b3d0961104222446860b6428f21a3e3ec329ab1c97784f984f8df542bdff2d` |
| `02-gate.log` | `aae6717209975b858d44d5a66e9abae118721da3e19ee735a5d002b84d178877` |
| `03-tauri-build.log` | `b1eb7889a9ee53c3366aa85e43102ba9d9beb8eaad49fb39e63350e1580845fa` |

## 6. Findings

1. **`tauri build` rewrites `src-tauri/Cargo.toml`, and git then calls the tree dirty when it is
   not.** Two seconds into the build the CLI wrote the manifest back with LF endings, dropping the
   CRs that `core.autocrlf=true` put there at checkout. The file it wrote is byte-identical to the
   committed blob (`git hash-object` = `git rev-parse HEAD:src-tauri/Cargo.toml` =
   `943c1cf1432f5bf95fabda879ff5a2767245a7f4`; `git diff` is empty), so the candidate was built from
   exactly `ac85d80f`. But `git status --porcelain` lists it as modified even after
   `git update-index --refresh`. **G3-07's release gate must check content, not porcelain** — an
   "empty status after build" rule would refuse every candidate built on an autocrlf machine.
2. **The install is offline; the build is not.** `bundle.windows.webviewInstallMode = offlineInstaller`
   makes the bundler download the WebView2 runtime installer from `go.microsoft.com` during the
   build, and embed it. This run fetched `MicrosoftEdgeWebView2RuntimeInstallerX64.exe`, 213,053,648
   bytes, SHA-256 `ad9b350625e132481bc0953eee9e032810134df9fedbd7be364c3f4e0e4dbd64`, Authenticode
   **Valid**, signed by Microsoft Corporation, version 1.3.269.9. It is most of the MSI's 285 MB.
   Nothing pins which version a later build will fetch, so two candidates built a month apart can
   carry different runtimes from the same commit — a thing for G3-07's inventory to record, not a
   defect in this candidate.
3. **No toolchain pin.** With no `rust-toolchain` file, "the compiler" is whatever stable rustup
   has installed — 1.97.0 here. The same commit on another machine can compile with a different
   compiler. Recorded; pinning it is a decision, not something this receipt should do quietly.
4. **The installer's Manufacturer is `sandibumi`**, lower-case, because `tauri.conf.json` names no
   `publisher` and the bundler derives one. It is what Windows shows as the publisher in *Apps* and
   what IT sees. Which legal name belongs there is the owner's call; recorded for G3-02, where the
   installed entry is first seen.
5. **The launch card was not checked here.** T-INS-01 expects the built `sandibumi.exe` to show the
   launch card with the `package.json` version. Launching it on the reference machine would open
   the most recent real project through the shared `%APPDATA%\SandiBumi` config, so the version was
   read from the binary instead (version resource 0.1.0, MSI ProductVersion 0.1.0) —
   `project::startup_path()` opens "the most recently opened project that still exists". The visual check
   is left to T-INS-01's own run, and the launch card is brief by design (it appears only after
   400 ms and leaves the moment the project opens).
