pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let doc_paths: Vec<std::path::PathBuf> = noargs::opt("doc-path")
        .short('d')
        .ty("FILE|DIR[:FILE|DIR]...")
        .doc("Path(s) to documentation files or directories containing *.json files, separated by colons")
        .env("SIRU_DOC_PATH")
        .default("target/doc/")
        .take(args)
        .then(|a| a.value().split(':').map(|a| a.parse()).collect())?;

    let mut target_crates = std::collections::HashSet::new();
    while let Some(a) = noargs::opt("crate")
        .short('c')
        .ty("CRATE_NAME")
        .doc("Filter to specific crate(s) by name (can be specified multiple times)")
        .take(args)
        .present()
    {
        target_crates.insert(a.value().to_owned());
    }

    let mut target_kinds = std::collections::HashSet::new();
    while let Some(kinds) = noargs::opt("kind")
        .short('k')
        .ty(crate::doc::ItemKind::KEYWORDS)
        .doc("Filter to specific item kind(s) (can be specified multiple times)")
        .take(args)
        .present_and_then(|a| {
            crate::doc::ItemKind::parse_keyword_str(a.value()).ok_or_else(|| "TODO")
        })?
    {
        target_kinds.extend(kinds);
    }

    let view_command = noargs::opt("view-command")
        .short('v')
        .ty("COMMAND")
        .doc("Shell command to pipe output through (e.g., less, bat -lmd)")
        .env("SIRU_VIEW_COMMAND")
        .take(args)
        .present_and_then(|o| o.value().parse::<String>())?;

    let verbose = noargs::flag("verbose")
        .doc("Enable verbose output")
        .take(args)
        .is_present();

    let mut target_path_parts = Vec::new();
    while let Some(part) = noargs::arg("[ITEM_PATH_PART]")
        .doc("Filter items to only those having all specified path parts")
        .take(args)
        .present_and_then(|a| a.value().parse::<String>())?
    {
        target_path_parts.push(part);
    }

    if args.metadata().help_mode {
        return Ok(());
    }

    let doc_file_paths = collect_doc_file_paths(&doc_paths)?;
    if verbose {
        eprintln!("Documentation file paths:");
        for path in &doc_file_paths {
            eprintln!("  {}", path.display());
        }
    }

    let mut docs = Vec::new();
    let mut known_crates = std::collections::HashSet::new();
    for path in doc_file_paths {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read file '{}': {e}", path.display()))?;
        let mut doc = crate::doc::CrateDoc::parse(path, &text)
            .map_err(|e| crate::json::format_parse_error(&text, e))?;

        if !target_crates.is_empty() && !target_crates.contains(&doc.crate_name) {
            continue;
        }
        if !known_crates.insert(doc.crate_name.clone()) {
            if verbose {
                eprintln!("Warning: duplicate crate '{}' ignored", doc.crate_name);
            }
            continue;
        }
        if !target_kinds.is_empty() {
            doc.show_items
                .retain(|(_, item)| target_kinds.contains(&item.kind));
        }
        if !target_path_parts.is_empty() {
            doc.show_items.retain(|(path, _)| {
                let path = path.to_string();
                target_path_parts.iter().all(|part| path.contains(part))
            });
        }
        if verbose {
            eprintln!("Public items in crate '{}':", doc.crate_name);
            for (path, item) in &doc.show_items {
                eprintln!("  [{}] {}", item.kind, path);
            }
        }

        docs.push(doc);
    }

    let result = if let Some(cmd) = view_command {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

        let mut child = std::process::Command::new(&shell)
            .arg("-c")
            .arg(&cmd)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn shell '{}': {}", shell, e))?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or("failed to get child process stdin")?;
        let result = print_output(&mut stdin, &docs);
        std::mem::drop(stdin);
        let _ = child.wait();
        result
    } else {
        let stdout = std::io::stdout();
        let mut writer = stdout.lock();
        print_output(&mut writer, &docs)
    };

    if let Err(PrintError::Json { error, text }) = result {
        return Err(crate::json::format_parse_error(&text, error).into());
    }
    Ok(())
}

fn collect_doc_file_paths(
    doc_paths: &[std::path::PathBuf],
) -> noargs::Result<Vec<std::path::PathBuf>> {
    let mut file_paths = Vec::new();

    for path in doc_paths {
        if path.is_file() {
            file_paths.push(path.clone());
        } else if path.is_dir() {
            // Collect *.json files non-recursively
            for entry in std::fs::read_dir(path)
                .map_err(|e| format!("failed to read directory '{}': {e}", path.display()))?
            {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
                let file_path = entry.path();

                if file_path.is_file() && file_path.extension().is_some_and(|ext| ext == "json") {
                    file_paths.push(file_path);
                }
            }
        } else {
            return Err(format!(
                "Path '{}' is neither a file nor a directory",
                path.display()
            )
            .into());
        }
    }

    Ok(file_paths)
}

enum PrintError {
    Io, // Output errors are ignored
    Json {
        error: nojson::JsonParseError,
        text: String,
    },
}

impl PrintError {
    fn set_text(self, text: &str) -> Self {
        match self {
            PrintError::Io => PrintError::Io,
            PrintError::Json { error, .. } => PrintError::Json {
                error,
                text: text.to_string(),
            },
        }
    }
}

impl From<std::io::Error> for PrintError {
    fn from(_err: std::io::Error) -> Self {
        PrintError::Io
    }
}

impl From<nojson::JsonParseError> for PrintError {
    fn from(err: nojson::JsonParseError) -> Self {
        PrintError::Json {
            error: err,
            text: String::new(),
        }
    }
}

fn print_output<W: std::io::Write>(
    writer: &mut W,
    docs: &[crate::doc::CrateDoc],
) -> Result<(), PrintError> {
    print_summary(writer, docs)?;
    for doc in docs {
        if doc.show_items.is_empty() {
            continue;
        }
        print_detail(writer, doc).map_err(|e| e.set_text(doc.json.text()))?;
    }
    Ok(())
}

fn print_summary<W: std::io::Write>(
    writer: &mut W,
    docs: &[crate::doc::CrateDoc],
) -> std::io::Result<()> {
    writeln!(writer, "# Crates Overview\n")?;
    for doc in docs {
        writeln!(
            writer,
            "- `{}` ({} public items, {} items to show)",
            doc.crate_name,
            doc.public_item_count,
            doc.show_items.len()
        )?;
    }
    writeln!(writer)?;

    for doc in docs {
        if doc.show_items.is_empty() {
            continue;
        }

        writeln!(writer, "# Crate Items: `{}`\n", doc.crate_name)?;

        // Calculate the longest kind keyword for padding
        let max_kind_len = doc
            .show_items
            .iter()
            .map(|(_, item)| item.kind.as_keyword_str().len())
            .max()
            .unwrap_or(0);

        for (path, item) in &doc.show_items {
            writeln!(
                writer,
                "- [{:<width$}] `{}`",
                item.kind.as_keyword_str(),
                path,
                width = max_kind_len
            )?;
        }

        writeln!(writer)?;
    }

    Ok(())
}

fn print_detail<W: std::io::Write>(
    writer: &mut W,
    doc: &crate::doc::CrateDoc,
) -> Result<(), PrintError> {
    for (path, item) in &doc.show_items {
        writeln!(writer, "# [{}] `{}`\n", item.kind.as_keyword_str(), path)?;

        if let Some(deprecation_note) = item.deprecation_note(&doc.json)? {
            if !deprecation_note.is_empty() {
                writeln!(writer, "**Deprecated**: {}\n", deprecation_note)?;
            } else {
                writeln!(writer, "**Deprecated**\n")?;
            }
        }

        if let Some(docs) = item.docs(&doc.json)? {
            let formatted_docs = crate::markdown::add_rust_to_code_blocks(&docs);
            let increased_headings = crate::markdown::increase_heading_levels(&formatted_docs);
            writeln!(writer, "{}\n", increased_headings)?;
        }

        writeln!(writer)?;
    }

    Ok(())
}
