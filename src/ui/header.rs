// Copyright 2018 Ivan Dzhonev
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use gtk::{HeaderBar, HeaderBarExt, Button, WidgetExt};

#[derive(Clone)]
pub struct Header {
    pub container:  HeaderBar,
    pub about:      Button,
}

impl Header {
    pub fn new() -> Header {
        // Creates the main header bar container widget.
        let container = HeaderBar::new();
        let about = Button::new_with_mnemonic("_About");

        about.set_tooltip_text("About RBEdic.");

        // Sets the text to display in the title section of the header bar.
        container.set_title("RBEdic");
        // Enable the window controls within this headerbar.
        container.set_show_close_button(true);
        container.pack_start(&about);

        // Returns the header and all of it's state
        Header { container, about }
    }
}
