use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anime_game_core::game::GameExt;
use anime_game_core::game::diff::DiffExt;
use anime_game_core::game::genshin::diff::{Diff, Updater, Status};
use anime_game_core::game::genshin::{Game, Edition};

use anime_game_core::filesystem::DriverExt;
use anime_game_core::updater::UpdaterExt;

use crate::ui::components::game_card::CardVariant;
use crate::ui::components::tasks_queue::{QueuedTask, ResolvedTask, TaskStatus};

use super::RunGameExt;

pub struct Genshin {
    driver: Arc<dyn DriverExt>,
    edition: Edition
}

impl From<&Game> for Genshin {
    #[inline]
    fn from(game: &Game) -> Self {
        Self {
            driver: game.get_driver(),
            edition: game.get_edition()
        }
    }
}

impl RunGameExt for Genshin {
    #[inline]
    fn get_game_binary(&self) -> &'static str {
        match self.edition {
            Edition::Global => "GenshinImpact.exe",
            Edition::China  => "YuanShen.exe"
        }
    }

    #[inline]
    fn deploy_game_folder(&self) -> anyhow::Result<PathBuf> {
        Ok(self.driver.deploy()?)
    }

    #[inline]
    fn dismantle_game_folder(&self) -> anyhow::Result<()> {
        Ok(self.driver.dismantle()?)
    }

    #[inline]
    fn get_user_environment(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub struct DownloadDiffQueuedTask {
    pub diff: Diff
}

impl std::fmt::Debug for DownloadDiffQueuedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadDiffQueuedTask").finish()
    }
}

impl QueuedTask for DownloadDiffQueuedTask {
    fn get_variant(&self) -> CardVariant {
        CardVariant::Genshin
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let Some(updater) = self.diff.install() else {
            anyhow::bail!("Queued genshin diff cannot be installed");
        };

        Ok(Box::new(DownloadDiffResolvedTask {
            updater
        }))
    }
}

pub struct DownloadDiffResolvedTask {
    pub updater: Updater
}

impl std::fmt::Debug for DownloadDiffResolvedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadDiffResolvedTask").finish()
    }
}

impl ResolvedTask for DownloadDiffResolvedTask {
    fn get_variant(&self) -> CardVariant {
        CardVariant::Genshin
    }

    fn is_finished(&mut self) -> bool {
        self.updater.is_finished()
    }

    fn get_current(&self) -> u64 {
        self.updater.current()
    }

    fn get_total(&self) -> u64 {
        self.updater.total()
    }

    fn get_progress(&self) -> f64 {
        self.updater.progress()
    }

    fn get_status(&mut self) -> anyhow::Result<TaskStatus> {
        match self.updater.status() {
            Ok(status) => Ok(match status {
                Status::PreparingTransition   => TaskStatus::PreparingTransition,
                Status::Downloading           => TaskStatus::Downloading,
                Status::Unpacking             => TaskStatus::Unpacking,
                Status::FinishingTransition   => TaskStatus::FinishingTransition,
                Status::ApplyingHdiffPatches  => TaskStatus::ApplyingHdiffPatches,
                Status::DeletingObsoleteFiles => TaskStatus::DeletingObsoleteFiles,
                Status::Finished              => TaskStatus::Finished
            }),

            Err(err) => anyhow::bail!(err.to_string())
        }
    }
}