use serde_json::Value;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lsg"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}
fn json(args: &[&str]) -> Value {
    let result = run(args);
    assert!(result.status.success(), "{:?}", result);
    assert!(result.stderr.is_empty());
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["format_version"], 1);
    value
}
#[test]
fn traverse_help_and_sections() {
    let mut paths = vec![Vec::<String>::new()];
    while let Some(path) = paths.pop() {
        let prefix: Vec<&str> = path.iter().map(String::as_str).collect();
        let invoke = |suffix: &[&str]| run(&[prefix.as_slice(), suffix].concat());
        let help = invoke(&["help"]);
        assert!(help.status.success());
        assert_eq!(help.stdout, invoke(&["--help"]).stdout);
        assert_eq!(help.stdout, invoke(&["-h"]).stdout);
        let discovered = json(&[prefix.as_slice(), &["help", "--format", "json"]].concat());
        assert_eq!(discovered["command_path"], serde_json::json!(path));
        for child in discovered["child_commands"].as_array().unwrap() {
            let mut child_path = path.clone();
            child_path.push(child["name"].as_str().unwrap().into());
            assert!(child_path.len() < 4);
            paths.push(child_path);
        }
        for programmatic in [false, true] {
            let flags = if programmatic {
                vec!["--programmatic"]
            } else {
                vec![]
            };
            let outline = json(
                &[
                    prefix.as_slice(),
                    &["help", "outline", "--format", "json"],
                    &flags,
                ]
                .concat(),
            );
            let headings = outline["headings"].as_array().unwrap();
            for (index, heading) in headings.iter().enumerate() {
                let name = heading["section"].as_str().unwrap();
                for recursive in [false, true] {
                    let mut args = [
                        prefix.as_slice(),
                        &["help", "section", name, "--format", "json"],
                        &flags,
                    ]
                    .concat();
                    if recursive {
                        args.push("--recursive");
                    }
                    let response = json(&args);
                    let expected = if recursive {
                        1 + headings[index + 1..]
                            .iter()
                            .take_while(|h| h["level"].as_u64() > heading["level"].as_u64())
                            .count()
                    } else {
                        1
                    };
                    assert_eq!(response["sections"].as_array().unwrap().len(), expected);
                    assert_eq!(response["sections"][0]["section"], name);
                }
            }
        }
    }
}
#[test]
fn invalid_invocations_keep_stdout_empty() {
    for args in [
        vec!["--unknown"],
        vec!["--unknown\x1b[31m"],
        vec!["page", "resolve", "note"],
        vec!["help", "section", "missing", "--format", "json"],
        vec!["help", "outline", "--level", "0"],
        vec!["help", "outline", "--level", "2", "--max-level", "3"],
        vec!["--format", "xml"],
        vec!["version", "extra"],
        vec!["help", "section"],
    ] {
        let result = run(&args);
        assert_eq!(result.status.code(), Some(2), "{args:?}");
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("help"));
        assert!(!result.stderr.contains(&27));
    }
}
#[test]
fn version_aliases_and_clean_output() {
    let version = run(&["version"]);
    for args in [&["--version"][..], &["-V"]] {
        assert_eq!(version.stdout, run(args).stdout);
    }
    let result = json(&["version", "--format", "json"]);
    assert_eq!(result["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(result["program"], "lsg");
    for args in [
        vec![],
        vec!["help"],
        vec!["version"],
        vec!["help", "--no-input", "--programmatic"],
    ] {
        let result = run(&args);
        assert!(result.status.success());
        assert!(result.stderr.is_empty());
        assert!(!result.stdout.contains(&27));
    }
}
#[cfg(unix)]
#[test]
fn closed_downstream_pipe_is_successful() {
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;
    let (writer, reader) = UnixStream::pair().unwrap();
    drop(reader);
    let status = Command::new(env!("CARGO_BIN_EXE_lsg"))
        .arg("help")
        .stdout(Stdio::from(OwnedFd::from(writer)))
        .status()
        .unwrap();
    assert!(status.success());
}

#[cfg(target_os = "linux")]
#[test]
fn output_failure_returns_one() {
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_lsg"))
        .arg("help")
        .stdout(Stdio::from(full))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("cannot write output"));
}
