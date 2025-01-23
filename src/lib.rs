// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![feature(trait_alias)]
#![feature(sort_floats)]

#[macro_use]
pub mod macros;

pub mod core;

pub mod modules {
  automod::dir!(pub "src/modules");
}
pub mod plugins {
  #[path="atakku/plugin.rs"]
  pub mod atakku;
  #[path="beatleader/plugin.rs"]
  pub mod beatleader;
  #[path="discord/plugin.rs"]
  pub mod discord;
  #[path="drg/plugin.rs"]
  pub mod drg;
  //#[path="ftvroles/plugin.rs"]
  //pub mod ftvroles;
  #[path="gwaaa/plugin.rs"]
  pub mod gwaaa;
  #[path="neko/plugin.rs"]
  pub mod neko;
  //#[path="radio/plugin.rs"]
  //pub mod radio;
  #[path="starboard/plugin.rs"]
  pub mod starboard;
  #[path="steam/plugin.rs"]
  pub mod steam;
  //#[path="warnsys/plugin.rs"]
  //pub mod warnsys;
  #[path="welcomer/plugin.rs"]
  pub mod welcomer;
}
