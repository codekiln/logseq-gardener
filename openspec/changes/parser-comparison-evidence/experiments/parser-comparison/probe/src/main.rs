use serde_json::{json, Value};
use std::{env, fs};
fn main() {
    let corpus: Vec<Value> = serde_json::from_str(&fs::read_to_string(env::args().nth(1).unwrap()).unwrap()).unwrap();
    let results: Vec<Value> = corpus.iter().map(|item| {
        let input = item["input"].as_str().unwrap();
        let outline = lsdoc::parse_outline(input, item["format"].as_str().unwrap_or("md")).unwrap();
        let mut rebuilt = String::new();
        let mut cursor = 0;
        for header in &outline.headers {
            assert!(header.line.start >= cursor);
            rebuilt.push_str(&input[cursor..header.line.start]);
            rebuilt.push_str(header.line.slice(input).expect("valid UTF-8 range"));
            header.structural_prefix.slice(input).expect("valid prefix range");
            header.line_content.slice(input).expect("valid content range");
            cursor = header.line.end;
        }
        rebuilt.push_str(&input[cursor..]);
        assert_eq!(rebuilt, input);
        json!({"file":item["id"],"outline":outline,"retained_source_equal":true})
    }).collect();
    println!("{}", serde_json::to_string(&results).unwrap());
}
