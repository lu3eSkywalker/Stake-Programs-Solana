use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_lang::system_program;
use solana_program::system_instruction;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

declare_id!("FV7mYYyx5FiDoaer7yzKaEJPfwCdRtj3feDuLwsJXWfk");

#[program]
pub mod stake_with_token_reward {
    const REWARD_RATE_PER_SOL_PER_SECOND: u64 = 1; // 1 point per SOL per second

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
        let lamports = rent.minimum_balance(0);
        let size = 0;
    
        let seeds = &[
            b"pdaVault",
            ctx.accounts.authorityVault.key.as_ref(),
            &[ctx.bumps.pda_vault_account],
        ];
        let signer_seeds = &[&seeds[..]];
    
        let create_ix = system_instruction::create_account(
            ctx.accounts.authorityVault.key,
            ctx.accounts.pda_vault_account.key,
            lamports,
            size,
            &system_program::ID,
        );
    
        invoke_signed(
            &create_ix,
            &[
                ctx.accounts.authorityVault.to_account_info(),
                ctx.accounts.pda_vault_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer_seeds,
        )?;
    
        msg!("✅ PDA Vault System Account created successfully!");
        Ok(())
    }

    pub fn create_token_mint(
        ctx: Context<CreateTokenMint>,
        metadata: TokenMintMetadata,
    ) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.authority]]];

        let token_data = DataV2 {
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        );

        create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;
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

        // Minting Tokens
        let signer_seeds: &[&[&[u8]]] = &[&[b"authority", &[ctx.bumps.authority]]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        mint_to(cpi_ctx, amount)?;

        msg!("Staking Successfull and Minted Tokens Successfully");
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakingError::InvalidAmount);

        let pda_account_time = ctx.accounts.pda_account.last_update_time;

        let current_time = Clock::get()?;

        require!(current_time.unix_timestamp - pda_account_time >= 1 * 86400, StakingError::InvalidUnstake);

        let pda_account = &mut ctx.accounts.pda_account;

        require!(pda_account.staked_amount > amount, StakingError::InvalidBalance);

        let authority_key = ctx.accounts.authority.key();

        // Transfer SOL from PDA back to user
        let seeds = &[
            b"pdaVault",
            authority_key.as_ref(),
            &[ctx.bumps.pda_vault_account],
        ];
        let signer = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pda_vault_account.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer,
        );
        transfer(cpi_context, amount)?;

        // Update Staked Amount
        pda_account.staked_amount -= amount;

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
    /// CHECK
    #[account(mut)]
    pub authorityVault: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pdaVault", authorityVault.key().as_ref()],
        bump
    )]

    /// CHECK
    pub pda_vault_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: TokenMintMetadata)]
pub struct CreateTokenMint<'info> {
    /// CHECK: PDA derived from [b"metadata", metadata_program_id, mint]
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"mint"],
        bump,
        mint::decimals = params.decimals,
        mint::authority = authority.key(),
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA that controls the mint
    #[account(
        seeds = [b"authority"],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
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

    #[account(mut)]
    pub authorityVault: Signer<'info>,

    #[account(
        mut, 
        seeds = [b"pdaVault", authorityVault.key().as_ref()],
        bump,
    )]

    /// CHECK
    pub pda_vault_account: SystemAccount<'info>,

    // For Minting Tokens
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = authority.key(),
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK
    #[account(
    seeds = [b"authority"],
    bump
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = destination_owner,
    )]
    pub destination: Account<'info, TokenAccount>,

    /// CHECK: we use this to validate token owner
    pub destination_owner: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}



#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"client1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key()
    )]

    pub pda_account: Account<'info, StakeAccount>,

    /// CHECK
    pub authority: AccountInfo<'info>,

    #[account(
        mut, 
        seeds = [b"pdaVault", authority.key().as_ref()],
        bump,
    )]

    /// CHECK
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

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TokenMintMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
}

#[error_code]
pub enum StakingError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,

    #[msg("Invalid Balance")]
    InvalidBalance,

    #[msg("Cannot Unstake before 30 days of staking")]
    InvalidUnstake
}