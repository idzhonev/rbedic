// Copyright 2018 Ivan Dzhonev
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use gtk;
use gtk::*;
use gdk;
use gdk::enums::key;
use std::process;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::collections::HashMap;
//use log;

use super::{Content, Header};
use database::DictDB;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct App {
    pub window: gtk::Window,
    pub header: Header,
    pub content: Content,
    pub history_dictdb: Rc<Mutex<Vec<DictDB>>>,
    pub history_dictdb_unsorted: Rc<Mutex<Vec<DictDB>>>,
    pub selection_isize: Rc<AtomicIsize>,
    pub searched_hash: Rc<Mutex<HashMap<String, String>>>,
}

/// A wrapped `App` which provides the capability to execute the program.
pub struct ConnectedApp(App);

impl ConnectedApp {
    /// Display the window, and execute the gtk main event loop.
    pub fn then_execute(self) {
        self.0.window.show_all();
        gtk::main();
    }
}

impl App {
    pub fn new(history_file_path: &str, prevents_reading_history_file_bool: bool ) -> App {
        // Initialize GTK before proceeding.
        if gtk::init().is_err() {
            eprintln!("failed to initialize GTK Application");
            process::exit(1);
        }
        // Create a new top level window.
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        // Create a the headerbar and it's associated content.
        let header = Header::new();
        // Create the content container and all of it's widgets.
        let content = Content::new();

        // Set the headerbar as the title bar widget.
        window.set_titlebar(&header.container);
        // Set the title of the window.
        window.set_title("RBEdic");
        // Set the window manager class.
        window.set_wmclass("RBEdic", "RBEdic");
        // The icon the app will display.
        window.set_default_size(580, 310);
        gtk::Window::set_default_icon_name("iconname");
        // Add the content to the window.
        window.add(&content.container);

        // Programs what to do when the exit button is used.
        window.connect_delete_event(move |_, _| {
            main_quit();
            Inhibit(false)
        });
        info!(
            "Running RBEdic v{} with GTK v{}.{}",
            VERSION,
            gtk::get_major_version(),
            gtk::get_minor_version()
        );

        let selection_isize = Rc::new(AtomicIsize::new(0));
        let searched_hash = Rc::new(Mutex::new(HashMap::new()));

        // Loading history from file
        let mut vec_history_db: Vec<DictDB> = Vec::new();
        if prevents_reading_history_file_bool {
            debug!("Do not read history file.");
        } else {
            trace!("Reading the history file.");
            vec_history_db = DictDB::new_history(&history_file_path);
        }

        let vec_history_db_clonned = vec_history_db.clone();
        vec_history_db.sort();
        let history_dictdb = Rc::new(Mutex::new(vec_history_db));

        let history_dictdb_unsorted = Rc::new(Mutex::new(vec_history_db_clonned));



        // Return the application structure.
        App {
            window,
            header,
            content,
            history_dictdb,
            history_dictdb_unsorted,
            selection_isize,
            searched_hash,
        }
    }

    /// Creates external state, and maps all of the UI functionality to the UI.
    pub fn connect_events(self) -> ConnectedApp {
        let content = self.content.clone();
        let content_clonned = content.clone();

        let history_dictdb = self.history_dictdb.clone();
        let history_dictdb_unsorted = self.history_dictdb_unsorted.clone();
        // External state to share across events.
        // Keep track of whether we are fullscreened or not.
        let fullscreen = Rc::new(AtomicBool::new(false));
        let button_history = content_clonned.s_bar.history;
        let button_add_2_history = content_clonned.s_bar.add_2_history;
        {
            button_history.set_sensitive(false);
            button_add_2_history.set_sensitive(false);

            // Connect all of the events that this UI will act upon.
            self.about_event();
            self.history_event();
            self.add_2_history_event(history_dictdb.clone(), history_dictdb_unsorted.clone());
            self.key_events(fullscreen);
        }
        {
            // Enabled initially the history button, if history from file was loaded
            let app_clonned = self.clone();
            {
                debug!("history_event: Clear searched_hash");
                let mut searched_hash_locked = app_clonned.searched_hash.lock().unwrap();
                searched_hash_locked.clear();
            }
            {
                let mut history_dictdb_unsorted =
                    app_clonned.history_dictdb_unsorted.lock().unwrap();
                history_dictdb_unsorted.reverse();
                trace!(
                    "history_event: history_data -> {:?}",
                    history_dictdb_unsorted
                );
                app_clonned.selection(&history_dictdb_unsorted, false, true);
            }
            {
                let mut history_dictdb_unsorted =
                    app_clonned.history_dictdb_unsorted.lock().unwrap();
                history_dictdb_unsorted.reverse();
            }
            let content = self.content.clone();
            let search_entry = content.s_bar.search_entry.clone();
            search_entry.grab_focus();
        }
        // Load dictionaries
        let app = self;
        let app_clonned = app.clone();
        let vec_dict_db = DictDB::new();
        content_clonned
            .s_bar
            .search_entry
            .connect_changed(move |search_selection| {
                let search_text_len = search_selection.get_text_length();
                if search_text_len > 0 && search_text_len < 51 {
                    let search_text = search_selection.get_text();
                    {
                        debug!("connect_events: Clear searched_hash");
                        let mut searched_hash_locked = app_clonned.searched_hash.lock().unwrap();
                        searched_hash_locked.clear();
                    }
                    match search_text {
                        Some(txt) => {
                            trace!("Search txt: {:?}", txt);
                            // Search into databases
                            match DictDB::search(&txt, &vec_dict_db) {
                                Ok(vec_result) => {
                                    // Success
                                    trace!("Search into DB is Ok: {:?}", vec_result);
                                    // Write to GUI
                                    app_clonned.selection(&vec_result, false, false);
                                }
                                Err(vec_result_err) => {
                                    trace!(
                                        "Error: Can not find the exact word: {:?}",
                                        vec_result_err
                                    );
                                    if vec_result_err.len() == 0 {
                                        button_add_2_history.set_sensitive(false);
                                    };
                                    // Write to GUI
                                    app_clonned.selection(&vec_result_err, false, false);
                                }
                            };
                        }
                        None => trace!("None"),
                    };
                }
            });
        // Wrap the `App` within `ConnectedApp` to enable the developer to execute the program.
        ConnectedApp(app)
    }

    /// Handles special functions that should be invoked when certain keys and key combinations
    /// are pressed on the keyboard.
    fn key_events(
        &self,
        fullscreen: Rc<AtomicBool>,
    ) {
        // Grab required references beforehand.
        let add_2_history_button = self.content.s_bar.add_2_history.clone();
        let history_button = self.content.s_bar.history.clone();
        let history_dictdb_clonned = self.history_dictdb.clone();
        // Each key press will invoke this function.
        self.window.connect_key_press_event(move |window, gdk| {
            match gdk.get_keyval() {
                // Fullscreen the UI when F11 is pressed.
                key::F11 => if fullscreen.fetch_xor(true, Ordering::SeqCst) {
                    window.unfullscreen();
                } else {
                    window.fullscreen();
                },
                // View History when ctrl+d is pressed.
                key if key == 'd' as u32 && gdk.get_state().contains(gdk::ModifierType::CONTROL_MASK) => {
                    trace!("Pressed CTRL+h");
                    let history_len: usize;
                    {
                        let history_dictdb_locked = history_dictdb_clonned.lock().unwrap();
                        history_len = history_dictdb_locked.len();
                    }
                    debug!("key_events: history_len -> {}", history_len);
                    if history_len > 0 {
                        history_button.clicked();
                    }
                }
                // Add to History when ctrl+s is pressed.
                key if key == 's' as u32 && gdk.get_state().contains(gdk::ModifierType::CONTROL_MASK) => {
                    trace!("Pressed CTRL+s");
                    add_2_history_button.clicked();
                }
                _ => (),
            }
            Inhibit(false)
        });
    }

    /// Program About button
    fn about_event(&self) {
        let button_about = self.header.about.clone();
        let content_clonned = self.content.clone();
        let history_dictdb = self.history_dictdb.clone();
        let button_add_2_history = content_clonned.s_bar.add_2_history.clone();
        let button_history = content_clonned.s_bar.history.clone();
        let tree_store = content_clonned.inner_paned.words.tree_store.clone();
        let right_buff = content_clonned.inner_paned.translation.buff.clone();
        let about_text = "
  This is RBEdic - Bulgarian-English two-way dictionary,
written in Rust with GTK and analogous to KBE Dictionary.

    Version: 0.2.0

    Web site: https://github.com/idzhonev/rbedic
  
    Copyright: Ivan Dzhonev <ivan.dzhonev@gmail.com>

    License:

  RBEdic is licensed under either of the following, at your option:

* Apache License, Version 2.0 ->
http://www.apache.org/licenses/LICENSE-2.0

* MIT License ->
http://opensource.org/licenses/MIT

  The database files (en_bg-utf8.dat and bg_en-utf8.dat)
are licensed under GNU GPL Version 2 ->
https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html

* See http://kbedic.sourceforge.net/questions.html (in Bulgarian)
";
        button_about.connect_clicked(move |_| {
            trace!("about_event: button About clicked");
            button_add_2_history.set_sensitive(false);
            tree_store.clear();
            right_buff.set_text(&about_text);
            {
                let history_dictdb_locked = history_dictdb.lock().unwrap();
                if history_dictdb_locked.len() > 0 {
                    button_history.set_sensitive(true);
                };
            }
        });
    }

    /// Program History button
    fn history_event(&self) {
        let content = self.content.clone();
        let content_clonned = content.clone();
        let app = self;
        let app_clonned = app.clone();
        let search_entry = content_clonned.s_bar.search_entry.clone();
        let button_history = content_clonned.s_bar.history.clone();
        button_history.connect_clicked(move |_| {
            {
                debug!("history_event: Clear searched_hash");
                let mut searched_hash_locked = app_clonned.searched_hash.lock().unwrap();
                searched_hash_locked.clear();
            }
            {
                let mut history_dictdb_unsorted =
                    app_clonned.history_dictdb_unsorted.lock().unwrap();
                history_dictdb_unsorted.reverse();
                trace!(
                    "history_event: history_data -> {:?}",
                    history_dictdb_unsorted
                );
                app_clonned.selection(&history_dictdb_unsorted, true, false);
            }
            {
                let mut history_dictdb_unsorted =
                    app_clonned.history_dictdb_unsorted.lock().unwrap();
                history_dictdb_unsorted.reverse();
            }
            search_entry.grab_focus();
        });
    }

    /// Program add_2_history button
    fn add_2_history_event(
        &self,
        history_dictdb: Rc<Mutex<Vec<DictDB>>>,
        history_dictdb_unsorted: Rc<Mutex<Vec<DictDB>>>,
    ) {
        let right_buff = self.content.inner_paned.translation.buff.clone();
        let left_selection = self.content.inner_paned.words.tree_view.get_selection();
        let button_history = &self.content.s_bar.history;
        let button_history_clonned = button_history.clone();
        let button_add_2_history = &self.content.s_bar.add_2_history;
        let button_add_2_history_clonned = button_add_2_history.clone();
        let search_entry = &self.content.s_bar.search_entry;
        let search_entry_clonned = search_entry.clone();
        let selection_atomic_isize = self.selection_isize.clone();

        &self.content.s_bar.add_2_history.connect_clicked(move |_| {
            // Get left tree_view selection value
            let left_selection_value: String = match left_selection.get_selected() {
                Some((left_model, iter)) => {
                    match left_model.get_value(&iter, 0).get::<String>() {
                        Some(value_string) => {
                            trace!("add_2_history_event: Selected -> {}", value_string);
                            //button_history_clonned.set_sensitive(true);
                            //button_add_2_history_clonned.set_sensitive(true);
                            value_string
                        }
                        _ => {
                            trace!("add_2_history_event: Warn: left_model.get_value() == _");
                            button_add_2_history_clonned.set_sensitive(false);
                            "empty _".to_string()
                        }
                    }
                }
                None => {
                    trace!("add_2_history_event: Can not get selected. Exit from this method.");
                    button_add_2_history_clonned.set_sensitive(false);
                    "add_2_history_event: Can not get selected".to_string();
                    return;
                }
            };
            // Insert into history_dictdb
            let mut history_dictdb = history_dictdb.lock().unwrap();
            let mut history_dictdb_unsorted = history_dictdb_unsorted.lock().unwrap();
            history_dictdb.sort();
            let search_struct = DictDB {
                word: left_selection_value.clone(),
                translation: "__".to_string(),
            };
            // Search for duplicates and set the button Add
            match history_dictdb.binary_search(&search_struct) {
                Ok(_index) => {
                    trace!("add_2_history_event: bin search OK -> {}", _index);
                    button_add_2_history_clonned.set_sensitive(false);
                }
                Err(_index_err) => {
                    button_history_clonned.set_sensitive(true);
                    button_add_2_history_clonned.set_sensitive(false);
                    let iter1: TextIter = right_buff.get_start_iter();
                    let iter2: TextIter = right_buff.get_end_iter();
                    let right_buff_text = right_buff.get_text(&iter1, &iter2, false).unwrap();
                    // Print to standard out
                    // Warning: these dashes are used for field delimiter in
                    // database.rs::parse_history()
                    println!("------------------------------------------------------------------------------");
                    println!("{}", right_buff_text);
                    let dictdb_entry = DictDB {
                        word: left_selection_value.clone(),
                        translation: right_buff_text.to_string(),
                    };
                    history_dictdb.push(dictdb_entry.clone());
                    history_dictdb.sort();
                    history_dictdb_unsorted.push(dictdb_entry.clone());
                    search_entry_clonned.grab_focus();
                    selection_atomic_isize.store(400, Ordering::SeqCst);
                }
            };
        });
    }

    #[inline(always)]
    fn selection(&self, vec_dict_db: &Vec<DictDB>, history_mode: bool, history_mode_starting: bool) {
        let app1 = self.clone();
        let app2 = self.clone();

        let content = &self.content;
        let content_clonned = content.clone();

        let left_tree = content_clonned.inner_paned.words.tree_view.clone();
        let right_buff = content_clonned.inner_paned.translation.buff.clone();
        let button_add_2_history = content_clonned.s_bar.add_2_history.clone();
        let button_history = content_clonned.s_bar.history.clone();
        // selection
        let left_selection = left_tree.get_selection();
        let left_selection_clonned = left_selection.clone();

        let vec_dict_db_loop: &Vec<DictDB>;
        trace!("selection: history_mode bool == {}; history_mode_starting bool = {}", history_mode, history_mode_starting);
        if history_mode {
            button_history.set_sensitive(false);
            button_add_2_history.set_sensitive(false);
            vec_dict_db_loop = vec_dict_db;
        } else {
            if history_mode_starting {
                button_history.set_sensitive(false);
            } else {
                button_history.set_sensitive(true);
            }
            vec_dict_db_loop = vec_dict_db;
            {
                let history_dictdb_locked = app1.history_dictdb.lock().unwrap();
                if history_dictdb_locked.len() == 0 {
                    button_history.set_sensitive(false);
                };
            }
        }
        // Clear the TreeStore
        content.inner_paned.words.tree_store.clear();

        // Fill searched_hash
        {
            let searched_hash = app1.searched_hash;
            trace!("selection: searched_hash -> MUTEX LOCK before for loop");
            let mut searched_hash_locked = searched_hash.lock().unwrap();

            let mut j = 1;
            debug!(
                "selection: before for loop: searched_hash LEN == {}",
                searched_hash_locked.len()
            );
            for i in vec_dict_db_loop {
                // insert_with_values takes two slices: column indices and ToValue
                // trait objects. ToValue is implemented for strings, numeric types,
                // bool and Object descendants
                content.inner_paned.words.tree_store.insert_with_values(
                    None,
                    None,
                    &[0],
                    &[&format!("{}", &i.word)],
                );
                //trace!("selection: for loop: Insert into searched_hash {} -> {}", i.clone().word, i.clone().translation);
                searched_hash_locked.insert(i.clone().word, i.clone().translation);
                // Set cursor on first item
                if j == 1 {
                    // Selection
                    let path_default = TreePath::new_first();
                    left_selection_clonned.select_path(&path_default);
                    left_tree.set_cursor(&path_default, None, false);
                    right_buff.set_text(&i.translation.as_str());
                    // Search for duplicates into history data and set Add button sensitivity
                    // if history_mode == true -> history_data == vec_dict_db
                    // else  history_data = history_dictdb with mutex
                    let search_struct = DictDB {
                        word: i.clone().word,
                        translation: "___".to_string(),
                    };
                    if history_mode {
                        //let history_data = vec_dict_db;
                        match vec_dict_db.binary_search(&search_struct) {
                            Ok(_index) => {
                                trace!(
                                    "selection: into loop history mode: Ok: {:?}",
                                    vec_dict_db[_index]
                                );
                                button_add_2_history.set_sensitive(false);
                            }
                            Err(_index_err) => {
                                trace!(
                                    "selection: into loop history mode: Err for -> {:?}",
                                    search_struct
                                );
                                button_add_2_history.set_sensitive(true);
                            }
                        };
                    } else {
                        {
                            let history_dictdb = app1.history_dictdb.lock().unwrap();
                            //history_dictdb.sort();
                            debug!("selection: for loop history_len");
                            if history_dictdb.len() == 0 {
                                button_history.set_sensitive(false);
                            }
                            match history_dictdb.binary_search(&search_struct) {
                                Ok(index) => {
                                    trace!("selection: into loop Ok: {:?}", index);
                                    button_add_2_history.set_sensitive(false);
                                }
                                Err(_index_err) => {
                                    trace!("selection: into loop Err for -> {:?}", search_struct);
                                    button_add_2_history.set_sensitive(true);
                                }
                            };
                        }
                    }
                }
                j += 1;
            }
            // Fill searched_hash _END_
        }
        {

        let selection_atomic_isize = self.selection_isize.clone();
        let selection_atomic_isize_1 = selection_atomic_isize.load(Ordering::SeqCst);
        if selection_atomic_isize_1 < -99 {
            trace!(
                "selection: RETURN from method because selection_atomic_isize_1 < -99 -> {}",
                selection_atomic_isize_1
            );
            if history_mode {
                button_add_2_history.set_sensitive(false);
            }
            return;
        } else {
            // On change selected row
            let app = self.clone();
            if history_mode {
                button_add_2_history.set_sensitive(false);
            } else {
                {
                    let history_dictdb = app.history_dictdb.lock().unwrap();
                    debug!("selection: connect_changed: history_len");
                    if history_dictdb.len() == 0 {
                        button_history.set_sensitive(false);
                    }
                }
            }
            let selection_atomic_isize = app.selection_isize.clone();
            {
                left_selection.connect_changed(move |tree_selection| {
                    match tree_selection.get_selected() {
                        Some((left_model, iter)) => {
                            trace!(
                                "selection: connect_changed: ATOMICISIZE {:?}",
                                selection_atomic_isize
                            );
                            match left_model.get_value(&iter, 0).get::<String>() {
                                Some(value_string) => {
                                    let value_string_owned = value_string.to_owned();
                                    let value_str = value_string.as_str();
                                    let mut path =
                                        left_model.get_path(&iter).expect("Couldn't get path");
                                    // get the top-level element path
                                    while path.get_depth() > 1 {
                                        path.up();
                                    }
                                    left_selection_clonned.select_path(&path);
                                    left_tree.set_cursor(&path, None, false);
                                    // Get hash value and write to right_buff
                                    let sai = selection_atomic_isize.load(Ordering::SeqCst);
                                    trace!("selection: connect_changed: searched_hash and value_str == -> {}, sai == {}", value_str, sai);
                                    {
                                        match app2.searched_hash.try_lock() {
                                            Ok(searched_hash_locked_1) => {
                                                trace!("selection: connect_changed: searched_hash MUTEX LOCK before searched_hash");
                                                match searched_hash_locked_1.get(value_str) {
                                                    Some(s) => {
                                                        trace!("selection: connect_changed: Success: Searched hash key -> {}", value_str);
                                                        debug!("selection: connect_changed: searched_hash LEN == {}", searched_hash_locked_1.len());
                                                        right_buff.set_text(&s);
                                                    }
                                                    None => {
                                                        trace!("selection: connect_changed: Warn: Searched hash returned Option is None and value_str -> {}", value_str);
                                                        //right_buff.set_text(value_str);
                                                        right_buff.set_text("");
                                                        {
                                                            let history_dictdb_locked =
                                                                app.history_dictdb.lock().unwrap();
                                                            if history_dictdb_locked.len() == 0 {
                                                                button_history.set_sensitive(false);
                                                            };
                                                        }
                                                        selection_atomic_isize
                                                            .store(-300, Ordering::SeqCst);
                                                    }
                                                };
                                            }
                                            Err(err) => {
                                                debug!("selection: connect_changed: MUTEX LOCK before searched_hash ERROR -> {}", err);
                                            }
                                        };
                                    }
                                    let selection_atomic_isize_2 =
                                        selection_atomic_isize.load(Ordering::SeqCst);
                                    if selection_atomic_isize_2 < -399 {
                                        trace!(
                                            "******** return ******** < -299 -> {}",
                                            selection_atomic_isize_2
                                        );
                                        trace!("selection: connect_changed: RETURN from method because selection_atomic_isize_2 < -99 -> {}", selection_atomic_isize_2);
                                        return;
                                    } else {
                                        let search_struct = DictDB {
                                            word: value_string_owned,
                                            translation: "___".to_string(),
                                        };
                                        if history_mode == false {
                                            trace!("selection: connect_changed: history_mode {} and ATOMICISIZE BEFORE binary_search -> {:?}", history_mode, selection_atomic_isize);
                                            {
                                                let history_dictdb =
                                                    app.history_dictdb.lock().unwrap();
                                                //history_dictdb.sort();
                                                match history_dictdb.binary_search(&search_struct) {
                                                    Ok(_index) => {
                                                        trace!("selection: connect_changed: bin_search Ok: {:?}", history_dictdb[_index]);
                                                        // Workaround
                                                        right_buff.set_text(
                                                            &history_dictdb[_index].translation,
                                                        );
                                                        button_add_2_history.set_sensitive(false);
                                                    }
                                                    Err(_index_err) => {
                                                        trace!("selection: connect_changed: bin_search Err for -> {:?}", search_struct);
                                                        button_add_2_history.set_sensitive(true);
                                                        selection_atomic_isize
                                                            .store(-100, Ordering::SeqCst);
                                                        trace!("selection: connect_changed: ERR ATOMICISIZE AFTER binary_search ->{:?}", selection_atomic_isize);
                                                    }
                                                };
                                            }
                                        } else {
                                            //button_history.set_sensitive(false);
                                            selection_atomic_isize.store(-200, Ordering::SeqCst);
                                        }
                                    }
                                }
                                _ => {
                                    trace!("selection: connect_changed: Warn: left_model.get_value() == _");
                                    button_add_2_history.set_sensitive(true);
                                    selection_atomic_isize.store(1, Ordering::SeqCst);
                                }
                            };
                        }
                        None => {
                            trace!("selection: connect_changed: Can not get selected");
                            button_add_2_history.set_sensitive(false);
                        }
                    };
                });
            }
        }
        // selection _END_
        }
    }
}
