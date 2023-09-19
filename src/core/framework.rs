// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::{
  state::{AnyData, State, SyncData},
  Res, R,
};
use futures::future::{join_all, LocalBoxFuture};
use futures_locks::RwLock;
use tokio::task::JoinHandle;

pub type ModuleState = State<dyn AnyData>;
pub type RuntimeState = State<dyn SyncData>;

pub type StateLock = RwLock<RuntimeState>;
pub type RuntimeClosure =
  fn(&mut ModuleState, StateLock) -> Res<LocalBoxFuture<'static, Res<Option<JoinHandle<R>>>>>;

pub struct Framework {
  pub modules: ModuleState,
  pub state: RuntimeState,
  pub runtime: Vec<RuntimeClosure>,
}

impl Framework {
  pub fn new() -> Self {
    Self {
      modules: State::new(),
      state: State::new(),
      runtime: vec![],
    }
  }

  pub async fn run(mut self) -> R {
    let lock = RwLock::new(self.state);
    let mut handles = vec![];
    // Run all async mains and collect any handles
    for run in self.runtime {
      if let Some(handle) = run(&mut self.modules, lock.clone())?.await? {
        handles.push(handle);
      }
    }
    // Await any returned handles
    for res in join_all(handles).await {
      res??
    }
    Ok(())
  }
}
