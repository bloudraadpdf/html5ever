use html5ever::driver;
use html5ever::serialize;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{RcDom, SerializableHandle};

#[test]
fn from_utf8() {
    let dom = driver::parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .one("<title>Test".as_bytes());
    let mut serialized = Vec::new();
    let document: SerializableHandle = dom.document.clone().into();
    serialize::serialize(&mut serialized, &document, Default::default()).unwrap();
    assert_eq!(
        String::from_utf8(serialized).unwrap().replace(' ', ""),
        "<html><head><title>Test</title></head><body></body></html>"
    );
}

#[test]
fn many_templates() {
    let mut body = String::new();
    for _ in 1..10000 {
        body.push_str("<template>");
    }
    let _ = driver::parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .one(body.as_bytes());
}

#[test]
fn opted_in_self_closing_html_elements_do_not_capture_following_text() {
    for tag in ["span", "marker", "b", "style", "script"] {
        let source = format!("<!doctype html><body><div><{tag}/>After</div>");
        let mut options = html5ever::ParseOpts::default();
        options.tree_builder.honor_self_closing = true;
        let dom = driver::parse_document(RcDom::default(), options).one(source);
        let mut serialized = Vec::new();
        let document: SerializableHandle = dom.document.clone().into();
        serialize::serialize(&mut serialized, &document, Default::default()).unwrap();
        assert_eq!(String::from_utf8(serialized).unwrap(), format!("<!DOCTYPE html><html><head></head><body><div><{tag}></{tag}>After</div></body></html>"));
    }
}

#[test]
fn default_html_parsing_retains_nonvoid_self_closing_behaviour() {
    let dom = driver::parse_document(RcDom::default(), Default::default()).one("<!doctype html><body><div><span/>After</div>");
    let mut serialized = Vec::new();
    let document: SerializableHandle = dom.document.clone().into();
    serialize::serialize(&mut serialized, &document, Default::default()).unwrap();
    assert_eq!(String::from_utf8(serialized).unwrap(), "<!DOCTYPE html><html><head></head><body><div><span>After</span></div></body></html>");
}
