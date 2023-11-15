// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::{Res, R};
use futures::future::{join_all, LocalBoxFuture};
use std::{
  any::{type_name, Any, TypeId},
  collections::HashMap,
  hash::{BuildHasherDefault, Hasher},
};
use tokio::task::JoinHandle;

pub type RuntimeClosure =
  fn(&mut ModuleMap) -> Res<LocalBoxFuture<'static, Res<Option<JoinHandle<R>>>>>;

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

pub type ModuleMap = HashMap<TypeId, Box<dyn Any>, BuildHasherDefault<IdHasher>>;
pub struct ModuleFramework {
  pub modules: ModuleMap,
  pub runtime: Vec<RuntimeClosure>,
}

pub trait Module: Any {
  fn init(&mut self, fw: &mut ModuleFramework) -> R;
}

impl ModuleFramework {
  pub fn new() -> Self {
    Self {
      modules: HashMap::default(),
      runtime: vec![],
    }
  }

  pub async fn run(mut self) -> R {
    let mut handles = vec![];
    // Run all async mains and collect any handles
    for run in self.runtime {
      if let Some(handle) = run(&mut self.modules)?.await? {
        handles.push(handle);
      }
    }
    // Await any returned handles
    for res in join_all(handles).await {
      res??
    }
    Ok(())
  }

  pub fn has<T: Module>(&mut self) -> bool {
    self.modules.get(&TypeId::of::<T>()).is_some()
  }

  /// Load a supplied module
  pub fn init<T: Module>(&mut self, mut module: T) -> Res<&mut Self> {
    log::info!("Initializing {}", std::any::type_name::<T>());
    module.init(self)?;
    self.modules.insert(TypeId::of::<T>(), Box::new(module));
    Ok(self)
  }

  /// Check if module is already loaded, and if not, load a default impl
  pub fn req<T: Module + Default>(&mut self) -> Res<&mut T> {
    if !self.has::<T>() {
      self.init(T::default())?;
    }

    Ok(
      self
        .modules
        .get_mut(&TypeId::of::<T>())
        .and_then(|b| b.downcast_mut::<T>())
        .ok_or(format!(
          "Required module {} is not loaded",
          type_name::<T>()
        ))?,
    )
  }

  pub fn take<T: Module>(modules: &mut ModuleMap) -> Res<T> {
    Ok(
        modules
        .remove(&TypeId::of::<T>())
        .and_then(|b| b.downcast::<T>().ok())
        .map(|b| *b)
        .ok_or(format!(
          "Required module {} is not loaded",
          type_name::<T>()
        ))?,
    )
  }
}

macro_rules! first {
  {$first:expr, $other:expr} => { $first };
  {,$other:expr} => { $other };
}

macro_rules! module {
  (@internal $name:ident, $fw:ident, $block:block) => {
    impl $crate::core::Module for $name {
      fn init(&mut self, $fw: &mut $crate::core::ModuleFramework) -> $crate::core::R {
        $block
        Ok(())
      }
    }
  };
  ($(#[$m:meta])* $name:ident; fn init($fw:ident) $block:block) => {
    $(#[$m])*
    #[derive(Default)]
    pub struct $name;
    module!(@internal $name, $fw, $block);
  };
  ($(#[$m:meta])* $name:ident {$($pv:vis $pn:ident: $pt:ty $(= $pd:expr)?),*$(,)?}
    fn init($fw:ident) $block:block

    $($fn_vis:vis fn $fn_name:ident$(<$($fn_gn:tt$(: $fn_gt:tt)?),*>)?($($fn_tt:tt)*) $fn_block:block)*
  ) => {
    $(#[$m])*
    pub struct $name {
      $($pv $pn: $pt),*
    }

    impl Default for $name {
      fn default() -> Self {
        Self {
          $($pn: first!($($pd)?, Default::default())),*
        }
      }
    }

    module!(@internal $name, $fw, $block);

    impl $name {
      $($fn_vis fn $fn_name$(<$($fn_gn$(: $fn_gt)?),*>)?($($fn_tt)*) $fn_block)*
    }
  };
}
