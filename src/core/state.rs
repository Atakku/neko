// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::Res;
use std::{
  any::{type_name, Any, TypeId},
  collections::HashMap,
  hash::{BuildHasherDefault, Hasher},
};

// Code taken from https://github.com/gotham-rs/gotham/blob/main/gotham/src/state/mod.rs
#[derive(Default)]
pub struct IdHasher(u64);

impl Hasher for IdHasher {
  fn write(&mut self, _: &[u8]) {
    unreachable!("TypeId calls write_u64");
  }

  #[inline]
  fn write_u64(&mut self, id: u64) {
    self.0 = id;
  }

  #[inline]
  fn finish(&self) -> u64 {
    self.0
  }
}

pub struct State {
  data: HashMap<TypeId, Box<dyn Any>, BuildHasherDefault<IdHasher>>,
}

impl State {
  pub fn new() -> State {
    State {
      data: HashMap::default(),
    }
  }
  pub fn put<T: Any>(&mut self, t: T) {
    self.data.insert(TypeId::of::<T>(), Box::new(t));
  }

  pub fn has<T: Any>(&self) -> bool {
    self.data.get(&TypeId::of::<T>()).is_some()
  }

  pub fn try_borrow<T: Any>(&self) -> Option<&T> {
    self
      .data
      .get(&TypeId::of::<T>())
      .and_then(|b| b.downcast_ref::<T>())
  }

  pub fn borrow<T: Any>(&self) -> Res<&T> {
    Ok(self.try_borrow().ok_or(format!(
      "Required type {} is not present in State container",
      type_name::<T>()
    ))?)
  }

  pub fn borrow_or_default<T: Any + Default>(&mut self) -> Res<&T> {
    if !self.has::<T>() {
      self.put(T::default());
    }
    self.borrow()
  }

  pub fn try_borrow_mut<T: Any>(&mut self) -> Option<&mut T> {
    self
      .data
      .get_mut(&TypeId::of::<T>())
      .and_then(|b| b.downcast_mut::<T>())
  }

  pub fn borrow_mut<T: Any>(&mut self) -> Res<&mut T> {
    Ok(self.try_borrow_mut().ok_or(format!(
      "Required type {} is not present in State container",
      type_name::<T>()
    ))?)
  }

  pub fn get_mut_or_default<T: Any + Default>(&mut self) -> Res<&mut T> {
    if !self.has::<T>() {
      self.put(T::default());
    }
    self.borrow_mut()
  }

  pub fn try_take<T: Any>(&mut self) -> Option<T> {
    self
      .data
      .remove(&TypeId::of::<T>())
      .and_then(|b| b.downcast::<T>().ok())
      .map(|b| *b)
  }

  pub fn take<T: Any>(&mut self) -> Res<T> {
    Ok(self.try_take().ok_or(format!(
      "Required type {} is not present in State container",
      type_name::<T>()
    ))?)
  }
}
