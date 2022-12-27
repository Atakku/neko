// Copyright 2022 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_router::{Router, Route, Link};

pub fn app(cx: Scope) -> Element {
  cx.render(rsx! {
    Router {
      Menubar {}
      Route { to: "/", Home {} },
      Route { to: "/test", Test {} },
      Route { to: "/test2", Test2 {} },
      Route { to: "", NotFound {} }
      Footer {}
    }
  })
}
fn Home(cx: Scope) -> Element {
  cx.render(rsx! {
    p {
      "homepage"
    }

    Link {
      to: "/test",
      "test"
    }
    br {}
    Link {
      to: "/test2",
      "test2"
    }
  })
}

fn Test(cx: Scope) -> Element {
  cx.render(rsx! {
    p {
      "test"
    }
  })
}
fn Test2(cx: Scope) -> Element {
  cx.render(rsx! {
    p {
      "test 2: electric boogaloo"
    }
  })
}

fn NotFound(cx: Scope) -> Element {
  cx.render(rsx! {
    p {
      "404 lmao"
    }
  })
}

fn Menubar(cx: Scope) -> Element {
  cx.render(rsx! {
    p {
      "wip menubar"
    }
  })
}

fn Footer(cx: Scope) -> Element {
  cx.render(rsx! {
    p {
      "this is a footer"
    }
    Link {
      to: "/",
      "click here to go to main page"
    }
  })
}