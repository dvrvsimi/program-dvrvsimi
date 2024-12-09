import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LockdIn } from "../target/types/lockd_in";
import { PublicKey, Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("lock-in", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.LockdIn as Program<LockdIn>;
  
  // Test accounts
  const user = Keypair.generate();
  const assignee = Keypair.generate();

  // Store PDA for todo list
  let todoListPda: PublicKey;

  before(async () => {
    // Airdrop SOL to user for transactions
    const signature = await provider.connection.requestAirdrop(
      user.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);

    // Find PDA for todo list
    const [pda] = await PublicKey.findProgramAddress(
      [
        Buffer.from("user-todo-list"),
        user.publicKey.toBuffer()
      ],
      program.programId
    );
    todoListPda = pda;
  });

  describe("create_todo_task", () => {
    it("should create a new task successfully", async () => {
      const title = "Test Task";
      const description = "Test Description";
      
      await program.methods
        .createTodoTask(
          title,
          description,
          { casual: {} },  // TaskPriority
          { work: {} },    // TaskCategory
          null            // No assignee
        )
        .accounts({
          user: user.publicKey,
          todoList: todoListPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      // Fetch the todo list account and verify
      const todoList = await program.account.userTodoList.fetch(todoListPda);
      expect(todoList.taskCount.toString()).to.equal("1");
      expect(todoList.tasks[0].title).to.equal(title);
      expect(todoList.tasks[0].description).to.equal(description);
      expect(todoList.tasks[0].creator.toString()).to.equal(user.publicKey.toString());
    });

    it("should fail with invalid title", async () => {
      try {
        await program.methods
          .createTodoTask(
            "",  // Empty title
            "Description",
            { casual: {} },
            { work: {} },
            null
          )
          .accounts({
            user: user.publicKey,
            todoList: todoListPda,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();
        expect.fail("Expected to throw InvalidTitle error");
      } catch (error: any) {
        const errorMsg = error.error?.message || error.message;
        expect(errorMsg).to.include("Invalid task title or description");
      }
    });
  });

  describe("reassign_task", () => {
    it("should reassign a task to new assignee", async () => {
      const newAssignee = Keypair.generate();
      const initAssignee = Keypair.generate();

      // First create a task with initial assignee
      await program.methods
        .createTodoTask(
          "Task to Reassign",
          "Description",
          { urgent: {} },
          { work: {} },
          initAssignee.publicKey
        )
        .accounts({
          user: user.publicKey,
          todoList: todoListPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      // Get task ID
      const todoListBefore = await program.account.userTodoList.fetch(todoListPda);
      const taskId = todoListBefore.taskCount.subn(1);

      // Verify initial and new assignee are different
      expect(initAssignee.publicKey.toString()).to.not.equal(newAssignee.publicKey.toString());

      // Reassign task
      await program.methods
        .reassignTask(
          taskId,
          newAssignee.publicKey
        )
        .accounts({
          creator: user.publicKey,
          todoList: todoListPda,
          assignee: newAssignee.publicKey,
        })
        .signers([user])
        .rpc();

      // Verify the reassignment
      const todoListAfter = await program.account.userTodoList.fetch(todoListPda);
      const task = todoListAfter.tasks[todoListAfter.tasks.length - 1];
      
      expect(task.assignee?.toString()).to.equal(newAssignee.publicKey.toString());
    });
  });

  describe("update_task_status", () => {
    let taskId: anchor.BN;

    beforeEach(async () => {
      // Create a new task for status updates
      await program.methods
        .createTodoTask(
          "Status Test Task",
          "Description",
          { casual: {} },
          { work: {} },
          null
        )
        .accounts({
          user: user.publicKey,
          todoList: todoListPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const todoList = await program.account.userTodoList.fetch(todoListPda);
      taskId = todoList.taskCount.subn(1);
    });

    it("should update task status to in progress", async () => {
      await program.methods
        .updateTaskStatus(
          taskId,
          { inProgress: {} }
        )
        .accounts({
          user: user.publicKey,
          todoList: todoListPda,
        })
        .signers([user])
        .rpc();

      // Verify status update
      const todoList = await program.account.userTodoList.fetch(todoListPda);
      const task = todoList.tasks[todoList.tasks.length - 1];
      expect(JSON.stringify(task.status)).to.equal(JSON.stringify({ inProgress: {} }));
    });

    it("should update task status to completed", async () => {
      await program.methods
        .updateTaskStatus(
          taskId,
          { completed: {} }
        )
        .accounts({
          user: user.publicKey,
          todoList: todoListPda,
        })
        .signers([user])
        .rpc();

      // Verify status update
      const todoList = await program.account.userTodoList.fetch(todoListPda);
      const task = todoList.tasks[todoList.tasks.length - 1];
      expect(JSON.stringify(task.status)).to.equal(JSON.stringify({ completed: {} }));
    });
  });
});
