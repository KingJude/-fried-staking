use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("Fr1edStake11111111111111111111111111111111");

const SECONDS_PER_YEAR: u64 = 31_536_000;
const PROGRAM_SECONDS: i64 = 31_536_000;
const FLEX_APY_BPS: u16 = 3_000;
const SEVEN_DAY_APY_BPS: u16 = 4_000;
const THIRTY_DAY_APY_BPS: u16 = 6_000;
const SEVEN_DAYS: u64 = 604_800;
const THIRTY_DAYS: u64 = 2_592_000;

#[program]
pub mod fried_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, minimum_stake: u64) -> Result<()> {
        require!(minimum_stake > 0, StakingError::InvalidAmount);
        let now = Clock::get()?.unix_timestamp;
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.mint = ctx.accounts.mint.key();
        config.stake_vault = ctx.accounts.stake_vault.key();
        config.reward_vault = ctx.accounts.reward_vault.key();
        config.minimum_stake = minimum_stake;
        config.start_time = now;
        config.end_time = now.checked_add(PROGRAM_SECONDS).ok_or(StakingError::MathOverflow)?;
        config.total_staked = 0;
        config.reward_liability = 0;
        config.paused = false;
        config.bump = ctx.bumps.config;
        config.stake_vault_bump = ctx.bumps.stake_vault;
        config.reward_vault_bump = ctx.bumps.reward_vault;
        Ok(())
    }

    pub fn fund_rewards(ctx: Context<FundRewards>, amount: u64) -> Result<()> {
        require!(amount > 0, StakingError::InvalidAmount);
        token::transfer(ctx.accounts.transfer_ctx(), amount)
    }

    pub fn set_paused(ctx: Context<AdminAction>, paused: bool) -> Result<()> {
        ctx.accounts.config.paused = paused;
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, stake_id: u64, pool: Pool, amount: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let config = &mut ctx.accounts.config;
        require!(!config.paused, StakingError::Paused);
        require!(now >= config.start_time && now < config.end_time, StakingError::ProgramInactive);
        require!(amount >= config.minimum_stake, StakingError::BelowMinimum);

        let (lock_seconds, apy_bps) = pool.terms();
        // Reserve the maximum possible one-year reward so every accepted stake remains solvent.
        let max_reward = calculate_reward(amount, apy_bps, SECONDS_PER_YEAR)?;
        let available = ctx.accounts.reward_vault.amount
            .checked_sub(config.reward_liability)
            .ok_or(StakingError::InsufficientRewards)?;
        require!(available >= max_reward, StakingError::InsufficientRewards);

        token::transfer(ctx.accounts.deposit_ctx(), amount)?;

        let position = &mut ctx.accounts.position;
        position.owner = ctx.accounts.owner.key();
        position.stake_id = stake_id;
        position.pool = pool;
        position.principal = amount;
        position.started_at = now;
        position.unlock_at = now.checked_add(lock_seconds as i64).ok_or(StakingError::MathOverflow)?;
        position.apy_bps = apy_bps;
        position.bump = ctx.bumps.position;

        config.total_staked = config.total_staked.checked_add(amount).ok_or(StakingError::MathOverflow)?;
        config.reward_liability = config.reward_liability.checked_add(max_reward).ok_or(StakingError::MathOverflow)?;
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let position = &ctx.accounts.position;
        require!(now >= position.unlock_at, StakingError::StillLocked);

        let elapsed = now.saturating_sub(position.started_at) as u64;
        let reward_seconds = elapsed.min(SECONDS_PER_YEAR);
        let reward = calculate_reward(position.principal, position.apy_bps, reward_seconds)?;
        require!(ctx.accounts.reward_vault.amount >= reward, StakingError::InsufficientRewards);

        let config_key = ctx.accounts.config.key();
        let signer_seeds: &[&[u8]] = &[b"config", ctx.accounts.config.mint.as_ref(), &[ctx.accounts.config.bump]];
        let signer = &[signer_seeds];

        token::transfer(ctx.accounts.return_principal_ctx().with_signer(signer), position.principal)?;
        if reward > 0 {
            token::transfer(ctx.accounts.pay_reward_ctx().with_signer(signer), reward)?;
        }

        let config = &mut ctx.accounts.config;
        let reserved = calculate_reward(position.principal, position.apy_bps, SECONDS_PER_YEAR)?;
        config.total_staked = config.total_staked.checked_sub(position.principal).ok_or(StakingError::MathOverflow)?;
        config.reward_liability = config.reward_liability.saturating_sub(reserved);
        msg!("Unstaked {} principal and {} rewards", position.principal, reward);
        let _ = config_key;
        Ok(())
    }
}

fn calculate_reward(principal: u64, apy_bps: u16, seconds: u64) -> Result<u64> {
    let numerator = (principal as u128)
        .checked_mul(apy_bps as u128).ok_or(StakingError::MathOverflow)?
        .checked_mul(seconds as u128).ok_or(StakingError::MathOverflow)?;
    let denominator = 10_000u128.checked_mul(SECONDS_PER_YEAR as u128).unwrap();
    u64::try_from(numerator / denominator).map_err(|_| error!(StakingError::MathOverflow))
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq, Eq)]
pub enum Pool {
    Flexible,
    SevenDay,
    ThirtyDay,
}

impl Pool {
    fn terms(&self) -> (u64, u16) {
        match self {
            Self::Flexible => (0, FLEX_APY_BPS),
            Self::SevenDay => (SEVEN_DAYS, SEVEN_DAY_APY_BPS),
            Self::ThirtyDay => (THIRTY_DAYS, THIRTY_DAY_APY_BPS),
        }
    }
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub stake_vault: Pubkey,
    pub reward_vault: Pubkey,
    pub minimum_stake: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub total_staked: u64,
    pub reward_liability: u64,
    pub paused: bool,
    pub bump: u8,
    pub stake_vault_bump: u8,
    pub reward_vault_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Position {
    pub owner: Pubkey,
    pub stake_id: u64,
    pub pool: Pool,
    pub principal: u64,
    pub started_at: i64,
    pub unlock_at: i64,
    pub apy_bps: u16,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"config", mint.key().as_ref()],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        token::mint = mint,
        token::authority = config,
        seeds = [b"stake-vault", config.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = admin,
        token::mint = mint,
        token::authority = config,
        seeds = [b"reward-vault", config.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct FundRewards<'info> {
    #[account(mut, has_one = admin, has_one = mint, has_one = reward_vault)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut, constraint = admin_token.mint == mint.key(), constraint = admin_token.owner == admin.key())]
    pub admin_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> FundRewards<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.admin_token.to_account_info(),
            to: self.reward_vault.to_account_info(),
            authority: self.admin.to_account_info(),
        })
    }
}

#[derive(Accounts)]
pub struct AdminAction<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(stake_id: u64)]
pub struct Stake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = mint, has_one = stake_vault, has_one = reward_vault)]
    pub config: Account<'info, Config>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = owner,
        space = 8 + Position::INIT_SPACE,
        seeds = [b"position", owner.key().as_ref(), &stake_id.to_le_bytes()],
        bump
    )]
    pub position: Account<'info, Position>,
    #[account(mut, constraint = owner_token.mint == mint.key(), constraint = owner_token.owner == owner.key())]
    pub owner_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    fn deposit_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.owner_token.to_account_info(),
            to: self.stake_vault.to_account_info(),
            authority: self.owner.to_account_info(),
        })
    }
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = mint, has_one = stake_vault, has_one = reward_vault)]
    pub config: Account<'info, Config>,
    pub mint: Account<'info, Mint>,
    #[account(mut, close = owner, has_one = owner)]
    pub position: Account<'info, Position>,
    #[account(mut, constraint = owner_token.mint == mint.key(), constraint = owner_token.owner == owner.key())]
    pub owner_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Unstake<'info> {
    fn return_principal_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.stake_vault.to_account_info(),
            to: self.owner_token.to_account_info(),
            authority: self.config.to_account_info(),
        })
    }

    fn pay_reward_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(self.token_program.to_account_info(), Transfer {
            from: self.reward_vault.to_account_info(),
            to: self.owner_token.to_account_info(),
            authority: self.config.to_account_info(),
        })
    }
}

#[error_code]
pub enum StakingError {
    #[msg("Amount must be greater than zero.")]
    InvalidAmount,
    #[msg("Stake amount is below the configured minimum.")]
    BelowMinimum,
    #[msg("The staking program is paused.")]
    Paused,
    #[msg("The staking program is not active.")]
    ProgramInactive,
    #[msg("This position is still locked.")]
    StillLocked,
    #[msg("The reward vault does not have enough unreserved tokens.")]
    InsufficientRewards,
    #[msg("Arithmetic overflow.")]
    MathOverflow,
}
