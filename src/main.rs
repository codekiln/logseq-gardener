mod cli;
mod help;
mod output;

fn main() -> std::process::ExitCode {
    let code = match cli::run() {
        Ok(response) => {
            let diagnostics = output::emit(&output::clean(&response.stderr), 0, true);
            let result = output::emit(&response.stdout, 0, false);
            diagnostics.max(result)
        }
        Err(failure) => output::emit(
            &format!(
                "error: {}\nTry 'lsg help'.\n",
                output::clean(&failure.message).trim_start_matches("error: ")
            ),
            failure.code,
            true,
        ),
    };
    std::process::ExitCode::from(code)
}
