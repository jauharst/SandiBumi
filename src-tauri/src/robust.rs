//! Constants shared by robust estimators.
//!
//! Keeping the value and its authority here prevents two filters from quietly using almost-the-
//! same scale and then reporting their thresholds as comparable.

/// Gaussian consistency multiplier for the median absolute deviation.
///
/// Derived as the reciprocal of the standard-normal 75th percentile. This is a mathematical
/// estimator constant, not a petrophysical cutoff or field calibration.
pub const C_MAD: f64 = 1.482_602;

/// Source record for [`C_MAD`], kept as data rather than only as prose.
pub const C_MAD_SOURCE: &str =
    "docs/PRD_v2/20_envcorr-qc.md §5.3 — reciprocal of the standard-normal 75th percentile";

// A source-free definition is a build error, not metadata that a reviewer must remember to inspect.
const _: () = assert!(!C_MAD_SOURCE.is_empty());

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    fn rust_sources(path: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(path).expect("read the Rust source tree") {
            let path = entry.expect("read a Rust source entry").path();
            if path.is_dir() {
                rust_sources(&path, out);
            } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn the_mad_gaussian_consistency_constant_has_one_cited_definition_and_no_duplicate_literal() {
        // CORRECTNESS — SB-ENV-T41. docs/PRD_v2/20_envcorr-qc.md §4.4, §5.3 and
        // §6.4 require one named, cited definition of the Gaussian MAD consistency constant,
        // every current robust estimator to use it, and no second numeric literal.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut paths = Vec::new();
        rust_sources(&root, &mut paths);
        paths.sort();

        let definition = ["pub const C_", "MAD: f64"].concat();
        let source_definition = ["pub const C_", "MAD_SOURCE: &str"].concat();
        let numeric_prefix = ["1.", "4826"].concat();
        let mut definition_sites = Vec::new();
        let mut source_sites = Vec::new();
        let mut numeric_sites = Vec::new();

        for path in &paths {
            let source = std::fs::read_to_string(path).expect("read a UTF-8 Rust source file");
            let normalized = source.replace('_', "");
            for (line_index, (line, normalized_line)) in
                source.lines().zip(normalized.lines()).enumerate()
            {
                if line.contains(&definition) {
                    definition_sites.push(format!("{}:{}", path.display(), line_index + 1));
                }
                if line.contains(&source_definition) {
                    source_sites.push(format!("{}:{}", path.display(), line_index + 1));
                }
                if normalized_line.contains(&numeric_prefix) {
                    numeric_sites.push(format!("{}:{}", path.display(), line_index + 1));
                }
            }
        }

        assert_eq!(definition_sites.len(), 1, "one C_MAD definition: {definition_sites:?}");
        assert_eq!(source_sites.len(), 1, "one C_MAD source declaration: {source_sites:?}");
        assert_eq!(
            numeric_sites,
            definition_sites,
            "the value may occur only on its one named definition: {numeric_sites:?}",
        );
        let chapter_value = ["1.", "482602"]
            .concat()
            .parse::<f64>()
            .expect("the chapter value is numeric");
        assert_eq!(super::C_MAD, chapter_value, "the one definition must hold the cited value");

        for consumer in ["condition.rs", "frame.rs"] {
            let source = std::fs::read_to_string(root.join(consumer)).expect("read MAD consumer");
            assert!(
                source.contains("crate::robust::C_MAD"),
                "{consumer} must consume the shared constant",
            );
        }
        assert_eq!(
            super::C_MAD_SOURCE,
            "docs/PRD_v2/20_envcorr-qc.md §5.3 — reciprocal of the standard-normal 75th percentile",
            "the definition's authority must be executable metadata, not an unattached comment",
        );
    }
}
