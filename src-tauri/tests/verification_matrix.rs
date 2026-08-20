use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn text(path: &Path) -> String {
    String::from_utf8(fs::read(path).unwrap()).unwrap()
}

fn test_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "sandibumi-verification-matrix-{}-{stamp}",
        std::process::id()
    ))
}

#[test]
fn a_capability_matrix_is_generated_from_review_and_a_capability_map_and_checked_by_the_gate() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let script = repo.join("tools/generate-verification-matrix.mjs");
    let gate = repo.join("tools/check.ps1");
    let dir = test_dir();
    fs::create_dir(&dir).unwrap();

    let review = dir.join("REVIEW.md");
    let map = dir.join("capabilities.json");
    let output = dir.join("VERIFICATION_MATRIX.md");
    fs::write(
        &review,
        "# Review\n\n## 2026-08-01 — Log import\n\n- [x] Import a real text delivery\n\n## 2026-08-02 — Uncertainty analysis\n\n- [ ] Run a real uncertainty study\n\n## 2026-08-03 — Legacy capability\n\n> **Try:** Exercise this capability\n\n## Undated exercise\n\n- [ ] Exercise without a recorded date\n",
    )
    .unwrap();
    fs::write(
        &map,
        r#"{
  "schema_version": 1,
  "capabilities": [
    {
      "id": "log-import",
      "title": "Log import",
      "review_sections": ["Log import", "Legacy capability"]
    },
    {
      "id": "uncertainty-analysis",
      "title": "Uncertainty analysis",
      "review_sections": ["Uncertainty analysis"]
    },
    {
      "id": "unlisted-capability",
      "title": "Capability without a ledger scenario",
      "not_listed": true
    },
    {
      "id": "legacy-capability",
      "title": "Legacy capability",
      "review_sections": ["Legacy capability"]
    },
    {
      "id": "undated-exercise",
      "title": "Undated exercise",
      "review_sections": ["Undated exercise"]
    }
  ],
  "unmapped_review_sections": []
}
"#,
    )
    .unwrap();

    let generated = Command::new("node")
        .arg(&script)
        .args(["--review"])
        .arg(&review)
        .args(["--map"])
        .arg(&map)
        .args(["--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "generator failed: {}",
        String::from_utf8_lossy(&generated.stderr)
    );

    let matrix = text(&output);
    assert!(
        matrix.contains("| `log-import` | Log import | Partially exercised | 1 / 1 | 2026-08-01 |")
    );
    assert!(matrix
        .contains("| `uncertainty-analysis` | Uncertainty analysis | Not exercised | 0 / 1 | — |"));
    assert!(matrix.contains(
        "| `unlisted-capability` | Capability without a ledger scenario | Not listed | 0 / 0 | — | 0 |"
    ));
    assert!(matrix
        .contains("| `legacy-capability` | Legacy capability | Not recorded | 0 / 0 | — | 1 |"));
    assert!(matrix.contains("Fully exercised: **0 / 5**."));

    fs::write(
        &review,
        text(&review).replace(
            "- [ ] Exercise without a recorded date",
            "- [x] Exercise without a recorded date",
        ),
    )
    .unwrap();
    let undated = Command::new("node")
        .arg(&script)
        .args(["--review"])
        .arg(&review)
        .args(["--map"])
        .arg(&map)
        .args(["--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(!undated.status.success());
    assert!(String::from_utf8_lossy(&undated.stderr).contains("has no ledger date"));
    fs::write(
        &review,
        text(&review).replace(
            "- [x] Exercise without a recorded date",
            "- [ ] Exercise without a recorded date",
        ),
    )
    .unwrap();

    fs::write(
        &review,
        text(&review).replace(
            "- [ ] Run a real uncertainty study",
            "- [x] Run a real uncertainty study",
        ),
    )
    .unwrap();
    let stale_from_review = Command::new("node")
        .arg(&script)
        .arg("--check")
        .args(["--review"])
        .arg(&review)
        .args(["--map"])
        .arg(&map)
        .args(["--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(!stale_from_review.status.success());
    assert!(String::from_utf8_lossy(&stale_from_review.stderr).contains("out of date"));

    fs::write(
        &review,
        text(&review).replace(
            "- [x] Run a real uncertainty study",
            "- [ ] Run a real uncertainty study",
        ),
    )
    .unwrap();
    fs::write(
        &map,
        text(&map).replace(
            "\"title\": \"Log import\"",
            "\"title\": \"Text log import\"",
        ),
    )
    .unwrap();
    let stale_from_map = Command::new("node")
        .arg(&script)
        .arg("--check")
        .args(["--review"])
        .arg(&review)
        .args(["--map"])
        .arg(&map)
        .args(["--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(!stale_from_map.status.success());
    assert!(String::from_utf8_lossy(&stale_from_map.stderr).contains("out of date"));

    let gate_text = text(&gate);
    assert!(gate_text.contains("generate-verification-matrix.mjs"));
    assert!(gate_text.contains("--check"));

    fs::remove_dir_all(dir).unwrap();
}

/// CORRECTNESS — the matrix checked one direction only. It refused a capability that matched no
/// review section, and said nothing whatever about a review section that matched no capability.
///
/// A section nothing claims contributes to no capability's count. That is a defensible state for
/// the entries predating the capability map, but the silence is not: a newly written section
/// counted toward nothing and the only symptom was a total that failed to move — which is
/// indistinguishable from a total that was already right. The fix is not to force a mapping, which
/// would mean inventing a capability assignment for every historical section; it is to make the
/// unclaimed set EXPLICIT, so an addition to it is a deliberate line in a reviewed file.
///
/// Pinned from BOTH sides, because each refusal alone is satisfied by a lazier implementation. A
/// generator that only refuses unacknowledged sections lets the list rot into a blanket exemption
/// — titles that were retitled or have since been mapped keep excusing nothing. A generator that
/// only refuses stale entries never catches the new unclaimed section, which is the original bug.
#[test]
fn a_review_section_claimed_by_no_capability_is_refused_unless_acknowledged_and_the_acknowledgement_cannot_go_stale(
) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let script = repo.join("tools/generate-verification-matrix.mjs");
    let dir = test_dir();
    fs::create_dir(&dir).unwrap();

    let review = dir.join("REVIEW.md");
    let map = dir.join("capabilities.json");
    let output = dir.join("VERIFICATION_MATRIX.md");

    // Two sections a capability claims, and one nothing claims — the historical case.
    let base_review = "# Review\n\n\
         ## 2026-08-01 — Log import\n\n- [x] Import a real text delivery\n\n\
         ## 2026-08-02 — Uncertainty analysis\n\n- [ ] Run a real uncertainty study\n\n\
         ## 2026-08-03 — A shape nobody mapped\n\n- [ ] Exercise the unmapped thing\n";
    let map_with = |acknowledged: &str| {
        format!(
            r#"{{
  "schema_version": 1,
  "capabilities": [
    {{ "id": "log-import", "title": "Log import", "review_sections": ["Log import"] }},
    {{ "id": "uncertainty", "title": "Uncertainty analysis", "review_sections": ["Uncertainty analysis"] }}
  ],
  "unmapped_review_sections": [{acknowledged}]
}}
"#
        )
    };
    let run = |check: bool| {
        let mut command = Command::new("node");
        command.arg(&script);
        if check {
            command.arg("--check");
        }
        command
            .args(["--review"])
            .arg(&review)
            .args(["--map"])
            .arg(&map)
            .args(["--output"])
            .arg(&output)
            .output()
            .unwrap()
    };

    // Acknowledged: the run succeeds, and the matrix STATES the debt rather than hiding it. A
    // number nobody can see is the same as no check at all.
    fs::write(&review, base_review).unwrap();
    fs::write(&map, map_with(r#""A shape nobody mapped""#)).unwrap();
    let acknowledged = run(false);
    assert!(
        acknowledged.status.success(),
        "an acknowledged unmapped section must generate: {}",
        String::from_utf8_lossy(&acknowledged.stderr)
    );
    assert!(
        text(&output).contains("Review sections counted toward no capability: **1** of 3,"),
        "the generated matrix must publish how much of the ledger it does not count"
    );

    // Side one — a NEW section nothing claims. This is the original bug: it used to generate
    // cleanly and count toward nothing. It must now be refused, and refused BY NAME, because
    // "some section is unmapped" does not tell an author which of six hundred to look at.
    fs::write(
        &review,
        format!("{base_review}\n## 2026-08-04 — Freshly written and unclaimed\n\n- [ ] Try it\n"),
    )
    .unwrap();
    let unacknowledged = run(false);
    assert!(
        !unacknowledged.status.success(),
        "a section claimed by no capability and acknowledged nowhere must be refused"
    );
    let unacknowledged_error = String::from_utf8_lossy(&unacknowledged.stderr).to_string();
    assert!(
        unacknowledged_error.contains("Freshly written and unclaimed")
            && unacknowledged_error.contains("unmapped_review_sections"),
        "the refusal must name the section and where to record it: {unacknowledged_error}"
    );

    // Side two — the list cannot become a blanket exemption. Both ways an entry goes stale are
    // refused: a title that no longer exists, and a title that has since been given a capability.
    fs::write(&review, base_review).unwrap();
    fs::write(
        &map,
        map_with(r#""A shape nobody mapped", "A section that was renamed away""#),
    )
    .unwrap();
    let vanished = run(false);
    assert!(
        !vanished.status.success(),
        "an acknowledgement naming no existing section must be refused"
    );
    assert!(
        String::from_utf8_lossy(&vanished.stderr).contains("A section that was renamed away"),
        "the stale refusal must name the entry to delete"
    );

    fs::write(&map, map_with(r#""A shape nobody mapped", "Log import""#)).unwrap();
    let now_mapped = run(false);
    assert!(
        !now_mapped.status.success(),
        "an acknowledgement for a section that now has a capability must be refused"
    );
    assert!(
        String::from_utf8_lossy(&now_mapped.stderr).contains("Log import"),
        "the stale refusal must name the entry that is no longer unmapped"
    );

    // The key is required rather than defaulted: an absent list would silently restore the
    // one-directional behaviour this whole test exists to prevent.
    fs::write(
        &map,
        r#"{
  "schema_version": 1,
  "capabilities": [
    { "id": "log-import", "title": "Log import", "review_sections": ["Log import"] }
  ]
}
"#,
    )
    .unwrap();
    let absent = run(false);
    assert!(!absent.status.success(), "an absent unmapped list must be refused, not defaulted");
    assert!(String::from_utf8_lossy(&absent.stderr).contains("unmapped_review_sections"));

    fs::remove_dir_all(dir).unwrap();
}
