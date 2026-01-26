pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let mut cargo_args = vec!["doc".to_owned()];
    while let Some(a) = noargs::arg("[CARGO_DOC_ARG]...")
        .doc("")
        .take(args)
        .present()
    {
        cargo_args.push(a.value().to_owned());
    }

    if args.metadata().help_mode {
        return Ok(());
    }

    eprintln!(
        "Running: `$ cargo {}` with RUSTC_BOOTSTRAP=1 and RUSTDOCFLAGS='-Z unstable-options --output-format json'",
        cargo_args.join(" ")
    );
    let status = std::process::Command::new("cargo")
        .env("RUSTC_BOOTSTRAP", "1")
        .env("RUSTDOCFLAGS", "-Z unstable-options --output-format json")
        .args(&cargo_args)
        .status()
        .map_err(|e| format!("Failed to run cargo command: {e}"))?;

    if !status.success() {
        // Cargo has already printed detailed error messages to stderr.
        // Exit with error code without additional logging.
        std::process::exit(1);
    }

    Ok(())
}
