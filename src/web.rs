use crate::parse::{self, Section, SectionIdentifier, Story};
use js_sys::{JsString, Reflect};
use std::{cell::RefCell, rc::Rc, sync::Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Element, Event, PointerEvent, Request, RequestInit, RequestMode, Response, console};

thread_local! {
    // static STORY: LazyCell<Mutex<Option<Story>>> = LazyCell::new(|| Mutex::new(None));
    static STORY: RefCell<Story> = unreachable!("initialized before use");
}

// TODO:
// pub async fn run() {
pub fn run() {
    wasm_bindgen_futures::spawn_local(async {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");

        // let p = document.create_element("p").expect("p is valid element");
        // p.set_text_content(Some("Hello World!!!"));
        // body.append_child(&p).unwrap();

        let req = RequestInit::new();
        req.set_method("GET");
        req.set_mode(RequestMode::SameOrigin);

        let page = "iraq-2004.fater";

        let req = Request::new_with_str_and_init(&page, &req).unwrap();

        let resp = JsFuture::from(window.fetch_with_request(&req))
            .await
            .unwrap();

        let resp: Response = resp.dyn_into().unwrap();

        let text_js = JsFuture::from(resp.text().unwrap()).await.unwrap();
        // let js_string = JsString::from(text_js);
        // let string = String::from(js_string);
        let string = text_js.as_string().unwrap();

        let story = parse::load_str(&string).unwrap();

        // console::log_1(&string.clone().into());
        console::log_1(&format!("{:?}", story).into());

        document
            .get_element_by_id("layout")
            .unwrap()
            .set_inner_html(
                &story
                    .sections()
                    .get(&SectionIdentifier::parse((0, "START"), false).unwrap())
                    .unwrap()
                    .to_string(),
            );

        // let p = document.create_element("p").expect("p is valid element");
        // p.set_text_content(Some(&string));
        // body.append_child(&p).unwrap();

        // TODO: we set up the page correctly, with the initial thing, and set
        // up the observers to run when buttons are pressed

        // let string_obj = JsValue::from_str(&string);
        // Reflect::set(&window, &JsValue::from_str("story"), &string_obj).unwrap();
        // Reflect::set(&window, &"story".into(), &story.into()).unwrap();
        // *STORY.get_mut() = Some(story);
        // STORY.with(move |st| {
        //     *st.borrow_mut() = Some(story);
        // });
        STORY.set(story);

        // let container = Rc::new(RefCell::new(None));

        // TODO: how to get access to the closure from inside the closure?

        // *container.borrow_mut() = Some(closure);

        let closure = Closure::wrap(Box::new(handle_click) as Box<dyn FnMut(_)>);

        let choices = document.query_selector_all(".choice").unwrap();

        for i in 0..choices.length() {
            let choice = choices.get(i).unwrap().dyn_into::<Element>().unwrap();

            choice
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
        }
        // for choice in choices.entries() {
        //     let choice = choice.unwrap().dyn_into::<Element>().unwrap();

        //     choice
        //         .add_event_listener_with_callback(".click", closure.as_ref().unchecked_ref())
        //         .unwrap();
        // }

        // let desc = document.get_element_by_id("description").unwrap();
        // desc.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        //     .unwrap();

        closure.forget();
    });
}

// async fn handle_click() {
//     let window = web_sys::window().expect("no global `window` exists");
//     let document = window.document().expect("should have a document on window");

//     let p = document.create_element("p").expect("p is valid element");
//     p.set_text_content(Some(&string));
//     body.append_child(&p).unwrap();
// }

fn handle_click(event: PointerEvent) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // let p = document.create_element("p").expect("p is valid element");
    // p.set_text_content(Some("Hello World!!!"));
    // body.append_child(&p).unwrap();

    let element = event.target().unwrap().dyn_into::<Element>().unwrap();
    // console::log_1(&element.get_attribute("data-fater-goto").unwrap().into());
    let goto = SectionIdentifier::parse(
        (0, &element.get_attribute("data-fater-goto").unwrap()),
        false,
    )
    .unwrap();

    // let str = Reflect::get(&window, &"story".into())
    //     .unwrap()
    //     .as_string()
    //     .unwrap();
    // console::log_1(&str.into());
    // let story: Story = Reflect::get(&window, &"story".into())
    //     .unwrap()
    //     .unchecked_into();

    STORY.with_borrow_mut(|story| {
        console::log_1(&format!("{:?}", story.sections().get(&goto).unwrap()).into());
        console::log_1(
            &story
                .sections()
                .get(&SectionIdentifier::parse((0, "START"), false).unwrap())
                .unwrap()
                .to_string()
                .into(),
        );
        document
            .get_element_by_id("layout")
            .unwrap()
            .set_inner_html(&story.sections().get(&goto).unwrap().to_string());
    });
    let choices = document.query_selector_all(".choice").unwrap();

    let closure = Closure::wrap(Box::new(handle_click) as Box<dyn FnMut(_)>);

    for i in 0..choices.length() {
        let choice = choices.get(i).unwrap().dyn_into::<Element>().unwrap();

        choice
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
    }

    closure.forget();
}
