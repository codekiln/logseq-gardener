use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::publish_site;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn shared_theme_is_available_on_source_collection_and_index_documents() {
    let root = std::env::temp_dir().join(format!(
        "lsg-theme-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let source = root.join("garden/pages");
    fs::create_dir_all(&source).unwrap();
    let original = "- # Readable source\n\t- Preserved content\n";
    fs::write(source.join("Notes___Page.md"), original).unwrap();
    let output = root.join("site");
    let report = publish_site(
        root.join("garden"),
        &output,
        FilenameFormat::TripleLowbar,
        &NamespaceSelection::new(&["Notes"], &[]).unwrap(),
    )
    .unwrap();
    assert_eq!(report.pages, 1);
    let mut documents = 0;
    for entry in fs::read_dir(&output).unwrap() {
        let path = entry.unwrap().path();
        if path
            .extension()
            .is_some_and(|extension| extension == "html")
        {
            let html = fs::read_to_string(path).unwrap();
            assert!(
                html.find("<script data-lsg-theme-script>").unwrap() < html.find("<body>").unwrap()
            );
            assert!(!html.contains("<select data-lsg-theme-select"));
            assert_eq!(
                html.matches("data-lsg-theme-toggle>Toggle theme</button>")
                    .count(),
                1
            );
            assert!(html.contains("data-lsg-theme-system hidden>Use system theme</button>"));
            assert!(html.contains("data-lsg-more aria-label=\"More\" aria-haspopup=\"menu\""));
            assert!(html.contains("@media(prefers-color-scheme:dark)"));
            assert!(html.contains(".theme-control[hidden]{display:none!important}"));
            assert!(html.contains("color:var(--lsg-text);background:var(--lsg-background)"));
            assert!(
                html.contains("--lsg-background:#fbfcfd") && html.contains("--lsg-link:#849dff")
            );
            documents += 1;
        }
    }
    assert!(documents >= 3);
    assert_eq!(
        fs::read_to_string(source.join("Notes___Page.md")).unwrap(),
        original
    );
    fs::remove_dir_all(root).unwrap();
}
