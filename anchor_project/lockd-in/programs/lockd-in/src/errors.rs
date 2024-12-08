use anchor_lang::prelude::*;

#[error_code]
pub enum TodoError {
    #[msg("Invalid task title or description")]
    InvalidTitle,

    #[msg("Task already completed")]
    TaskAlreadyCompleted,

    #[msg("Unauthorized task modification")]
    UnauthorizedModification,

    #[msg("Invalid task priority")]
    InvalidPriority,

    #[msg("Maximum tasks limit reached")]
    MaxTasksLimitReached,

    #[msg("Invalid task status transition")]
    InvalidStatusTransition,

    #[msg("Task not found")]
    TaskNotFound,
}