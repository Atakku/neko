// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::Res;
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

pub trait AnyData = Any;
pub trait SyncData = Any + Sync + Send;

pub struct State<T: ?Sized> {
  data: HashMap<TypeId, Box<T>, BuildHasherDefault<IdHasher>>,
}

impl<T: ?Sized> State<T> {
  pub fn new() -> State<T> {
    State {
      data: HashMap::default(),
    }
  }
}

macro_rules! impl_state {
  ($T:ident) => {
    impl State<dyn $T> {
      pub fn put<T: $T>(&mut self, t: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(t));
      }

      pub fn has<T: $T>(&self) -> bool {
        self.data.get(&TypeId::of::<T>()).is_some()
      }

      pub fn try_borrow<T: $T>(&self) -> Option<&T> {
        self
          .data
          .get(&TypeId::of::<T>())
          .and_then(|b| b.downcast_ref::<T>())
      }

      pub fn borrow<T: $T>(&self) -> Res<&T> {
        Ok(self.try_borrow().ok_or(format!(
          "Required type {} is not present in State container",
          type_name::<T>()
        ))?)
      }

      pub fn borrow_or_default<T: $T + Default>(&mut self) -> Res<&T> {
        if !self.has::<T>() {
          self.put(T::default());
        }
        self.borrow()
      }

      pub fn try_borrow_mut<T: $T>(&mut self) -> Option<&mut T> {
        self
          .data
          .get_mut(&TypeId::of::<T>())
          .and_then(|b| b.downcast_mut::<T>())
      }

      pub fn borrow_mut<T: $T>(&mut self) -> Res<&mut T> {
        Ok(self.try_borrow_mut().ok_or(format!(
          "Required type {} is not present in State container",
          type_name::<T>()
        ))?)
      }

      pub fn get_mut_or_default<T: $T + Default>(&mut self) -> Res<&mut T> {
        if !self.has::<T>() {
          self.put(T::default());
        }
        self.borrow_mut()
      }

      pub fn try_take<T: $T>(&mut self) -> Option<T> {
        self
          .data
          .remove(&TypeId::of::<T>())
          .and_then(|b| b.downcast::<T>().ok())
          .map(|b| *b)
      }

      pub fn take<T: $T>(&mut self) -> Res<T> {
        Ok(self.try_take().ok_or(format!(
          "Required type {} is not present in State container",
          type_name::<T>()
        ))?)
      }
    }
  };
}

impl_state!(AnyData);
impl_state!(SyncData);
