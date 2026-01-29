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
            crate::doc::ItemKind::parse_keyword_str(a.value()).ok_or_else(|| {
                format!(
                    "invalid item kind: must be one of {}",
                    crate::doc::ItemKind::KEYWORDS
                )
            })
        })?
    {
        target_kinds.extend(kinds);
    }

    let show_options = ShowOptions {
        show_inner_json: noargs::flag("show-inner-json")
            .doc("Print inner JSON representation before item signature")
            .take(args)
            .is_present(),
        verbose: noargs::flag("verbose")
            .doc("Enable verbose output")
            .take(args)
            .is_present(),
    };

    let mut target_path_parts = Vec::new();
    while let Some(part) = noargs::arg("[ITEM_PATH_PART]...")
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
    if show_options.verbose {
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
            .map_err(|e| crate::json::format_parse_error(&text, &e))?;

        if !target_crates.is_empty() && !target_crates.contains(&doc.crate_name) {
            continue;
        }
        if !known_crates.insert(doc.crate_name.clone()) {
            if show_options.verbose {
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
        if show_options.verbose {
            eprintln!("Items in crate '{}':", doc.crate_name);
            for (path, item) in &doc.show_items {
                let inner = item.inner(&doc.json);
                eprintln!("  [{}] {}: {}", item.kind, path, inner);
            }
        }

        docs.push(doc);
    }

    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    print_output(&mut writer, &docs, &show_options)?;

    Ok(())
}

struct ShowOptions {
    show_inner_json: bool,
    verbose: bool,
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

fn print_output<W: std::io::Write>(
    writer: &mut W,
    docs: &[crate::doc::CrateDoc],
    show_options: &ShowOptions,
) -> crate::Result<()> {
    print_summary(writer, docs, show_options)?;
    for doc in docs {
        if doc.show_items.is_empty() {
            continue;
        }
        print_detail(writer, doc, show_options)?;
    }
    Ok(())
}

fn print_summary<W: std::io::Write>(
    writer: &mut W,
    docs: &[crate::doc::CrateDoc],
    _show_options: &ShowOptions,
) -> crate::Result<()> {
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
    show_options: &ShowOptions,
) -> crate::Result<()> {
    for (path, item) in &doc.show_items {
        writeln!(writer, "# [{}] `{}`\n", item.kind.as_keyword_str(), path)?;

        // Print inner JSON if requested
        if show_options.show_inner_json {
            writeln!(writer, "**Inner JSON**:\n")?;
            writeln!(writer, "```json\n{}\n```\n", item.inner(&doc.json))?;
        }

        print_item_signature(writer, doc, item)?;

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

        let inner = item.inner(&doc.json);
        if let Some(_impls) = inner.to_member("impls")?.get() {
            // TODO
        }

        if let Some(_impls) = inner.to_member("implementations")?.get() {
            // TODO
        }

        writeln!(writer)?;
    }

    Ok(())
}

fn print_item_signature<W: std::io::Write>(
    writer: &mut W,
    doc: &crate::doc::CrateDoc,
    item: &crate::doc::Item,
) -> crate::Result<()> {
    let inner = item.inner(&doc.json);

    writeln!(writer, "```rust")?;
    match item.kind {
        crate::doc::ItemKind::TypeAlias | crate::doc::ItemKind::AssocType => {
            let kw = item.kind.as_keyword_str();
            let view = crate::item_view::TypeView::new(doc, item);
            if let Some(ty) = view.ty()? {
                writeln!(writer, "{kw} {} = {};", view.name()?, ty)?;
            } else {
                writeln!(writer, "{kw} {};", view.name()?)?;
            }
        }
        crate::doc::ItemKind::TraitAlias => {
            let kw = item.kind.as_keyword_str();
            let name = item.name.as_ref().expect("bug");
            let inner = item.inner(&doc.json);
            writeln!(writer, "{kw} {} = {};", name, inner)?;
        }
        crate::doc::ItemKind::Primitive => {
            let view = crate::item_view::PrimitiveView::new(doc, item);
            writeln!(writer, "type {};", view.name())?;
        }
        crate::doc::ItemKind::Constant | crate::doc::ItemKind::AssocConst => {
            let view = crate::item_view::ConstantView::new(doc, item);
            writeln!(writer, "const {}: {};", view.name(), view.ty()?)?;
        }
        crate::doc::ItemKind::Module => {
            let view = crate::item_view::ModuleView::new(doc, item);
            let child_count = view.child_count()?;
            writeln!(
                writer,
                "mod {} {{ /* {} items */ }}",
                view.name(),
                child_count
            )?;
        }
        crate::doc::ItemKind::Macro => {
            writeln!(writer, "{}", inner.to_unquoted_string_str()?)?;
        }
        crate::doc::ItemKind::ProcMacro => {
            let view = crate::item_view::ProcMacroView::new(doc, item);
            writeln!(writer, "{}", view.derive_attribute()?)?;
        }
        crate::doc::ItemKind::StructField => {
            let view = crate::item_view::FieldView::new(doc, item);
            writeln!(writer, "  {}: {}", view.name(), view.ty()?)?;
        }
        crate::doc::ItemKind::Function => {
            let view = crate::item_view::FunctionView::new(doc, item);
            writeln!(writer, "{}", view.signature()?)?;
        }
        crate::doc::ItemKind::Variant => {
            let s = crate::format_item::format_enum_variant_to_string(doc, item)?;
            writeln!(writer, "{s}")?;
        }
        crate::doc::ItemKind::Enum => {
            let s = crate::format_item::format_enum_to_string(doc, item)?;
            writeln!(writer, "{s}")?;
        }
        crate::doc::ItemKind::Trait => {
            let s = crate::format_item::format_trait_to_string(doc, item)?;
            writeln!(writer, "{s}")?;
        }
        crate::doc::ItemKind::Struct => {
            let s = crate::format_item::format_struct_to_string(doc, item)?;
            writeln!(writer, "{s}")?;
        }
        crate::doc::ItemKind::Union => {
            let s = crate::format_item::format_union_to_string(doc, item)?;
            writeln!(writer, "{s}")?;
        }
        kind => todo!("{kind:?}: {inner}"),
    }
    writeln!(writer, "```\n")?;

    Ok(())
}
