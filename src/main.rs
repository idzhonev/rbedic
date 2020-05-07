// Copyright 2018 Ivan Dzhonev
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//#![feature(use_extern_macros)]
extern crate env_logger;
extern crate gdk;
extern crate gtk;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;

extern crate clap;
extern crate dirs;

use clap::{Arg};

pub mod ui;
pub mod database;

use ui::App;

fn main() {
    env_logger::init().unwrap();
    info!("Starting up");

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    let matches: clap::ArgMatches<'static> = clap::App::new("RBEdic")
                          .version(VERSION)
                          .author("Ivan Dzhonev <ivan.dzhonev@gmail.com>")
                          .about("Bulgarian-English two-way dictionary ")
                          .arg(Arg::with_name("history_file")
                               .short("h")
                               .long("history")
                               .value_name("FILE")
                               .help("Sets history file for reading.\nDefault file is ~/new_words.txt")
                               .takes_value(true)
                          )
                          .arg(Arg::with_name("not_reading_history_file")
                               .short("n")
                               .long("noread")
                               .help("Prevents from reading the history file on startup")
                               .takes_value(false)
                          )
                          .get_matches();

    let mut home_dir = dirs::home_dir().unwrap();
    trace!("clap: Value of home_dir: {:?}", home_dir);
    home_dir.push("new_words.txt");
    trace!("clap: Value of home_dir . file: {:?}", home_dir);
    let home_dir_os_string = home_dir.into_os_string();
    let default_history_filename = home_dir_os_string.into_string().unwrap();
    trace!("clap: Value of default_history_filename: {:?}",default_history_filename);

    // Gets a value for history file path if supplied by user, or defaults to "~/new_words.txt"
    let history_file_path: String = matches.value_of("history_file").unwrap_or(&default_history_filename).to_string();
    debug!("clap: Value for History File: {:?}", history_file_path);

    // Gets a value for preventing to read history file if supplied by user, or defaults to "read"
    let prevents_reading_history_file: u64 = matches.occurrences_of("not_reading_history_file");
    debug!("clap: Value of preventing read from history file: {:?}", prevents_reading_history_file);
    let mut prevents_reading_history_file_bool: bool  = false;
    if prevents_reading_history_file > 0 {
        //history_file_path = "".to_string();
        prevents_reading_history_file_bool = true;
    }

    // Initialize the UI's initial state
    App::new(&history_file_path, prevents_reading_history_file_bool)
    // Connect events to the UI
    .connect_events()
    // Display the UI and execute the program
    .then_execute();
}
