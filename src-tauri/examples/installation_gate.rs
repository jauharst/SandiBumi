//! Release-lane entry point for validating evidence captured from the final Windows MSI.
//!
//! Usage: `cargo run --example installation_gate -- <installer-qualification.json>
//! <offline-deployment-qualification.json> <clean-machine-qualification.json>`.

use std::path::Path;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(installer_path), Some(offline_path), Some(clean_machine_path)) =
        (args.next(), args.next(), args.next())
    else {
        eprintln!(
            "usage: installation_gate <installer-qualification.json> <offline-deployment-qualification.json> <clean-machine-qualification.json>"
        );
        std::process::exit(2);
    };
    if args.next().is_some() {
        eprintln!(
            "usage: installation_gate <installer-qualification.json> <offline-deployment-qualification.json> <clean-machine-qualification.json>"
        );
        std::process::exit(2);
    }
    if let Err(error) = sandibumi_lib::installation::validate_installer_qualification_file(
        Path::new(&installer_path),
    ) {
        eprintln!("installer qualification: FAIL: {error}");
        std::process::exit(1);
    }
    let installer = match sandibumi_lib::installation::read_installer_qualification_file(Path::new(
        &installer_path,
    )) {
        Ok(installer) => installer,
        Err(error) => {
            eprintln!("installer qualification: FAIL: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) =
        sandibumi_lib::installation::validate_offline_deployment_file(Path::new(&offline_path))
    {
        eprintln!("offline deployment qualification: FAIL: {error}");
        std::process::exit(1);
    }
    if let Err(error) = sandibumi_lib::installation::validate_clean_machine_qualification_file(
        Path::new(&clean_machine_path),
        &installer,
    ) {
        eprintln!("clean-machine qualification: FAIL: {error}");
        std::process::exit(1);
    }
    println!("installer, offline deployment and clean-machine qualification: PASS");
}
