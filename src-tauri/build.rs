use std::path::Path;

/// SB-DBM-002 (DEC-021): the per-module source digest, computed at build time. The artefact
/// boundary is the FILE that holds the module body; the cross-machine stability rule is that
/// the digest covers NORMALIZED source bytes - CR bytes dropped so a CRLF checkout and an LF
/// checkout of identical text carry identical identity - and nothing path- or
/// timestamp-dependent enters the hash. Accepted cost per the ruling: a comment or formatting
/// edit moves the digest. That over-reports change and never under-reports it.
fn emit_module_source_digests() {
    use sha2::{Digest, Sha256};
    const MODULE_FILES: &[&str] = &[
        "modules.rs",
        "ssc.rs",
        "lrlc.rs",
        "satheight.rs",
        "lithology.rs",
        "rocktyping.rs",
        "facies.rs",
        "condition.rs",
        "frame.rs",
        "unconventional.rs",
        "equations.rs",
    ];
    for name in MODULE_FILES {
        let path = Path::new("src").join(name);
        println!("cargo:rerun-if-changed=src/{name}");
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("module source {name} must be readable at build time: {e}"));
        let normalized: Vec<u8> = bytes.into_iter().filter(|b| *b != b'\r').collect();
        let digest = Sha256::digest(&normalized);
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        let stem = name.trim_end_matches(".rs").to_uppercase();
        println!("cargo:rustc-env=SB_MODULE_DIGEST_{stem}={}", &hex[..16]);
    }
}

fn main() {
    emit_module_source_digests();
    tauri_build::build()
}
