// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use tokio_cron_scheduler::{Job, JobScheduler};

module! {
  Cron {
    jobs: Vec<Job>
  }

  fn init(fw) {
    runtime!(fw, |cron| {
      let sched = JobScheduler::new().await?;
        for job in cron.jobs {
          sched.add(job).await?;
        }
        sched.start().await?;
      Ok(None)
    });
  }

  pub fn add_job(&mut self, job: Job) {
    self.jobs.push(job);
  }
}

macro_rules! job {
  ($fw:ident, $time:literal, $block:block) => {
    $fw.req::<Cron>()?.add_job(Job::new_async($time, |_id, _jsl| Box::pin(async move $block))?)
  };
}
