use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, transfer, Transfer};
use solana_program::program::invoke_signed;
use solana_program::system_instruction;

declare_id!("8y8S3RW35AuEj2N9B5RVB8jimLEmjbRDHk91Wzjant2h");

#[program]
pub mod basic_staking_program {
    use super::*;

    pub fn create_pda_account(ctx: Context<CreatePdaAccount>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        pda_account.owner = ctx.accounts.payer.key();
        pda_account.staked_amount = 0;
        pda_account.total_points = 0;
        pda_account.last_update_time = clock.unix_timestamp;
        pda_account.bump = ctx.bumps.pda_account;

        msg!("PDA account created successfully");
        Ok(())
    }

    pub fn create_vault_pda_account(ctx: Context<VaultPdaAccount>) -> Result<()> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0); // 0 because SystemAccount has no data
        let size = 0;

        let seeds = &[
            b"pdaVault",
            ctx.accounts.authority.key.as_ref(),
            &[ctx.bumps.pda_vault_account],
        ];
        let signer_seeds = &[&seeds[..]];

        let create_ix = system_instruction::create_account(
            ctx.accounts.authority.key,
            ctx.accounts.pda_vault_account.key,
            lamports,
            size,
            &system_program::ID,
        );

        invoke_signed(
            &create_ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.pda_vault_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        msg!("✅ PDA Vault System Account created successfully!");
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let from_pubkey = ctx.accounts.user.to_account_info();
        let to_pubkey = ctx.accounts.pda_vault_account.to_account_info();
        let program_id = ctx.accounts.system_program.to_account_info();

        let cpi_context = CpiContext::new(
            program_id,
            Transfer {
                from: from_pubkey,
                to: to_pubkey,
            },
        );

        transfer(cpi_context, amount)?;

        let pda_account = &mut ctx.accounts.pda_account;

        pda_account.staked_amount += amount;

        msg!("Staking Successfull");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePdaAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8 + 8 + 8 + 1, // discriminator + owner + staked_amount + total_points + last_update_time + bump
        seeds = [b"client1", payer.key().as_ref()],
        bump
    )]
    pub pda_account: Account<'info, StakeAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VaultPdaAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pdaVault", authority.key().as_ref()],
        bump
    )]
    pub pda_vault_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"client1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key()
    )]
    pub pda_account: Account<'info, StakeAccount>,

    pub authority: AccountInfo<'info>,

    #[account(
        mut, 
        seeds = [b"pdaVault", authority.key().as_ref()],
        bump,
    )]
    pub pda_vault_account: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub total_points: u64,
    pub last_update_time: i64,
    pub bump: u8,
}

#[error_code]
pub enum StakingError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,

    #[msg("Invalid Balance")]
    InvalidBalance,
}
