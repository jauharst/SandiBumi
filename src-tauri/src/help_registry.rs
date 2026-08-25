//! Per-module Help-card content - the brief method statement, the equations in plain
//! Unicode, and the PUBLISHED references (paper or publication only). Internal
//! provenance (PRD sections, Geolog line numbers, vendor helpfiles) stays in the
//! manifests' own source strings and in the guidebook chapter, where it belongs; it
//! never appears on this card.
//!
//! A registry keyed by module id, like `param_sources`, rather than new fields on
//! `ModuleSpec` - adding a card never touches the 52 manifest literals. The dump gate
//! (`manifest_reference_test.rs`) merges these entries into
//! `docs/generated/module_manifests.json` under a `help` key, so the HTML guidebook
//! (`tools/gen-guidebook.mjs`) renders the same card from the same single home.
//!
//! CITATION DISCIPLINE (docs/guidebook_prompt.md): a reference is COPIED from where
//! the repo records it, never written from memory. For the vsh_gr transforms the repo
//! deliberately records only author-year leads - docs/PRD_v2/10_clay-volume.md
//! escalation E4 marks the primary citations unverified, records that IP2018's
//! reported citations for these three are fabricated, and forbids closing the gap
//! from a secondary source (refusal R8). So the card carries exactly the recorded
//! leads and says its citations are pending verification. When E4 is resolved the
//! full citations replace the leads HERE and flow to card and book together.

use std::path::{Path, PathBuf};

#[derive(Clone, serde::Serialize)]
pub struct ModuleHelp {
    /// Two or three sentences: the method statement, without implementation caveats.
    pub summary: &'static str,
    /// Plain Unicode lines, copied from the module's own arithmetic - never LaTeX.
    pub equations: &'static [&'static str],
    /// Published references exactly as the repo records them. Empty for utilities.
    pub references: &'static [&'static str],
    /// Optional user-facing caveat under the reference lines. Empty means none.
    pub note: &'static str,
}

pub fn module_help(module: &str) -> Option<ModuleHelp> {
    match module {
        // Equations copied from `modules::vsh_gr` (the match on OPT_GR); the exact
        // normalised Larionov forms are DEC-096, the published decimals kept for parity.
        "vsh_gr" => Some(ModuleHelp {
            summary: "Shale volume from the gamma-ray log. A gamma-ray index is taken \
                      between a clean-sand endpoint (GR_MA) and a shale endpoint (GR_SH), \
                      then optionally passed through a published non-linear transform - \
                      Stieber, Larionov or Clavier. VSH is the result limited to 0-1; \
                      VSH_GR keeps the unlimited value beside it.",
            equations: &[
                "IGR = (GR − GR_MA) / (GR_SH − GR_MA)",
                "LINEAR: VSH = IGR",
                "Stieber: VSH = IGR / (3 − 2·IGR)   [variants: IGR / (2 − IGR), IGR / (4 − 3·IGR)]",
                "Larionov, Mesozoic and older: VSH = (2^(2·IGR) − 1) / (2² − 1)   [published decimal: 0.33·(2^(2·IGR) − 1)]",
                "Larionov, Tertiary / unconsolidated: VSH = (2^(3.7·IGR) − 1) / (2^3.7 − 1)   [published decimal: 0.083·(2^(3.7·IGR) − 1)]",
                "Clavier: VSH = 1.7 − √(3.38 − (IGR + 0.7)²)",
            ],
            references: &[
                "Larionov (1969) - the Mesozoic-and-older and Tertiary transforms",
                "Stieber (1970/71) - the three ratio forms",
                "Clavier et al. (1971)",
            ],
            note: "Author and year as this project's method ledger records them; the full \
                   primary citations are pending verification.",
        }),
        _ => None,
    }
}

/// Where this module's guidebook chapter lives, if it exists. The bundled app carries
/// the book as a resource (`guide/book/`); a dev checkout falls back to the repo's own
/// `docs/guide/book/`, so the link works in both layouts and simply hides when the
/// chapter is not written yet.
pub fn guide_chapter_path(resource_dir: Option<PathBuf>, module: &str) -> Option<PathBuf> {
    if module.is_empty()
        || !module
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return None;
    }
    let file = format!("{module}.html");
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = resource_dir {
        candidates.push(dir.join("guide").join("book").join(&file));
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../docs/guide/book")
            .join(&file),
    );
    candidates.into_iter().find(|p| p.is_file())
}

/// Open a chapter in the OS default browser - `cmd start`, zero dependencies. The
/// path was just verified to exist by `guide_chapter_path`, and its file name is
/// constrained to `[a-z0-9_].html` there, so nothing user-shaped reaches the shell.
pub fn open_in_default_browser(path: &Path) -> Result<(), String> {
    let mut cmd = std::process::Command::new("cmd");
    cmd.arg("/c").arg("start").arg("").arg(path);
    crate::python_engine::hide_console(&mut cmd);
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("Could not open the guidebook chapter: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Module ids that carry a help card. The test below pins that every id here is a
    /// real module, so a typo in a `module_help` match arm cannot sit unreachable.
    /// Test-only on purpose: production reaches cards through `module_help` alone.
    const MODULES_WITH_HELP: &[&str] = &["vsh_gr"];

    /// Pins the card's own rule from both sides: every registered id is a real module
    /// (a typo'd match arm cannot sit unreachable), and no reference line carries
    /// internal provenance - PRD sections, Loglan files and section marks belong to
    /// the guidebook and the manifests, never to the published-reference card.
    #[test]
    fn the_help_card_carries_publications_never_internal_provenance() {
        let module_ids: Vec<String> = crate::modules::list_modules()
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert!(!MODULES_WITH_HELP.is_empty());
        for id in MODULES_WITH_HELP {
            assert!(
                module_ids.iter().any(|m| m == id),
                "help registry names '{id}', which is not a module id"
            );
            let help = module_help(id).unwrap_or_else(|| {
                panic!("'{id}' is listed in MODULES_WITH_HELP but module_help returns None")
            });
            assert!(!help.summary.is_empty() && !help.equations.is_empty());
            for r in help.references {
                for marker in ["docs/", ".lls", ".info", "PRD", "\u{a7}", ".htm"] {
                    assert!(
                        !r.contains(marker),
                        "reference on the '{id}' card carries internal provenance ('{marker}'): {r}"
                    );
                }
            }
        }
        // The other side: an id with a card must be listed, or the dump and the list
        // disagree about what exists.
        for id in &module_ids {
            if module_help(id).is_some() {
                assert!(
                    MODULES_WITH_HELP.contains(&id.as_str()),
                    "module '{id}' has a help card but is missing from MODULES_WITH_HELP"
                );
            }
        }
    }

    /// The path helper refuses anything that is not a plain module id - the one gate
    /// between an IPC string and a shell `start`.
    #[test]
    fn a_guide_path_is_only_ever_a_plain_module_id() {
        for bad in ["", "vsh gr", "../evil", "vsh_gr.html", "VSH_GR", "a&b"] {
            assert!(guide_chapter_path(None, bad).is_none(), "accepted '{bad}'");
        }
    }
}
