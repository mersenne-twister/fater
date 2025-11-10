use std::fs;

use parse::Story;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(not(target_arch = "wasm32"))]
mod cli;
mod parse;
#[cfg(not(target_arch = "wasm32"))]
mod tui;
#[cfg(target_arch = "wasm32")]
mod web;

// then figure out tui renderer
// then egui renderer, and with web

#[cfg(not(target_arch = "wasm32"))]
pub fn run() {
    let story = parse::load("iraq-2004.fater").unwrap().unwrap();

    dbg!(story);
}

// entry point to web code
#[wasm_bindgen(start)]
#[cfg(target_arch = "wasm32")]
pub fn run() {
    web::run();
}
