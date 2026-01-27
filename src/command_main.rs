pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let doc_paths: Vec<std::path::PathBuf> = noargs::opt("doc-path")
        .short('d')
        .ty("FILE|DIR[:FILE|DIR]...")
        .doc("TODO")
        .env("SIRU_DOC_PATH")
        .default("target/doc/")
        .take(args)
        .then(|a| a.value().split(':').map(|a| a.parse()).collect())?;

    let mut target_crates = std::collections::HashSet::new();
    while let Some(a) = noargs::opt("crate")
        .short('c')
        .doc("TODO")
        .take(args)
        .present()
    {
        target_crates.insert(a.value().to_owned());
    }

    let mut target_kinds = std::collections::HashSet::new();
    while let Some(kinds) = noargs::opt("kind")
        .short('k')
        .ty("KIND")
        .doc("Filter items by kind")
        .take(args)
        .present_and_then(|a| {
            crate::doc::ItemKind::parse_keyword_str(a.value()).ok_or_else(|| "TODO")
        })?
    {
        target_kinds.extend(kinds);
    }

    // TODO:   <ITEM_NAME_PART>...
    let verbose = noargs::flag("verbose")
        .short('v')
        .doc("Enable verbose output")
        .take(args)
        .is_present();

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
            doc.public_items
                .retain(|(_, item)| target_kinds.contains(&item.kind));
        }
        if verbose {
            eprintln!("Public items in crate '{}':", doc.crate_name);
            for (path, item) in &doc.public_items {
                eprintln!("  [{}] {}", item.kind, path);
            }
        }
        if !doc.public_items.is_empty() {
            docs.push(doc);
        }
    }

    // show summary

    for _doc in docs {
        //
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
