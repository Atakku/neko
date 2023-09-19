// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![feature(trait_alias)]
#![feature(async_fn_in_trait)]

#[macro_use]
pub mod macros;

pub mod core;

pub mod interface {
  automod::dir!(pub "src/interface");
}
pub mod modules {
  automod::dir!(pub "src/modules");
}
pub mod query {
  automod::dir!(pub "src/query");
}
pub mod schema {
  automod::dir!(pub "src/schema");
}
