use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A garden in a temporary folder, removed when the value drops.
struct Garden(PathBuf);

impl Garden {
    fn with_config(config: &str, files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-rename-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let garden = Self(root);
        garden.write(CONFIG_PATH, config);
        for (path, text) in files {
            garden.write(path, text);
        }
        garden
    }

    fn new(files: &[(&str, &str)]) -> Self {
        Self::with_config("{:file/name-format :triple-lowbar}\n", files)
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn load(&self) -> GardenIndex {
        GardenIndex::load(&self.0).unwrap()
    }
}

impl Drop for Garden {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn page(path: &str) -> PageId {
    PageId::Document(path.into())
}

/// Each changed file's text after the plan, by the path it ends at.
fn outcome(plan: &RenamePlan) -> BTreeMap<String, String> {
    plan.documents
        .iter()
        .map(|document| (slashes(document.new_path()), document.text()))
        .collect()
}

fn files(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
        .collect()
}

#[test]
fn rewrites_every_way_of_naming_the_page_and_keeps_aliases() {
    let garden = Garden::new(&[
        ("pages/Fruit.md", "alias:: Produce\n\n- Grows on trees.\n"),
        (
            "pages/Notes.md",
            "tags:: fruit, [[Fruit]], other\n\n- [[Fruit]] and [[fruit]] and #Fruit, #fruit. #[[FRUIT]]\n\t- {{embed [[Fruit]]}} [label]([[Fruit]]) [[Produce]] #Produce `[[Fruit]]`\n\t- [[Fruity]] #Fruity [[Fruit ]]\n",
        ),
    ]);
    let index = garden.load();
    let plan = index
        .rename_page(&page("pages/Fruit.md"), " Sweet Food ")
        .unwrap();
    assert_eq!(
        outcome(&plan),
        files(&[
            (
                "pages/Sweet Food.md",
                "alias:: Produce\n\n- Grows on trees.\n"
            ),
            (
                "pages/Notes.md",
                "tags:: Sweet Food, [[Sweet Food]], other\n\n- [[Sweet Food]] and [[Sweet Food]] and #[[Sweet Food]], #[[Sweet Food]]. #[[Sweet Food]]\n\t- {{embed [[Sweet Food]]}} [label]([[Sweet Food]]) [[Produce]] #Produce `[[Fruit]]`\n\t- [[Fruity]] #Fruity [[Fruit ]]\n",
            ),
        ])
    );
    assert_eq!(
        plan.pages,
        [PageRename {
            path: "pages/Fruit.md".into(),
            from: "Fruit".into(),
            to: "Sweet Food".into(),
        }]
    );
    assert_eq!(plan.renamed_file_count(), 1);
    assert_eq!(plan.replacement_count(), 9);
}

#[test]
fn a_tag_stays_bare_when_the_new_name_can_be_a_bare_tag() {
    let garden = Garden::new(&[(
        "pages/Notes.md",
        "tags:: todo\n\n- #todo and #[[todo]] and [[TODO]], #todo.\n",
    )]);
    let index = garden.load();
    let target = index
        .rename_target_at("pages/Notes.md".as_ref(), 20)
        .unwrap();
    assert_eq!(target.name, "todo");
    let Subject::Page(tag) = target.subject else {
        panic!("{target:?}");
    };
    assert_eq!(tag, PageId::Name("todo".into()));
    let plan = index.rename_page(&tag, "task").unwrap();
    assert_eq!(
        outcome(&plan),
        files(&[(
            "pages/Notes.md",
            "tags:: task\n\n- #task and #[[task]] and [[task]], #task.\n"
        )])
    );
    assert!(plan.pages.is_empty());
    let plan = index.rename_page(&tag, "a, b").unwrap();
    assert_eq!(
        outcome(&plan),
        files(&[(
            "pages/Notes.md",
            "tags:: [[a, b]]\n\n- #[[a, b]] and #[[a, b]] and [[a, b]], #[[a, b]].\n"
        )])
    );
}

#[test]
fn namespace_children_follow_with_their_references_and_files() {
    let garden = Garden::new(&[
        ("pages/work.md", "- top\n"),
        ("pages/work___Log.md", "- [[work]] day\n"),
        ("pages/work___Log___2026.md", "- deep\n"),
        (
            "pages/workshop.md",
            "- [[Work/log]] #work/Log/2026 [[work/unwritten]] [[workshop]]\n",
        ),
        ("pages/sub/work___Plan.md", "- in a folder\n"),
        (
            "pages/[[work]] is a tool.md",
            "- nested [[[[Work]] is a tool]] [[see [[work/Log]] too]]\n",
        ),
    ]);
    let index = garden.load();
    let plan = index.rename_page(&page("pages/work.md"), "Job").unwrap();
    assert_eq!(
        outcome(&plan),
        files(&[
            ("pages/Job.md", "- top\n"),
            ("pages/Job___Log.md", "- [[Job]] day\n"),
            ("pages/Job___Log___2026.md", "- deep\n"),
            (
                "pages/workshop.md",
                "- [[Job/log]] #Job/Log/2026 [[Job/unwritten]] [[workshop]]\n"
            ),
            ("pages/sub/Job___Plan.md", "- in a folder\n"),
            (
                "pages/[[Job]] is a tool.md",
                "- nested [[[[Job]] is a tool]] [[see [[Job/Log]] too]]\n"
            ),
        ])
    );
    let titles: Vec<_> = plan.pages.iter().map(|page| page.to.as_str()).collect();
    assert_eq!(
        titles,
        [
            "[[Job]] is a tool",
            "Job/Plan",
            "Job",
            "Job/Log",
            "Job/Log/2026"
        ]
    );
}

#[test]
fn a_rename_of_letter_case_only_goes_through_a_temporary_name() {
    let garden = Garden::new(&[
        ("pages/Foo.md", "- [[foo]] [[Foo]]\n"),
        ("pages/Foo___Bar.md", "- child\n"),
        ("pages/.lsg-rename-1.tmp", "taken\n"),
    ]);
    let index = garden.load();
    let plan = index.rename_page(&page("pages/Foo.md"), "foo").unwrap();
    assert_eq!(
        outcome(&plan),
        files(&[
            ("pages/foo.md", "- [[foo]] [[foo]]\n"),
            ("pages/foo___Bar.md", "- child\n"),
        ])
    );
    let moves: Vec<_> = plan
        .documents
        .iter()
        .flat_map(|document| &document.moves)
        .map(|file| (slashes(&file.from), slashes(&file.to)))
        .collect();
    let pair = |from: &str, to: &str| (from.to_owned(), to.to_owned());
    assert_eq!(
        moves,
        [
            pair("pages/Foo.md", "pages/.lsg-rename-2.tmp"),
            pair("pages/.lsg-rename-2.tmp", "pages/foo.md"),
            pair("pages/Foo___Bar.md", "pages/.lsg-rename-3.tmp"),
            pair("pages/.lsg-rename-3.tmp", "pages/foo___Bar.md"),
        ]
    );
    // The same title gives an empty plan.
    assert!(
        index
            .rename_page(&page("pages/Foo.md"), "Foo")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_title_property_is_kept_and_added_when_the_file_name_cannot_carry_the_title() {
    let garden = Garden::with_config(
        "{}",
        &[
            ("pages/Node.md", "- [[node]]\n"),
            ("pages/Rust.md", "tags:: lang\n\n- text\n"),
            (
                "pages/custom.md",
                "Title:: Shown Name\nalias:: sn\n\n- [[shown name]]\n",
            ),
        ],
    );
    let index = garden.load();
    let node = index
        .rename_page(&page("pages/Node.md"), "Node.js")
        .unwrap();
    assert_eq!(
        outcome(&node),
        files(&[("pages/Node.js.md", "title:: Node.js\n\n- [[Node.js]]\n")])
    );
    let rust = index
        .rename_page(&page("pages/Rust.md"), "Rust v1.0")
        .unwrap();
    assert_eq!(
        outcome(&rust),
        files(&[(
            "pages/Rust v1.0.md",
            "title:: Rust v1.0\ntags:: lang\n\n- text\n"
        )])
    );
    // The legacy format writes a namespace separator as `%2F`.
    let shown = index
        .rename_page(&page("pages/custom.md"), "New/Name")
        .unwrap();
    assert_eq!(
        outcome(&shown),
        files(&[(
            "pages/New%2FName.md",
            "Title:: New/Name\nalias:: sn\n\n- [[New/Name]]\n"
        )])
    );
}

#[test]
fn a_journal_keeps_its_file_and_gets_a_title_property() {
    let garden = Garden::new(&[
        ("journals/2026_01_02.md", "- baked\n"),
        ("pages/Notes.md", "- [[Jan 2nd, 2026]] [[2026-01-02]]\n"),
    ]);
    let index = garden.load();
    let plan = index
        .rename_page(&page("journals/2026_01_02.md"), "Bake Day")
        .unwrap();
    assert_eq!(
        outcome(&plan),
        files(&[
            ("journals/2026_01_02.md", "title:: Bake Day\n\n- baked\n"),
            ("pages/Notes.md", "- [[Bake Day]] [[2026-01-02]]\n"),
        ])
    );
    assert_eq!(plan.renamed_file_count(), 0);
}

#[test]
fn the_home_page_in_the_config_follows_the_rename() {
    let config = "{:file/name-format :triple-lowbar\n :default-home {:page \"home/Start\" :sidebar \"Contents\"}}\n";
    let garden = Garden::with_config(config, &[("pages/Home.md", "- hi\n")]);
    let index = garden.load();
    let plan = index
        .rename_page(&page("pages/Home.md"), "My \"Base\"")
        .unwrap();
    assert_eq!(
        outcome(&plan).get(CONFIG_PATH).map(String::as_str),
        Some(
            "{:file/name-format :triple-lowbar\n :default-home {:page \"My \\\"Base\\\"/Start\" :sidebar \"Contents\"}}\n"
        )
    );
}

#[test]
fn refuses_names_that_cannot_be_given() {
    let garden = Garden::new(&[
        ("pages/A.md", "alias:: Ay\n\n- [[B]] [[ghost]] [[A/x]]\n"),
        ("pages/B.md", "alias:: Bee\n\n- b\n"),
        ("pages/X.md", "- x\n"),
        ("pages/dup1.md", "title:: Same\n"),
        ("pages/dup2.md", "title:: Same\n"),
        ("pages/empty-file.md", " \n"),
        ("pages/front.md", "---\ntitle: Front\n---\n\n- text\n"),
    ]);
    let index = garden.load();
    let a = page("pages/A.md");
    let refusal = |name: &str| index.rename_page(&a, name).unwrap_err();
    assert_eq!(refusal("  "), RenameRefusal::EmptyName);
    assert_eq!(refusal("/"), RenameRefusal::EmptyName);
    assert_eq!(refusal("a\nb"), RenameRefusal::LineBreak);
    let taken = |name: &str| RenameRefusal::NameTaken {
        name: name.into(),
        path: "pages/B.md".into(),
    };
    assert_eq!(refusal("b"), taken("b"));
    assert_eq!(refusal("bee"), taken("bee"));
    assert_eq!(
        refusal("2026-03-04"),
        RenameRefusal::JournalDate {
            name: "2026-03-04".into()
        }
    );
    assert_eq!(
        refusal("empty-file"),
        RenameRefusal::FileExists {
            path: "pages/empty-file.md".into()
        }
    );
    // The page's own alias and a name that only references carry are free.
    assert!(index.rename_page(&a, "Ay").is_ok());
    assert!(index.rename_page(&a, "Ghost").is_ok());
    // `A/x` would become `X/x`... while `A` itself would take the title of X.
    assert_eq!(
        refusal("X"),
        RenameRefusal::NameTaken {
            name: "X".into(),
            path: "pages/X.md".into()
        }
    );
    // A name that only references carry merges into an existing page.
    assert_eq!(
        index
            .rename_page(&PageId::Name("ghost".into()), "B")
            .unwrap_err(),
        taken("B")
    );
    assert!(matches!(
        index.rename_page(&page("pages/dup1.md"), "Other"),
        Err(RenameRefusal::Ambiguous { .. })
    ));
    assert!(matches!(
        index.rename_page(
            &PageId::Journal(crate::journals::JournalDate::new(2026, 1, 9).unwrap()),
            "x"
        ),
        Err(RenameRefusal::JournalWithoutFile { .. })
    ));
    if index
        .document("pages/front.md".as_ref())
        .and_then(|d| d.title.as_deref())
        == Some("Front")
    {
        assert_eq!(
            index
                .rename_page(&page("pages/front.md"), "Back")
                .unwrap_err(),
            RenameRefusal::TitleNotEditable {
                path: "pages/front.md".into()
            }
        );
    }
}

#[test]
fn the_rename_target_follows_what_is_under_the_cursor() {
    let id = "66f1a2b0-0000-4000-8000-000000000001";
    let text = format!(
        "title:: Shown\nalias:: Other\n\n- plain [[malus]] #[[Fruit/Apple]] (({id})) [[Jan 9th, 2026]]\n  key:: v\n"
    );
    let garden = Garden::new(&[
        ("pages/Fruit___Apple.md", "alias:: Malus\n\n- red\n"),
        ("pages/notes.md", &text),
    ]);
    let index = garden.load();
    let notes: &Path = "pages/notes.md".as_ref();
    let at = |needle: &str, inside: usize| {
        index.rename_target_at(notes, text.find(needle).unwrap() + inside)
    };
    let apple = Subject::Page(page("pages/Fruit___Apple.md"));
    let target = at("[[malus]]", 3).unwrap();
    assert_eq!(
        (
            target.subject,
            target.range.text(&text),
            target.name.as_str()
        ),
        (apple.clone(), "malus", "Fruit/Apple")
    );
    let target = at("#[[Fruit", 5).unwrap();
    assert_eq!(
        (target.subject, target.range.text(&text)),
        (apple, "Fruit/Apple")
    );
    let own = Subject::Page(page("pages/notes.md"));
    let target = at("plain", 2).unwrap();
    assert_eq!(
        (
            target.subject,
            target.range.text(&text),
            target.name.as_str()
        ),
        (own.clone(), "", "Shown")
    );
    let target = at("Shown", 2).unwrap();
    assert_eq!((target.subject, target.range.text(&text)), (own, "Shown"));
    assert_eq!(at(id, 2), Err(RenameRefusal::Block));
    assert_eq!(
        at("key::", 1),
        Err(RenameRefusal::PropertyKey { key: "key".into() })
    );
    assert!(matches!(
        at("Jan 9th", 2),
        Err(RenameRefusal::JournalWithoutFile { .. })
    ));
    assert!(matches!(
        index.rename_target_at("pages/none.md".as_ref(), 0),
        Err(RenameRefusal::NoDocument { .. })
    ));
}
