// copyright 2017 Kaz Wesley

use core_affinity::{self, CoreId};
use hasher::{self, HasherBuilder};
use job::CpuId;
use poolclient::WorkSource;
use workgroup::stats::StatUpdater;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    cpu: CpuId,
    hasher: hasher::Config,
}

pub struct Worker {
    worksource: WorkSource,
    stat_updater: StatUpdater,
}

impl Worker {
    pub fn new(worksource: WorkSource, stat_updater: StatUpdater) -> Self {
        Worker {
            worksource,
            stat_updater,
        }
    }

    pub fn run(mut self, cfg: Config, hasher_builder: HasherBuilder, core_ids: Vec<CoreId>) -> ! {
        // TODO: CoreId error handling
        core_affinity::set_for_current(core_ids[cfg.cpu.0 as usize]);
        let base_nonce = cfg.cpu.into();
        let mut hasher = hasher_builder.into_hasher(&cfg.hasher, base_nonce);
        let mut job = self.worksource.get_new_work().unwrap();
        loop {
            let mut hashes = hasher.hashes(job);
            job = loop {
                for (nonce, hash) in hashes.by_ref().take(16) {
                    self.stat_updater.inc_hashes();
                    self.worksource.result(nonce, &hash).unwrap();
                }

                if let Some(new_job) = self.worksource.get_new_work() {
                    break new_job;
                }
            }
        }
    }
}
