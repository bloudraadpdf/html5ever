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
    let dom = driver::parse_document(RcDom::default(), Default::default())
        .one("<!doctype html><body><div><span/>After</div>");
    let mut serialized = Vec::new();
    let document: SerializableHandle = dom.document.clone().into();
    serialize::serialize(&mut serialized, &document, Default::default()).unwrap();
    assert_eq!(
        String::from_utf8(serialized).unwrap(),
        "<!DOCTYPE html><html><head></head><body><div><span>After</span></div></body></html>"
    );
}

#[test]
fn processing_instruction_tokens_survive_every_input_boundary() {
    let source = "<?before?><!doctype html><html><?html?><head><?head?></head><?afterhead?><body><table><?table?><colgroup><?column?></colgroup><tbody><tr><td><?cell?></td></tr></tbody></table><template><?template?></template><svg><?foreign?></svg></body><?afterbody?></html><?afterhtml?>";
    let mut baseline = None;
    for split in 0..=source.len() {
        let mut parser = driver::parse_document(RcDom::default(), Default::default());
        parser.process(source[..split].into());
        parser.process(source[split..].into());
        let dom = parser.finish();
        assert_eq!(
            dom.quirks_mode.get(),
            html5ever::interface::QuirksMode::NoQuirks
        );
        let mut graph = vec![(dom.document.clone(), "".to_owned())];
        let mut instruction_count = 0;
        while let Some((node, parent)) = graph.pop() {
            let label = match &node.data {
                markup5ever_rcdom::NodeData::Element {
                    name,
                    template_contents,
                    ..
                } => {
                    if let Some(contents) = template_contents.borrow().as_ref() {
                        graph.extend(
                            contents
                                .children
                                .borrow()
                                .iter()
                                .map(|child| (child.clone(), "#template-content".to_owned())),
                        );
                    }
                    name.local.to_string()
                },
                markup5ever_rcdom::NodeData::ProcessingInstruction { target, .. } => {
                    let expected_parent = match &**target {
                        "before" | "afterhtml" => "#document",
                        "html" | "afterhead" | "afterbody" => "html",
                        "head" => "head",
                        "table" => "table",
                        "column" => "colgroup",
                        "cell" => "td",
                        "template" => "#template-content",
                        "foreign" => "svg",
                        _ => panic!("unexpected PI target {target}"),
                    };
                    assert_eq!(
                        parent, expected_parent,
                        "native parent for {target} at boundary {split}"
                    );
                    instruction_count += 1;
                    continue;
                },
                _ => "#document".to_owned(),
            };
            graph.extend(
                node.children
                    .borrow()
                    .iter()
                    .map(|child| (child.clone(), label.clone())),
            );
        }
        assert_eq!(instruction_count, 11, "native PI count at boundary {split}");
        let mut nodes = vec![dom.document.clone()];
        let template = loop {
            let node = nodes.pop().expect("missing native template element");
            if matches!(&node.data, markup5ever_rcdom::NodeData::Element { name, .. } if &*name.local == "template")
            {
                break node;
            }
            nodes.extend(node.children.borrow().iter().cloned());
        };
        assert!(template.children.borrow().is_empty());
        let markup5ever_rcdom::NodeData::Element {
            template_contents, ..
        } = &template.data
        else {
            unreachable!()
        };
        let contents = template_contents.borrow();
        let contents = contents
            .as_ref()
            .expect("missing template content fragment");
        let children = contents.children.borrow();
        assert_eq!(
            children.len(),
            1,
            "native template contents at boundary {split}"
        );
        assert!(
            matches!(&children[0].data, markup5ever_rcdom::NodeData::ProcessingInstruction { target, contents } if &**target == "template" && contents.is_empty())
        );
        let mut serialized_contents = Vec::new();
        let template: SerializableHandle = template.clone().into();
        serialize::serialize(&mut serialized_contents, &template, Default::default()).unwrap();
        assert_eq!(serialized_contents, b"<?template ?>");
        let mut serialized = Vec::new();
        let document: SerializableHandle = dom.document.clone().into();
        serialize::serialize(&mut serialized, &document, Default::default()).unwrap();
        let serialized = String::from_utf8(serialized).unwrap();
        for target in [
            "before",
            "html",
            "head",
            "afterhead",
            "table",
            "column",
            "cell",
            "template",
            "foreign",
            "afterbody",
            "afterhtml",
        ] {
            assert!(
                serialized.contains(&format!("<?{target} ?>")),
                "missing {target} at boundary {split}: {serialized}"
            );
        }
        if let Some(baseline) = &baseline {
            assert_eq!(&serialized, baseline, "boundary {split}");
        } else {
            baseline = Some(serialized);
        }
    }
}

#[test]
fn processing_instruction_fragments_use_context_insertion_and_raw_text_rules() {
    use html5ever::{ns, parse_fragment, QualName};
    for context in [
        "div", "table", "colgroup", "tbody", "tr", "td", "select", "template",
    ] {
        let dom = parse_fragment(
            RcDom::default(),
            Default::default(),
            QualName::new(None, ns!(html), context.into()),
            vec![],
            true,
        )
        .one("<?task a=\"b\"?>");
        let root = dom.document.children.borrow()[0].clone();
        assert_eq!(
            root.children.borrow().len(),
            1,
            "fragment context {context}"
        );
        assert!(
            matches!(&root.children.borrow()[0].data, markup5ever_rcdom::NodeData::ProcessingInstruction { target, contents } if &**target == "task" && &**contents == "a=\"b\""),
            "fragment context {context}"
        );
    }
    for context in ["script", "style", "title", "textarea"] {
        let dom = parse_fragment(
            RcDom::default(),
            Default::default(),
            QualName::new(None, ns!(html), context.into()),
            vec![],
            true,
        )
        .one("<?task?>");
        let root = dom.document.children.borrow()[0].clone();
        assert!(
            matches!(&root.children.borrow()[0].data, markup5ever_rcdom::NodeData::Text { contents } if &**contents.borrow() == "<?task?>"),
            "raw fragment context {context}"
        );
    }
}

#[test]
fn template_fragment_form_rules_ignore_even_a_seeded_form_pointer() {
    use html5ever::{local_name, ns, tree_builder::create_element, QualName};
    for (source, expected) in [
        (
            "<form><form><input>",
            "<html><form><form><input></form></form></html>",
        ),
        (
            "<br>BC<form>D<div>E</form>F</div>G",
            "<html><br>BC<form>D<div>E</div></form>FG</html>",
        ),
    ] {
        let sink = RcDom::default();
        let context = create_element(
            &sink,
            QualName::new(None, ns!(html), local_name!("template")),
            vec![],
        );
        let form = create_element(
            &sink,
            QualName::new(None, ns!(html), local_name!("form")),
            vec![],
        );
        let dom = driver::parse_fragment_for_element(
            sink,
            Default::default(),
            context,
            false,
            Some(form),
        )
        .one(source);
        let mut output = Vec::new();
        let document: SerializableHandle = dom.document.clone().into();
        serialize::serialize(&mut output, &document, Default::default()).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }
}

#[test]
fn template_table_form_is_retained_without_changing_the_outer_pointer() {
    let dom = driver::parse_document(RcDom::default(), Default::default())
        .one("<form id=outer><template><table><form id=inside></table></template><input></form>");
    let mut output = Vec::new();
    let document: SerializableHandle = dom.document.clone().into();
    serialize::serialize(&mut output, &document, Default::default()).unwrap();
    assert_eq!(String::from_utf8(output).unwrap(), "<html><head></head><body><form id=\"outer\"><template><table><form id=\"inside\"></form></table></template><input></form></body></html>");
}

fn assert_search_parse(source: &str, body: &str) {
    let dom = driver::parse_document(RcDom::default(), Default::default())
        .one(format!("<!doctype html>{source}"));
    let document: SerializableHandle = dom.document.clone().into();
    let mut output = Vec::new();
    serialize::serialize(&mut output, &document, Default::default()).unwrap();
    assert_eq!(
        String::from_utf8(output).unwrap(),
        format!("<!DOCTYPE html><html><head></head><body>{body}</body></html>"),
        "source: {source}"
    );
}

#[test]
fn search_is_a_special_furthest_block_for_adoption_agency() {
    assert_search_parse("<a><search>x</a>y", "<a></a><search><a>x</a>y</search>");
    assert_search_parse(
        "<a><search>x<p>y<a>z",
        "<a></a><search><a>x</a><p><a>y</a><a>z</a></p></search>",
    );
}

#[test]
fn search_stops_implicit_list_item_end_tag_searches() {
    assert_search_parse(
        "<li>a<search><li>b",
        "<li>a<search><li>b</li></search></li>",
    );
    assert_search_parse(
        "<dd>a<search><dt>b",
        "<dd>a<search><dt>b</dt></search></dd>",
    );
}

#[test]
fn search_blocks_unmatched_generic_end_tags() {
    assert_search_parse("<x><search></x>y", "<x><search>y</search></x>");
}

#[test]
fn search_retains_block_start_end_and_foreign_namespace_rules() {
    assert_search_parse(
        "<p>foo<search>bar<p>baz",
        "<p>foo</p><search>bar<p>baz</p></search>",
    );
    assert_search_parse(
        "<search><p>foo</search>bar",
        "<search><p>foo</p></search>bar",
    );
    assert_search_parse(
        "xxx<svg><x><g><a><search><b>",
        "xxx<svg><x><g><a><search></search></a></g></x></svg><b></b>",
    );
}
