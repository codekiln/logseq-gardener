use std::io::{self, Write};

pub fn emit(text: &str, code: u8, stderr: bool) -> u8 {
    let result = if stderr {
        io::stderr().lock().write_all(text.as_bytes())
    } else {
        io::stdout().lock().write_all(text.as_bytes())
    };
    match result {
        Ok(()) => code,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => 0,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "error: cannot write output: {error}");
            1
        }
    }
}

/// Escape control characters in filesystem paths and external diagnostics.
pub fn clean(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            if c.is_control() && c != '\n' && c != '\t' {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}

/// JSON escapes must follow JSON syntax, including controls serde_json leaves
/// literal (DEL and Unicode C1 controls). Escape before adding the newline.
pub fn json_document(value: &serde_json::Value) -> String {
    let mut text = String::new();
    for c in value.to_string().chars() {
        if c.is_control() {
            text.push_str(&format!("\\u{:04x}", u32::from(c)));
        } else {
            text.push(c);
        }
    }
    text.push('\n');
    text
}
