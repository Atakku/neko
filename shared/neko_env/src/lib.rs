// Copyright 2022 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::env;

#[allow(unused_imports)]
use lazy_static::lazy_static;

#[cfg(feature = "http")]
lazy_static! {
  pub static ref HTTP_PORT: String = env_or("HTTP_PORT", "8080");
}

#[allow(dead_code)]
fn env_or(var: &str, default: &str) -> String {
  env::var(&var).unwrap_or(default.into())
}
