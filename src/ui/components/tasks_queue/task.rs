use crate::ui::components::game_card::CardVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// All the possible tasks statuses in one enum
pub enum TaskStatus {
    PreparingTransition,
    Downloading,
    Unpacking,
    FinishingTransition,
    ApplyingHdiffPatches,
    DeletingObsoleteFiles,
    CreatingPrefix,
    InstallingFonts,
    Finished
}

pub trait QueuedTask: Send + std::fmt::Debug {
    /// Get component variant
    fn get_variant(&self) -> CardVariant;

    /// Get tasked component title
    fn get_title(&self) -> String {
        self.get_variant().get_title().to_owned()
    }

    /// Get tasked component author
    fn get_author(&self) -> String {
        self.get_variant().get_author().to_owned()
    }

    /// Resolve queued task and start downloading stuff
    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>>;
}

pub trait ResolvedTask: Send + std::fmt::Debug {
    /// Get component variant
    fn get_variant(&self) -> CardVariant;

    /// Get tasked component title
    fn get_title(&self) -> String {
        self.get_variant().get_title().to_owned()
    }

    /// Get tasked component author
    fn get_author(&self) -> String {
        self.get_variant().get_author().to_owned()
    }

    /// Check if the task is finished
    fn is_finished(&mut self) -> bool;

    /// Get current task progress
    fn get_current(&self) -> u64;

    /// Get total task progress
    fn get_total(&self) -> u64;

    /// Get task completion progress
    fn get_progress(&self) -> f64;

    /// Get task status
    fn get_status(&mut self) -> anyhow::Result<TaskStatus>;
}