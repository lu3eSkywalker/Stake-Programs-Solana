use anchor_lang::prelude::*;

declare_id!("FGouhFiL9y9UsaFmYfnvEGjk7VWEWeYKxp9DEx4sPRGL");

#[program]
pub mod basic_staking_program {
    use anchor_lang::system_program::{transfer, Transfer};


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
        let vault_pda_account = &mut ctx.accounts.pda_vault_account;

        vault_pda_account.authority = ctx.accounts.authority.key();
        vault_pda_account.total_staked_amount = 0;
        vault_pda_account.bump = ctx.bumps.pda_vault_account;

        msg!("PDA account created successfully");
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
        let vault_pda_account = &mut ctx.accounts.pda_vault_account;
        
        pda_account.staked_amount += amount;
        vault_pda_account.total_staked_amount += amount;
        
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
    pub authority: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 1 + 32,
        seeds = [b"pdaVault", authority.key().as_ref()],
        bump
    )]

    pub pda_vault_account: Account<'info, VaultAccount>,

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
        bump = pda_vault_account.bump,
        constraint = pda_vault_account.authority == authority.key()
    )]
    pub pda_vault_account: Account<'info, VaultAccount>,

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

#[account]
pub struct VaultAccount {
    pub authority: Pubkey,
    pub total_staked_amount: u64,
    pub bump: u8
}