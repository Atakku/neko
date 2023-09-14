// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![feature(trait_alias)]

use std::error::Error;

pub mod framework;
pub use framework::*;
pub mod module;
pub use module::*;
mod state;

pub type Err = Box<dyn Error + Send + Sync>;
pub type Res<T> = Result<T, Err>;
pub type R = Res<()>;
