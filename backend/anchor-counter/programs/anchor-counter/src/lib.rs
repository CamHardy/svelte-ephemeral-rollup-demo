use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::{commit, delegate, DelegationProgram, MagicProgram};
use ephemeral_rollups_sdk::cpi::delegate_account;
use ephemeral_rollups_sdk::ephem::{commit_accounts, commit_and_undelegate_accounts};

declare_id!("7j3CNNHDtgrzPyyWaHREzxP7aj8Tqz9A4irQcQdz6rGq");

pub const TEST_PDA_SEED: &[u8] = b"test-pda";

#[delegate]
#[program]
pub mod anchor_counter {
  use super::*;

  // initialize the counter
  pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.count = 0;
    Ok(())
  }

  // increment the counter
  pub fn increment(ctx: Context<Increment>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.count += 1;
    Ok(())
  }

  // delegate the counter to the delegation program
  pub fn delegate(ctx: Context<DelegateInput>) -> Result<()> {
    let pda_seeds: &[&[u8]] = &[TEST_PDA_SEED];

    delegate_account(
      &ctx.accounts.payer,          // the account that pays for opening the delegation. rent will be recovered on undelegation.
      &ctx.accounts.pda,            // the PDA to delegate
      &ctx.accounts.owner_program,  // the owner program of the PDA
      &ctx.accounts.buffer,
      &ctx.accounts.delegation_record,
      &ctx.accounts.delegation_metadata,
      &ctx.accounts.delegation_program,
      &ctx.accounts.system_program,
      pda_seeds,                    // the seeds to make thhe PDA signer
      0,                            // the time limit for the delegation (0 for no limit)
      30_000,                        // the update frequency on the base layer in milliseconds
    )?;

    Ok(())
  }

  // increment the counter and manually commit the account in the Ephemeral Rollup session
  pub fn increment_and_commit(ctx: Context<IncrementAndCommit>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.count += 1;
    commit_accounts(
      &ctx.accounts.payer,
      vec![&ctx.accounts.counter.to_account_info()],
      &ctx.accounts.magic_context,
      &ctx.accounts.magic_program,
    )?;
    Ok(())
  }

  // undelegate the account from the delegation program
  pub fn undelegate(ctx: Context<IncrementAndCommit>) -> Result<()> {
    commit_and_undelegate_accounts(
      &ctx.accounts.payer,
      vec![&ctx.accounts.counter.to_account_info()],
      &ctx.accounts.magic_context,
      &ctx.accounts.magic_program,
    )?;
    Ok(())
  }

  // increment the counter and manually commit the account in the Ephemeral Rollup session
  pub fn increment_and_undelegate(ctx: Context<IncrementAndCommit>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.count += 1;
    counter.exit(&crate::ID)?;
    commit_and_undelegate_accounts(
      &ctx.accounts.payer,
      vec![&ctx.accounts.counter.to_account_info()],
      &ctx.accounts.magic_context,
      &ctx.accounts.magic_program,
    )?;
    Ok(())
  }
}

// everything below this line was not included in the tutorial

#[derive(Accounts)]
pub struct Initialize<'info> {
  #[account(init, payer = user, space = 8 + 8, seeds = [TEST_PDA_SEED], bump)]
  pub counter: Account<'info, Counter>,
  #[account(mut)]
  pub user: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
  #[account(mut, seeds = [TEST_PDA_SEED], bump)]
  pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
pub struct DelegateInput<'info> {
  pub payer: Signer<'info>,
  /// CHECK: the PDA to delegate
  #[account(mut)]
  pub pda: AccountInfo<'info>,
  /// CHECK: the owner program of the PDA
  #[account(address = crate::id())]
  pub owner_program: AccountInfo<'info>,
  /// CHECK: the temporary buffer account used during delegation
  #[account(
    mut, 
    seeds = [ephemeral_rollups_sdk::consts::BUFFER, crate::id().as_ref()],
    bump, 
    seeds::program = delegation_program.key()
  )]
  pub buffer: AccountInfo<'info>,
  /// CHECK: the delegation record account
  #[account(
    mut,
    seeds = [ephemeral_rollups_sdk::consts::DELEGATION_RECORD, pda.key().as_ref()],
    bump,
    seeds::program = delegation_program.key()
  )]
  pub delegation_record: AccountInfo<'info>,
  /// CHECK: the delegation metadata account
  #[account(
    mut,
    seeds = [ephemeral_rollups_sdk::consts::DELEGATION_METADATA, pda.key().as_ref()],
    bump,
    seeds::program = delegation_program.key()
  )]
  pub delegation_metadata: AccountInfo<'info>,
  pub delegation_program: Program<'info, DelegationProgram>,
  pub system_program: Program<'info, System>,
}

#[commit]
#[derive(Accounts)]
pub struct IncrementAndCommit<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,
  #[account(mut, seeds = [TEST_PDA_SEED], bump)]
  pub counter: Account<'info, Counter>,
}

#[account]
pub struct Counter {
  pub count: u64,
}