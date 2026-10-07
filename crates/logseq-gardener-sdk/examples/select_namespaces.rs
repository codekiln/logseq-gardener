use logseq_gardener_sdk::publishing::NamespaceSelection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let selection = NamespaceSelection::new(&["Logseq"], &["Logseq/Draft"])?;
    for page in [
        "Logseq",
        "Logseq/Frontmatter",
        "Logseq/Draft/Ideas",
        "LogseqOther",
    ] {
        if selection.includes(page) {
            println!("{page}");
        }
    }
    Ok(())
}
