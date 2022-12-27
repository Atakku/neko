// Copyright 2022 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use dioxus_desktop::*;

mod ui;

fn main() {
  dioxus_desktop::launch_cfg(
    ui::app,
    Config::new().with_window(WindowBuilder::new().with_title("neko_bin")),
  );
}
