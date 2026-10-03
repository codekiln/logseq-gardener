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
