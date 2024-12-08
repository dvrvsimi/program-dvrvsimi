use anchor_lang::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum TaskPriority {
    Leisure,
    Casual,
    Urgent,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Casual
    }
}

#[derive(Debug, Clone, Copy, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

#[derive(Debug, Clone, Copy, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum TaskCategory {
    Work,
    Personal,
    Home,
    Shopping,
}

impl Default for TaskCategory {
    fn default() -> Self {
        TaskCategory::Personal
    }
}

#[account]
#[derive(Debug, Default)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub creator: Pubkey,
    pub assignee: Option<Pubkey>,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub category: TaskCategory,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}

impl Task {
    pub const MAX_TITLE_LENGTH: usize = 50;
    pub const MAX_DESCRIPTION_LENGTH: usize = 250;
    pub const MAX_TASKS: usize = 100;

    pub fn new(
        id: u64,
        title: String,
        description: String,
        creator: Pubkey,
        assignee: Option<Pubkey>,
        priority: TaskPriority,
        category: TaskCategory, 
    ) -> Self {
        let now = Clock::get().unwrap().unix_timestamp;
        Self {
            id,
            title,
            description,
            creator,
            assignee,
            priority,
            status: TaskStatus::Pending,
            category,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }
}

#[account]
#[derive(Debug, Default)]
pub struct UserTodoList {
    pub owner: Pubkey,
    pub tasks: Vec<Task>,
    pub task_count: u64,
    pub completed_task_streak: u64,
    pub last_completed_date: Option<i64>,
    pub bump: u8,
}

impl UserTodoList {
    pub const SEED: &'static str = "user-todo-list";
    
    pub fn new(owner: Pubkey, bump: u8) -> Self {
        Self {
            owner,
            tasks: Vec::new(),
            task_count: 0,
            completed_task_streak: 0,
            last_completed_date: None,
            bump,
        }
    }
}