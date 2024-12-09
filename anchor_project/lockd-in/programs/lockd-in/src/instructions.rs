use anchor_lang::prelude::*;
use crate::errors::TodoError;
use crate::state::{Task, TaskPriority, TaskStatus, TaskCategory, UserTodoList};

#[derive(Accounts)]
pub struct CreateTask<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init_if_needed,
        seeds = [b"user-todo-list", user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<UserTodoList>() + 1024
    )]
    pub todo_list: Account<'info, UserTodoList>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTaskStatus<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"user-todo-list", user.key().as_ref()],
        bump,
    )]
    pub todo_list: Account<'info, UserTodoList>,
}

#[derive(Accounts)]
pub struct AssignTask<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"user-todo-list", creator.key().as_ref()],
        bump,
    )]
    pub todo_list: Account<'info, UserTodoList>,

    /// CHECK: Validated in handler
    pub assignee: UncheckedAccount<'info>,
}

pub fn create_task(
    ctx: Context<CreateTask>, 
    title: String, 
    description: String,
    priority: TaskPriority,
    category: TaskCategory,
    assignee: Option<Pubkey>
) -> Result<()> {
    require!(title.len() <= Task::MAX_TITLE_LENGTH, TodoError::InvalidTitle);
    require!(description.len() <= Task::MAX_DESCRIPTION_LENGTH, TodoError::InvalidTitle);
    require!(ctx.accounts.todo_list.tasks.len() < Task::MAX_TASKS, TodoError::MaxTasksLimitReached);

    let todo_list = &mut ctx.accounts.todo_list;
    let task_id = todo_list.task_count.checked_add(1)
        .ok_or(TodoError::MaxTasksLimitReached)?;

    let new_task = Task::new(
        task_id,
        title,
        description,
        ctx.accounts.user.key(),
        assignee,
        priority,
        category
    );

    todo_list.tasks.push(new_task);
    todo_list.task_count += 1;

    Ok(())
}

pub fn update_task_status(
    ctx: Context<UpdateTaskStatus>,
    task_id: u64,
    new_status: TaskStatus,
) -> Result<()> {
    let todo_list = &mut ctx.accounts.todo_list;
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    let task = todo_list.tasks.iter_mut()
        .find(|t| t.id == task_id)
        .ok_or(TodoError::TaskNotFound)?;

    require!(
        task.creator == ctx.accounts.user.key() || 
        task.assignee == Some(ctx.accounts.user.key()),
        TodoError::UnauthorizedModification
    );

    match task.status {
        TaskStatus::Completed => {
            return Err(TodoError::TaskAlreadyCompleted.into())
        }
        TaskStatus::Cancelled => {
            require!(
                new_status == TaskStatus::InProgress,
                TodoError::InvalidStatusTransition
            );
        }
        _ => {}
    }

    task.status = new_status;
    task.updated_at = current_timestamp;

    if new_status == TaskStatus::Completed {
        task.completed_at = Some(current_timestamp);
        
        let today = current_timestamp / 86400;
        if let Some(last_completed) = todo_list.last_completed_date {
            let last_completed_day = last_completed / 86400;
            if last_completed_day == today - 1 {
                todo_list.completed_task_streak += 1;
            } else {
                todo_list.completed_task_streak = 1;
            }
        } else {
            todo_list.completed_task_streak = 1;
        }
        todo_list.last_completed_date = Some(current_timestamp);
    }

    Ok(())
}

pub fn assign_task(
    ctx: Context<AssignTask>, 
    task_id: u64, 
    new_assignee: Pubkey
) -> Result<()> {
    let todo_list = &mut ctx.accounts.todo_list;
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    let task = todo_list.tasks.iter_mut()
        .find(|t| t.id == task_id)
        .ok_or(TodoError::UnauthorizedModification)?;

    require!(task.creator == ctx.accounts.creator.key(), TodoError::UnauthorizedModification);

    task.assignee = Some(new_assignee);
    task.updated_at = current_timestamp;

    Ok(())
}