// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Default)]
pub struct Cron {
  pub jobs: Vec<Job>
}

impl Module for Cron {
  async fn init(&mut self, fw: &mut Framework) -> R {
    fw.runtime.push(|mds| {
      let cron = mds.take::<Self>()?;
      Ok(Box::pin(async move {
        let sched = JobScheduler::new().await?;
        for job in cron.jobs {
          sched.add(job).await?;
        }
        sched.start().await?;
        Ok(None)
      }))
    });
    Ok(())
  }
}
