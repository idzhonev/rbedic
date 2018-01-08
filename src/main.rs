// Copyright 2018 Ivan Dzhonev
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//#![feature(use_extern_macros)]
extern crate gtk;
extern crate gdk;
#[macro_use]
extern crate lazy_static;
extern crate regex;
#[macro_use] extern crate log;
extern crate env_logger;

pub mod ui;
pub mod database;

use ui::App;

fn main() {
    env_logger::init().unwrap();
    info!("Starting up");

    // Initialize the UI's initial state
    App::new()
    // Connect events to the UI
    .connect_events()
    // Display the UI and execute the program
    .then_execute();
}
