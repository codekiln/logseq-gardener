mod cli;
mod help;
mod output;

fn main() -> std::process::ExitCode {
    let (text, code, stderr) = match cli::run() {
        Ok(text) => (text, 0, false),
        Err(message) => {
            let clean: String = message
                .chars()
                .flat_map(|c| {
                    if c.is_control() && c != '\n' && c != '\t' {
                        c.escape_default().collect::<Vec<_>>()
                    } else {
                        vec![c]
                    }
                })
                .collect();
            (
                format!(
                    "error: {}\nTry 'lsg help'.\n",
                    clean.trim_start_matches("error: ")
                ),
                2,
                true,
            )
        }
    };
    std::process::ExitCode::from(output::emit(&text, code, stderr))
}
