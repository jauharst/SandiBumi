//! Release-lane entry point for validating evidence captured from the final Windows MSI.
//!
//! Usage: `cargo run --example installation_gate -- <installer-qualification.json>
//! <offline-deployment-qualification.json>`.

use std::path::Path;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(installer_path), Some(offline_path)) = (args.next(), args.next()) else {
        eprintln!(
            "usage: installation_gate <installer-qualification.json> <offline-deployment-qualification.json>"
        );
        std::process::exit(2);
    };
    if args.next().is_some() {
        eprintln!(
            "usage: installation_gate <installer-qualification.json> <offline-deployment-qualification.json>"
        );
        std::process::exit(2);
    }
    if let Err(error) = sandibumi_lib::installation::validate_installer_qualification_file(
        Path::new(&installer_path),
    ) {
        eprintln!("installer qualification: FAIL: {error}");
        std::process::exit(1);
    }
    if let Err(error) =
        sandibumi_lib::installation::validate_offline_deployment_file(Path::new(&offline_path))
    {
        eprintln!("offline deployment qualification: FAIL: {error}");
        std::process::exit(1);
    }
    println!("installer and offline deployment qualification: PASS");
}
