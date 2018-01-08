// Copyright 2018 Ivan Dzhonev
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use gtk::{Box, SearchEntry, Button, Paned, ScrolledWindow, TreeView, TreeStore, TextView, TextBuffer,
            Orientation, WrapMode, TreeViewColumn, CellRendererText, ContainerExt, TextViewExt,
            WidgetExt, EntryExt, PanedExt, StaticType, TreeViewExt, CellLayoutExt, StyleContextExt};

#[derive(Clone)]
pub struct Content {
    pub container:      Box,
    pub s_bar:          SBar,
    pub inner_paned:    InnerPaned,
}

#[derive(Clone)]
pub struct SBar {
    pub container:      Box,
    pub search_entry:   SearchEntry,
    pub history:        Button,
    pub add_2_history:  Button,
}

#[derive(Clone)]
pub struct InnerPaned {
    pub container:      Paned,
    pub words:          Words,
    pub translation:    Translation,
}

#[derive(Clone)]
pub struct Words {
    pub container:      ScrolledWindow,
    pub tree_view:      TreeView,
    pub tree_store:     TreeStore,
}

#[derive(Clone)]
pub struct Translation {
    pub container:  ScrolledWindow,
    pub text_view:  TextView,
    pub buff:       TextBuffer,
}

impl Content {
    pub fn new() -> Content {
        // Create The Box.
        let container = Box::new(Orientation::Vertical, 0);
        let s_bar = SBar::new();
        let inner_paned = InnerPaned::new();

        container.add(&s_bar.container);
        container.add(&inner_paned.container);
        container.set_vexpand(true);

        Content { container, s_bar, inner_paned }
    }
}

impl SBar {
    pub fn new() -> SBar {
        // Create The Sbar
        let container = Box::new(Orientation::Horizontal, 0);
        let search_entry = SearchEntry::new();
        let history = Button::new_with_mnemonic("_History");
        history.set_tooltip_text("View history. CTRL+d");
        let add_2_history = Button::new_with_mnemonic("_Add");
        add_2_history.get_style_context().map(|x| x.add_class("suggested-action"));
        add_2_history.set_tooltip_text("Add word to history. CTRL+s");

        container.set_hexpand(true);
        search_entry.set_hexpand(true);
        search_entry.set_max_length(50);
        container.add(&search_entry);
        container.add(&history);
        container.add(&add_2_history);

        SBar { container, search_entry, history, add_2_history }
    }
}

impl InnerPaned {
    pub fn new() -> InnerPaned {
        // Create The Paned container for the main content.
        let container = Paned::new(Orientation::Horizontal);
        let words = Words::new();
        let translation = Translation::new();

        // Pack it in
        container.pack1(&words.container, true, true);
        container.pack2(&translation.container, true, true);

        words.container.set_size_request(50, -1);
        translation.container.set_size_request(100, -1);
        container.set_vexpand(true);

        InnerPaned { container, words, translation }
    }
}

impl Words {
    pub fn new() -> Words {
        // Create TreeView on the left pane
        let tree_view = TreeView::new();
        let tree_store = TreeStore::new(&[String::static_type()]);
        let container = ScrolledWindow::new(None, None);

        tree_view.set_model(Some(&tree_store));
        tree_view.set_headers_visible(false);
        append_text_column(&tree_view);
        container.add(&tree_view);

        Words { container, tree_view, tree_store }
    }
}

impl Translation {
    pub fn new() -> Translation {
        // Create TextView on the right pane
        let buff = TextBuffer::new(None);
        let text_view = TextView::new_with_buffer(&buff);
        text_view.set_editable(false);
        text_view.set_wrap_mode(WrapMode::Word);
        text_view.set_right_margin(10);
        text_view.set_left_margin(10);

        let container = ScrolledWindow::new(None, None);
        container.add(&text_view);

        Translation { container, text_view, buff }
    }
}

fn append_text_column(tree: &TreeView) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    tree.append_column(&column);
}
