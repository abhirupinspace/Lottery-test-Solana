use anchor_lang::prelude::*;

declare_id!("GEra2comDtoEtgoSUqdiezrDCnnHSSEZ4uSLTWxh86Rn");

#[program]
pub mod lottery_contract {
    use super::*;

    // Initialize the lottery
    pub fn initialize(
        ctx: Context<Initialize>,
        prize_pool_bump: u8,
        lottery_state_bump: u8,
    ) -> Result<()> {
        let lottery_state = &mut ctx.accounts.lottery_state;
        lottery_state.admin = ctx.accounts.admin.key();
        lottery_state.prize_pool = ctx.accounts.prize_pool.key();
        lottery_state.bump = lottery_state_bump;
        lottery_state.prize_pool_bump = prize_pool_bump;
        lottery_state.current_round = 1;
        
        // Initialize winning numbers (we'll use 1-50 range for example)
        lottery_state.winning_numbers = generate_winning_numbers();
        lottery_state.prizes = [
            1_000_000_000, // 1 SOL
            500_000_000,   // 0.5 SOL
            300_000_000,   // 0.3 SOL
            200_000_000,   // 0.2 SOL
            100_000_000,   // 0.1 SOL
            50_000_000,    // 0.05 SOL
        ];
        
        Ok(())
    }

    // Admin deposits SOL to prize pool
    pub fn deposit_prize_pool(ctx: Context<DepositPrizePool>, amount: u64) -> Result<()> {
        let from = &ctx.accounts.admin;
        let to = &ctx.accounts.prize_pool;

        // Transfer SOL from admin to prize pool
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &from.key(),
            &to.key(),
            amount
        );
        
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                from.to_account_info(),
                to.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    // User plays the lottery
    pub fn play(ctx: Context<Play>) -> Result<()> {
        let cost = 2_000_000_000; // 2 SOL for testing (should be $2 worth of SOL in production)
        
        // Transfer payment
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.player.key(),
            &ctx.accounts.admin.key(),
            cost / 10, // 10% to admin
        );
        
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Transfer to prize pool
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.player.key(),
            &ctx.accounts.prize_pool.key(),
            cost * 9 / 10, // 90% to prize pool
        );
        
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.prize_pool.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Generate player numbers and check for wins
        let player_numbers = generate_player_numbers();
        let lottery_state = &ctx.accounts.lottery_state;
        
        for (i, &player_num) in player_numbers.iter().enumerate() {
            if player_num == lottery_state.winning_numbers[i] {
                // Player won! Transfer prize
                let prize_amount = lottery_state.prizes[i];
                
                let ix = anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.prize_pool.key(),
                    &ctx.accounts.player.key(),
                    prize_amount,
                );
                
                anchor_lang::solana_program::program::invoke_signed(
                    &ix,
                    &[
                        ctx.accounts.prize_pool.to_account_info(),
                        ctx.accounts.player.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                    &[&[
                        b"prize_pool".as_ref(),
                        &[lottery_state.prize_pool_bump],
                    ]],
                )?;
                
                break; // Exit after first win
            }
        }

        Ok(())
    }

    // Free play for referrals (to be implemented)
    pub fn free_play(ctx: Context<FreePlay>, referral_count: u8) -> Result<()> {
        require!(referral_count >= 3, ErrorCode::InsufficientReferrals);
        
        // Generate player numbers and check for wins
        let player_numbers = generate_player_numbers();
        let lottery_state = &ctx.accounts.lottery_state;
        
        for (i, &player_num) in player_numbers.iter().enumerate() {
            if player_num == lottery_state.winning_numbers[i] {
                // Player won! Transfer prize
                let prize_amount = lottery_state.prizes[i];
                
                let ix = anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.prize_pool.key(),
                    &ctx.accounts.player.key(),
                    prize_amount,
                );
                
                anchor_lang::solana_program::program::invoke_signed(
                    &ix,
                    &[
                        ctx.accounts.prize_pool.to_account_info(),
                        ctx.accounts.player.to_account_info(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                    &[&[
                        b"prize_pool".as_ref(),
                        &[lottery_state.prize_pool_bump],
                    ]],
                )?;
                
                break;
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        init,
        payer = admin,
        space = 8 + LotteryState::SPACE,
        seeds = [b"lottery_state"],
        bump
    )]
    pub lottery_state: Account<'info, LotteryState>,
    
    #[account(
        seeds = [b"prize_pool"],
        bump
    )]
    /// CHECK: This is safe because it's just a SOL holding account
    pub prize_pool: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositPrizePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"prize_pool"],
        bump = lottery_state.prize_pool_bump
    )]
    /// CHECK: This is safe because it's just a SOL holding account
    pub prize_pool: AccountInfo<'info>,
    
    #[account(
        seeds = [b"lottery_state"],
        bump = lottery_state.bump
    )]
    pub lottery_state: Account<'info, LotteryState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(mut)]
    /// CHECK: This is safe as we just transfer SOL to it
    pub admin: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [b"prize_pool"],
        bump = lottery_state.prize_pool_bump
    )]
    /// CHECK: This is safe because it's just a SOL holding account
    pub prize_pool: AccountInfo<'info>,
    
    #[account(
        seeds = [b"lottery_state"],
        bump = lottery_state.bump
    )]
    pub lottery_state: Account<'info, LotteryState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FreePlay<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"prize_pool"],
        bump = lottery_state.prize_pool_bump
    )]
    /// CHECK: This is safe because it's just a SOL holding account
    pub prize_pool: AccountInfo<'info>,
    
    #[account(
        seeds = [b"lottery_state"],
        bump = lottery_state.bump
    )]
    pub lottery_state: Account<'info, LotteryState>,
    
    pub system_program: Program<'info, System>,
}

#[account]
pub struct LotteryState {
    pub admin: Pubkey,
    pub prize_pool: Pubkey,
    pub bump: u8,
    pub prize_pool_bump: u8,
    pub current_round: u64,
    pub winning_numbers: [u8; 6],
    pub prizes: [u64; 6],
}

impl LotteryState {
    pub const SPACE: usize = 32 + // admin pubkey
        32 + // prize_pool pubkey
        1 + // bump
        1 + // prize_pool_bump
        8 + // current_round
        6 + // winning_numbers array
        48; // prizes array (6 * 8)
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient referrals")]
    InsufficientReferrals,
}

// Helper functions for random number generation
// Note: In production, you should use a more secure random number generation method
fn generate_winning_numbers() -> [u8; 6] {
    let mut numbers = [0u8; 6];
    for i in 0..6 {
        // Generate numbers with decreasing ranges
        numbers[i] = ((Clock::get().unwrap().unix_timestamp as u64 % (50 - i as u64)) + 1) as u8;
    }
    numbers
}

fn generate_player_numbers() -> [u8; 6] {
    let mut numbers = [0u8; 6];
    for i in 0..6 {
        // Generate numbers with decreasing ranges
        numbers[i] = ((Clock::get().unwrap().unix_timestamp as u64 % (50 - i as u64)) + 1) as u8;
    }
    numbers
}