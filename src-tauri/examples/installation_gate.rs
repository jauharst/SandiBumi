//! Release-lane entry point for validating evidence captured from the final Windows MSI.
//!
//! Usage: `cargo run --example installation_gate -- <installer-qualification.json>`.

use std::path::Path;

fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: installation_gate <installer-qualification.json>");
        std::process::exit(2);
    };
    match sandibumi_lib::installation::validate_installer_qualification_file(Path::new(&path)) {
        Ok(()) => println!("installer qualification: PASS"),
        Err(error) => {
            eprintln!("installer qualification: FAIL: {error}");
            std::process::exit(1);
        }
    }
}
