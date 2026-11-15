use html5ever::interface::{ElementFlags, NodeOrText, QuirksMode};
use html5ever::tendril::TendrilSink;
use html5ever::{
    driver, expanded_name, local_name, ns, tree_builder::TreeSink, Attribute, ExpandedName,
    ParseOpts, QualName,
};
use markup5ever::tendril::StrTendril;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

#[derive(Default)]
struct Sink {
    next_id: Cell<usize>,
    names: RefCell<HashMap<usize, &'static QualName>>,
    form_associations: Cell<usize>,
}

impl Sink {
    fn get_id(&self) -> usize {
        let id = self.next_id.get();
        self.next_id.set(id + 2);
        id
    }
}

impl TreeSink for Sink {
    type Handle = usize;
    type Output = Self;
    type ElemName<'a> = ExpandedName<'a>;
    fn finish(self) -> Self {
        self
    }

    fn get_document(&self) -> usize {
        0
    }

    fn get_template_contents(&self, target: &usize) -> usize {
        if let Some(expanded_name!(html "template")) =
            self.names.borrow().get(target).map(|n| n.expanded())
        {
            target + 1
        } else {
            panic!("not a template element")
        }
    }

    fn same_node(&self, x: &usize, y: &usize) -> bool {
        x == y
    }

    fn elem_name(&self, target: &usize) -> ExpandedName<'_> {
        self.names
            .borrow()
            .get(target)
            .expect("not an element")
            .expanded()
    }

    fn create_element(&self, name: QualName, _: Vec<Attribute>, _: ElementFlags) -> usize {
        let id = self.get_id();
        // N.B. We intentionally leak memory here to minimize the implementation complexity
        //      of this example code. A real implementation would either want to use a real
        //      real DOM tree implentation, or else use an arena as the backing store for
        //      memory used by the parser.
        self.names
            .borrow_mut()
            .insert(id, Box::leak(Box::new(name)));
        id
    }

    fn associate_with_form(&self, _: &usize, _: &usize, _: (&usize, Option<&usize>)) {
        self.form_associations.set(self.form_associations.get() + 1);
    }

    fn create_comment(&self, _text: StrTendril) -> usize {
        self.get_id()
    }

    fn create_pi(&self, _target: StrTendril, _value: StrTendril) -> usize {
        self.get_id()
    }

    fn append_before_sibling(&self, _sibling: &usize, _new_node: NodeOrText<usize>) {}

    fn append_based_on_parent_node(
        &self,
        _element: &usize,
        _prev_element: &usize,
        _new_node: NodeOrText<usize>,
    ) {
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {}
    fn set_quirks_mode(&self, _mode: QuirksMode) {}
    fn append(&self, _parent: &usize, _child: NodeOrText<usize>) {}

    fn append_doctype_to_document(&self, _: StrTendril, _: StrTendril, _: StrTendril) {}
    fn add_attrs_if_missing(&self, target: &usize, _attrs: Vec<Attribute>) {
        assert!(self.names.borrow().contains_key(target), "not an element");
    }
    fn remove_from_parent(&self, _target: &usize) {}
    fn reparent_children(&self, _node: &usize, _new_parent: &usize) {}
    fn mark_script_already_started(&self, _node: &usize) {}
}

#[test]
fn test_driver_interrupted_by_non_script() {
    // https://github.com/servo/html5ever/issues/716
    let test_case = "<meta charset=\"UTF-8\" /><meta charset=\"UTF-8\" /> other stuff";
    let mut parser = driver::parse_document(Sink::default(), ParseOpts::default());
    parser.process(test_case.into());
    parser.finish();
}

#[test]
fn fragment_select_ignores_input_without_losing_following_options() {
    let sink = driver::parse_fragment(
        Sink::default(),
        ParseOpts::default(),
        QualName::new(None, ns!(html), local_name!("select")),
        vec![],
        false,
    )
    .one("<input><option>choice");
    let names = sink.names.borrow();
    assert!(!names
        .values()
        .any(|name| name.local == local_name!("input")));
    assert!(names
        .values()
        .any(|name| name.local == local_name!("option")));
}

#[test]
fn seeded_fragment_form_pointer_never_assigns_parser_form_owners() {
    let sink = Sink::default();
    let form = html5ever::tree_builder::create_element(
        &sink,
        QualName::new(None, ns!(html), local_name!("form")),
        vec![],
    );
    let context = html5ever::tree_builder::create_element(
        &sink,
        QualName::new(None, ns!(html), local_name!("div")),
        vec![],
    );
    let sink =
        driver::parse_fragment_for_element(sink, ParseOpts::default(), context, false, Some(form))
            .one("<input>");
    assert_eq!(sink.form_associations.get(), 0);
    let sink = driver::parse_document(Sink::default(), ParseOpts::default()).one("<form><input>");
    assert_eq!(sink.form_associations.get(), 1);
}

fn encoding_indicators(input: &str) -> Vec<String> {
    let parser = driver::parse_document(Sink::default(), ParseOpts::default());
    parser.input_buffer.push_back(input.into());
    let mut declarations = Vec::new();
    loop {
        match parser.tokenizer.feed(&parser.input_buffer) {
            html5ever::TokenizerResult::EncodingIndicator(label) => {
                declarations.push(label.to_string())
            },
            html5ever::TokenizerResult::Script(_) => {},
            html5ever::TokenizerResult::Done => break,
        }
    }
    parser.tokenizer.end();
    declarations
}

#[test]
fn encoding_indicators_belong_only_to_meta_elements() {
    for tag in ["base", "basefont", "bgsound", "link"] {
        for attributes in [
            "charset=windows-1251",
            "http-equiv=content-type content='text/html;charset=windows-1251'",
        ] {
            assert_eq!(
                encoding_indicators(&format!("<!doctype html><head><{tag} {attributes}>")),
                Vec::<String>::new(),
                "{tag} {attributes}"
            );
        }
    }
}

#[test]
fn encoding_indicators_keep_real_meta_character_references_and_content_type() {
    for markup in [
        "<meta charset='windows&#45;1251'>",
        "<meta http-equiv=Content-Type content='text/html;charset=windows-1251'>",
    ] {
        assert_eq!(encoding_indicators(markup), ["windows-1251"]);
    }
}

#[test]
fn encoding_indicators_respect_actual_raw_text_and_rcdata_contexts() {
    for tag in ["script", "style", "title"] {
        assert_eq!(
            encoding_indicators(&format!(
                "<head><{tag}><meta charset=windows-1251></{tag}><meta charset=UTF-8>"
            )),
            ["UTF-8"]
        );
    }
}
