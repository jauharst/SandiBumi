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
  ]
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
