use std::fs;

use parse::Story;

mod gui;
mod parse;
mod tui;

// TODO: set up parsing, to parse it into our data structure
// then figure out tui renderer
// then egui renderer, and with web

pub fn run() {
    let str = fs::read_to_string("iraq-2004.fater").unwrap();
    let story = Story::parse(&str);
}
