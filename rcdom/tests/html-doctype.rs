use html5ever::driver;
use html5ever::interface::QuirksMode;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::RcDom;

#[test]
fn empty_html4_system_identifier_selects_quirks() {
    for public in [
        "-//W3C//DTD HTML 4.01 Frameset//EN",
        "-//W3C//DTD HTML 4.01 Transitional//EN",
    ] {
        for public in [public.to_owned(), public.to_ascii_lowercase()] {
            let dom = driver::parse_document(RcDom::default(), Default::default())
                .one(format!("<!DOCTYPE html PUBLIC \"{public}\" \"\"><p>text"));
            assert_eq!(dom.quirks_mode.get(), QuirksMode::Quirks, "{public}");
        }
    }
}

#[test]
fn html4_system_presence_and_srcdoc_mode_remain_distinct() {
    for public in [
        "-//W3C//DTD HTML 4.01 Frameset//EN",
        "-//W3C//DTD HTML 4.01 Transitional//EN",
    ] {
        for (system, expected) in [
            ("", QuirksMode::Quirks),
            (" \"example.dtd\"", QuirksMode::LimitedQuirks),
            (" \" \"", QuirksMode::LimitedQuirks),
        ] {
            let dom = driver::parse_document(RcDom::default(), Default::default()).one(format!(
                "<!DOCTYPE html PUBLIC \"{public}\"{system}><p>text"
            ));
            assert_eq!(dom.quirks_mode.get(), expected, "{public}, system={system}");
        }
        let mut options = html5ever::ParseOpts::default();
        options.tree_builder.iframe_srcdoc = true;
        let dom = driver::parse_document(RcDom::default(), options)
            .one(format!("<!DOCTYPE html PUBLIC \"{public}\" \"\"><p>text"));
        assert_eq!(
            dom.quirks_mode.get(),
            QuirksMode::NoQuirks,
            "srcdoc {public}"
        );
    }
    for source in [
        "<!doctype html><p>text",
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.01//EN\" \"\"><p>text",
    ] {
        let dom = driver::parse_document(RcDom::default(), Default::default()).one(source);
        assert_eq!(dom.quirks_mode.get(), QuirksMode::NoQuirks, "{source}");
    }
}
