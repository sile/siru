fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();

    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    let ext = noargs::flag("ext")
        .short('x')
        .doc("Enable extended subcommands")
        .take(&mut args)
        .is_present();

    if ext {
        let _ = siru::command_build_doc::try_run(&mut args)?;
    } else {
        siru::command_main::run(&mut args)?;
    }

    if let Some(help) = args.finish()? {
        print!("{help}");
    }
    Ok(())
}
