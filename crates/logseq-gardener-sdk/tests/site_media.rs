use logseq_gardener_sdk::{
    page_titles::FilenameFormat, publishing::NamespaceSelection, site::publish_site,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

mod support;
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-media-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        Self(root)
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.0.join("garden").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn publish(&self) {
        publish_site(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Notes"], &["Notes/Excluded"]).unwrap(),
        )
        .unwrap();
    }
    fn html(&self, title: &str) -> String {
        let hash = title
            .as_bytes()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
            });
        let mut html = fs::read_to_string(self.0.join(format!("site/p-{hash:016x}.html"))).unwrap();
        // Reference sections load these fragments; their copies must stay inert too.
        let stem = format!("p-{hash:016x}-");
        let mut fragments: Vec<_> = fs::read_dir(self.0.join("site/_refs"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(&stem)
            })
            .collect();
        fragments.sort();
        for fragment in fragments {
            html.push_str(&support::read_text(&fragment));
        }
        html
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

const OMER: &str = "https://www.youtube.com/watch?v=6cqVzgVQJfE";

fn videos_page() -> String {
    format!(
        "- {{{{youtube-timestamp 5}}}} before any video\n\
- {{{{video {OMER}}}}}\n\
\t- ## {{{{youtube-timestamp 280}}}} Chapter\n\
\t  id:: 6a1d2b4c-0000-4000-8000-000000000001\n\
\t- {{{{youtube-timestamp 620}}}} Agent skill\n\
\t  id:: 6a1d2b4c-0000-4000-8000-000000000002\n\
\t- {{{{youtube-timestamp 3600}}}} An hour in [[Notes/Other]]\n\
\t- {{{{youtube-timestamp 61:00}}}} rejected\n\
- {{{{vimeo https://vimeo.com/76979871}}}}\n\
- {{{{video https://www.youtube.com/watch?v=TQsP_PlCY1I&t=205s}}}}\n\
- {{{{video https://www.youtube.com/watch?v=TQsP_PlCY1I}}}}\n\
\t- {{{{youtube-timestamp 10:20}}}} under the repeat\n\
- {{{{video}}}} and {{{{video not a url}}}}\n\
- ![Film](../assets/film.mp4)\n"
    )
}

#[test]
fn videos_render_click_to_load_placeholders_and_timestamps_bind_in_outline_order() {
    let f = Fixture::new();
    f.write("pages/Notes___Videos.md", &videos_page());
    f.write("pages/Notes___Other.md", "- Plain notes\n");
    f.write("assets/film.mp4", "not really a film");
    f.publish();
    let html = f.html("Notes/Videos");
    assert!(!html.contains("{{video http"), "{html}");
    assert!(html.contains(
        "<div class=\"lsg-video\" data-lsg-video=\"0\" data-provider=\"youtube\" data-embed=\"https://www.youtube.com/embed/6cqVzgVQJfE?enablejsapi=1\"><a class=\"lsg-video-play\" href=\"https://www.youtube.com/watch?v=6cqVzgVQJfE\" aria-label=\"Play YouTube video\"></a></div>"
    ));
    assert!(html.contains("data-lsg-video=\"1\" data-provider=\"vimeo\" data-embed=\"https://player.vimeo.com/video/76979871\""));
    assert!(html.contains("data-lsg-video=\"2\" data-provider=\"youtube\" data-embed=\"https://www.youtube.com/embed/TQsP_PlCY1I?enablejsapi=1\" data-start=\"205\""));
    assert!(html.contains("<span class=\"svg-small youtube-timestamp\">00:05</span>"));
    assert!(html.contains(
        "<a class=\"svg-small youtube-timestamp\" data-lsg-video=\"0\" data-seconds=\"280\" href=\"https://www.youtube.com/watch?v=6cqVzgVQJfE&amp;t=280s\">04:40</a>"
    ));
    assert!(html.contains(">01:00:00</a>"));
    assert!(!html.contains("61:00"));
    assert!(html.contains(
        "data-lsg-video=\"3\" data-seconds=\"620\" href=\"https://www.youtube.com/watch?v=TQsP_PlCY1I&amp;t=620s\">10:20</a>"
    ));
    assert!(html.contains("<span class=\"warning\" title=\"Empty URL\">{{video}}</span>"));
    assert!(
        html.contains("<span class=\"warning\" title=\"Invalid URL\">{{video not a url}}</span>")
    );
    assert!(html.contains("<video src=\"assets/"));
    assert!(html.contains("controls preload=\"none\""));
    assert!(html.contains("<script defer src=\"media.js\"></script>"));
    assert!(html.contains("<link rel=\"stylesheet\" href=\"media.css\">"));
    assert!(f.0.join("site/media.js").exists());
    assert!(f.0.join("site/media.css").exists());
    let other = f.html("Notes/Other");
    assert!(!other.contains("media.js") && !other.contains("media.css"));
}

#[test]
fn copied_timestamps_link_to_their_source_video_and_read_as_text_in_references() {
    let f = Fixture::new();
    f.write("pages/Notes___Videos.md", &videos_page());
    f.write(
        "pages/Notes___Other.md",
        "- See ((6a1d2b4c-0000-4000-8000-000000000002))\n- {{embed ((6a1d2b4c-0000-4000-8000-000000000001))}}\n",
    );
    f.publish();
    let other = f.html("Notes/Other");
    assert!(other.contains("10:20 Agent skill"), "{other}");
    assert!(other.contains(
        "<a class=\"svg-small youtube-timestamp\" href=\"https://www.youtube.com/watch?v=6cqVzgVQJfE&amp;t=280s\">04:40</a>"
    ));
    assert!(!other.contains("data-lsg-video"));
    assert!(!other.contains("media.js"));
    // Linked References copy the hour timestamp from the videos page.
    assert!(other.contains(
        "<a class=\"svg-small youtube-timestamp\" href=\"https://www.youtube.com/watch?v=6cqVzgVQJfE&amp;t=3600s\">01:00:00</a>"
    ));
}
