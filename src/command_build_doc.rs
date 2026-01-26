pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<bool> {
    if !noargs::cmd("build-doc")
        .doc("Build JSON format documentation via rustdoc")
        .take(args)
        .is_present()
    {
        return Ok(false);
    }

    if args.metadata().help_mode {
        return Ok(true);
    }

    eprintln!(
        "Running: `$ cargo doc` with RUSTC_BOOTSTRAP=1 and RUSTDOCFLAGS='-Z unstable-options --output-format json'"
    );
    let status = std::process::Command::new("cargo")
        .env("RUSTC_BOOTSTRAP", "1")
        .env("RUSTDOCFLAGS", "-Z unstable-options --output-format json")
        .args(&["doc"])
        .status()
        .map_err(|e| format!("Failed to run cargo doc: {}", e))?;

    if !status.success() {
        // Cargo has already printed detailed error messages to stderr.
        // Exit with error code without additional logging.
        std::process::exit(1);
    }

    Ok(true)
}
