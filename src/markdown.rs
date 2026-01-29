pub fn add_rust_to_code_blocks(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some(rest) = line.strip_prefix("```") {
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
    let mut result = String::new();
    let mut in_code_block = false;

    for line in text.lines() {
        // Check if this line toggles code block state
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            result.push_str(line);
        } else if in_code_block {
            // Don't modify lines inside code blocks
            result.push_str(line);
        } else if line.starts_with('#') {
            // Only modify heading lines outside code blocks
            let hash_count = line.chars().take_while(|c| *c == '#').count();
            let rest = &line[hash_count..];
            result.push_str(&format!("{}{}", "#".repeat(hash_count + 1), rest));
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    // Remove trailing newline if original text didn't have one
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}
