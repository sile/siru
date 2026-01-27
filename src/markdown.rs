pub fn add_rust_to_code_blocks(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.starts_with("```") {
            let rest = &line[3..];

            if rest.is_empty() || rest.chars().all(|c| c.is_whitespace()) {
                result.push_str("```rust\n");
            } else {
                result.push_str(line);
                result.push('\n');
            }

            i += 1;

            while i < lines.len() {
                let current_line = lines[i];
                if current_line.starts_with("```") {
                    result.push_str(current_line);
                    result.push('\n');
                    i += 1;
                    break;
                } else {
                    result.push_str(current_line);
                    result.push('\n');
                    i += 1;
                }
            }
        } else {
            result.push_str(line);
            result.push('\n');
            i += 1;
        }
    }

    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

pub fn increase_heading_levels(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.starts_with('#') {
                let hash_count = line.chars().take_while(|c| *c == '#').count();
                let rest = &line[hash_count..];
                format!("{}{}", "#".repeat(hash_count + 1), rest)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
