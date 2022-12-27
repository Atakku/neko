// Copyright 2022 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![allow(non_snake_case)]
use dioxus::prelude::*;

pub fn app(cx: Scope) -> Element {
  cx.render(rsx! {
    Menubar {}
      div {
        "ayo another div?"
      }
  })
}

fn Menubar(cx: Scope) -> Element {
  cx.render(rsx! {
    div {
      "menubar moment"
    }
  })
}
