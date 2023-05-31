use anchor_lang::prelude::*;
// use anchor_spl::{
//     token::{self, Mint, Token, TokenAccount},
// };
use anchor_spl::{
    token::{Mint, TokenAccount},
    token_2022::{self, Token2022},
};

use crate::errors::*;
use crate::states::*;



/*
 * Initialize a new raffle
 */

#[derive(Accounts)]
#[instruction(seed: Pubkey)]
pub struct InitRaffle<'info> {

    // AdminSettings account
    #[account(seeds = [b"admin".as_ref()], bump)]
    pub admin_settings: Account<'info, AdminSettings>,
    
    // Raffle account
    #[account(
        init, 
        seeds = [b"raffle".as_ref(), seed.as_ref()], 
        bump, 
        payer = authority, 
        space = 8 + std::mem::size_of::<Raffle>(),
    )]
    pub raffle: Account<'info, Raffle>,    

    // Proceeds token account
    #[account(
        init,
        seeds = [b"proceeds".as_ref(), raffle.key().as_ref()],
        bump,
        payer = authority,
        token::mint = currency,
        token::authority = raffle,
        token::token_program = token_program,
    )]
    pub proceeds: Account<'info, TokenAccount>,

    // Proceeds mint
    #[account(
        mint::token_program = token_program,
    )]
    pub currency: Account<'info, Mint>,

    // Reward mint
    #[account(
        mint::token_program = token_program,
        constraint = reward.mint_authority == anchor_lang::solana_program::program_option::COption::Some(raffle.key())
    )]
    pub reward: Account<'info, Mint>,
    
    // Admin account
    #[account(mut, constraint = admin_settings.admin == authority.key())]
    pub authority: Signer<'info>,

    // Token program
    pub token_program: Program<'info, Token2022>,
    
    // System program
    pub system_program: Program<'info, System>,
}

pub fn initialize(
    ctx: Context<InitRaffle>,
    seed: Pubkey,
    price: u64,
    rewards_num: u64,
    rewards_amount: u64,
    start_timestamp: i64,
    end_timestamp: i64,
    fee: Option<u64>,
    tickets: Option<u64>,
    limit: Option<u64>,
) -> Result<()> {
        
    let clock = Clock::get()?;
    
    require!(
        price > fee.unwrap_or(0),
        RaffleError::FeeGreaterThanPrice
    );

    require!(
        rewards_num < tickets.unwrap_or(u64::MAX),
        RaffleError::RewardsNumGreaterThanTickets
    );

    require!(
        limit.unwrap_or(1) > 0,
        RaffleError::LimitLessThanOne
    );

    require!(
        start_timestamp < end_timestamp,
        RaffleError::StartAfterEndTimestamp
    );
    
    require!(
        clock.unix_timestamp < end_timestamp,
        RaffleError::EndTimestampAlreadyPassed
    );
    
    let raffle = &mut ctx.accounts.raffle;
    raffle.bump = ctx.bumps["raffle"];
    raffle.seed = seed;
    raffle.price = price;
    raffle.fee = fee.unwrap_or(0);
    raffle.currency = ctx.accounts.currency.key();
    raffle.rewards_num = rewards_num;
    raffle.rewards_amount = rewards_amount;
    raffle.reward = ctx.accounts.reward.key();
    raffle.start_timestamp = start_timestamp;
    raffle.end_timestamp = end_timestamp;
    raffle.tickets = tickets;
    raffle.limit = limit;
    raffle.tickets_sold = 0;
    raffle.rewards_awarded = 0;
    raffle.rewards_claimed = 0;
    raffle.admin_claimed = false;
    
    Ok(())
}


/*
 * Buy tickets
 */

#[derive(Accounts)]
pub struct BuyTickets<'info> {

    // Raffle account
    #[account(mut)]
    pub raffle: Account<'info, Raffle>,

    // Entrant account
    #[account(
        mut,
        seeds = [b"entrant".as_ref(), raffle.key().as_ref(), user.key().as_ref()], 
        bump,
    )]
    pub entrant: Account<'info, Entrant>,

    // Proceeds token account
    #[account(
        mut,
        seeds = [b"proceeds".as_ref(), raffle.key().as_ref()],
        bump,
        token::mint = currency,
        token::authority = raffle,
    )]
    pub proceeds: Account<'info, TokenAccount>,

    // User's proceeds token account
    #[account(
        mut,
        associated_token::mint = currency,
        associated_token::authority = user,
    )]
    pub user_proceeds: Account<'info, TokenAccount>,

    // Proceeds mint
    pub currency: Account<'info, Mint>,

    // User account
    pub user: Signer<'info>,

    // Token program
    pub token_program: Program<'info, Token2022>,
}


pub fn buy_tickets(
    ctx: Context<BuyTickets>,
    amount: u64
) -> Result<()> {
    
    let raffle = &mut ctx.accounts.raffle;
    let entrant = &mut ctx.accounts.entrant;
    let currency = &ctx.accounts.currency;
    
    raffle.assert_active()?;
    
    let total_price = raffle.price.checked_mul(amount).ok_or(RaffleError::InvalidCalculation)?;
    let total_tickets = raffle.tickets_sold.checked_add(amount).ok_or(RaffleError::InvalidCalculation)?;
    let entrant_tickets = entrant.tickets.checked_add(amount).ok_or(RaffleError::InvalidCalculation)?;

    require!(
        total_tickets <= raffle.tickets.unwrap_or(u64::MAX),
        RaffleError::RaffleTicketsUnavailable
    );

    require!(
        entrant_tickets <= raffle.limit.unwrap_or(u64::MAX),
        RaffleError::EntrantTicketLimitReached
    );
    
//     let cpi_accounts = token::Transfer {
//         from: ctx.accounts.user_proceeds.to_account_info(),
//         to: ctx.accounts.proceeds.to_account_info(),
//         authority: ctx.accounts.user.to_account_info(),
//     };

    let cpi_accounts = token_2022::TransferChecked {
        from: ctx.accounts.user_proceeds.to_account_info(),
        mint: ctx.accounts.currency.to_account_info(),
        to: ctx.accounts.proceeds.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token_2022::transfer_checked(cpi_ctx, total_price, currency.decimals)?;
    
    raffle.tickets_sold = total_tickets;
    entrant.tickets = entrant_tickets;
    
    Ok(())
}


/*
 * Set rewards
 */

#[derive(Accounts)]
#[instruction(_user: Pubkey)]
pub struct SetRewards<'info> {

    // AdminSettings account
    #[account(
        seeds = [b"admin".as_ref()], 
        bump
    )]
    pub admin_settings: Account<'info, AdminSettings>,

    // Raffle account
    #[account(mut)]
    pub raffle: Account<'info, Raffle>,

    // Entrant account
    #[account(
        mut,
        seeds = [b"entrant".as_ref(), raffle.key().as_ref(), _user.key().as_ref()], 
        bump,
    )]
    pub entrant: Account<'info, Entrant>,

    // Admin account
    #[account(constraint = admin_settings.admin == authority.key())]
    pub authority: Signer<'info>,
}

pub fn set_rewards(
    ctx: Context<SetRewards>,
    _user: Pubkey,
    amount: u64,
) -> Result<()> {
    
    let raffle = &mut ctx.accounts.raffle;
    let entrant = &mut ctx.accounts.entrant;
    
    raffle.assert_ended()?;
    entrant.assert_not_awarded()?;

    let rewards_awarded = raffle.rewards_awarded.checked_add(amount).ok_or(RaffleError::InvalidCalculation)?;
    
    require!(
        amount <= entrant.tickets,
        RaffleError::RewardsNumGreaterThanTicketsBought
    );
    
    require!(
        rewards_awarded <= raffle.rewards_amount,
        RaffleError::RewardsAmountGreaterThanTotal
    );
    
    raffle.rewards_awarded = rewards_awarded;
    entrant.rewards = amount;
    
    Ok(())
}


/*
 * Claim proceeds
 */

#[derive(Accounts)]
pub struct ClaimProceeds<'info> {

    // AdminSettings account
    #[account(
        seeds = [b"admin".as_ref()], 
        bump
    )]
    pub admin_settings: Account<'info, AdminSettings>,

    // Raffle account
    #[account(mut)]
    pub raffle: Account<'info, Raffle>,

    // Proceeds token account
    #[account(
        mut,
        seeds = [b"proceeds".as_ref(), raffle.key().as_ref()],
        bump,
        token::mint = currency,
        token::authority = raffle,
    )]
    pub proceeds: Account<'info, TokenAccount>,

    // Admin's proceeds token account
    #[account(
        mut,
        associated_token::mint = currency,
        associated_token::authority = authority,
    )]
    pub admin_proceeds: Account<'info, TokenAccount>,

    // Proceeds mint
    pub currency: Account<'info, Mint>,

    // Admin account
    #[account(
        mut,
        constraint = admin_settings.admin == authority.key()
    )]
    pub authority: Signer<'info>,

    // Token program
    pub token_program: Program<'info, Token2022>,
}

pub fn claim_proceeds(ctx: Context<ClaimProceeds>,) -> Result<()> {
    
    let raffle = &mut ctx.accounts.raffle;
    let currency = &ctx.accounts.currency;
    
    raffle.assert_claimable()?;
    
    let authority_proceeds = raffle.get_authority_proceeds()?;
    if authority_proceeds > 0 {
        let cpi_accounts = token_2022::TransferChecked {
            from: ctx.accounts.proceeds.to_account_info(),
            mint: ctx.accounts.currency.to_account_info(),
            to: ctx.accounts.admin_proceeds.to_account_info(),
            authority: raffle.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        token_2022::transfer_checked(
            CpiContext::new_with_signer(
                cpi_program,
                cpi_accounts,
                &[&[b"raffle".as_ref(), raffle.seed.as_ref(), &[raffle.bump]]]
            ),
            authority_proceeds,
            currency.decimals
        )?;
    }

    raffle.admin_claimed = true;
    
    Ok(())
}


/*
 * Close raffle
 */

#[derive(Accounts)]
pub struct CloseRaffle<'info> {
    
    // AdminSettings account
    #[account(
        seeds = [b"admin".as_ref()], 
        bump
    )]
    pub admin_settings: Account<'info, AdminSettings>,
    
    // Raffle account
    #[account(
        mut,
        close = authority
    )]
    pub raffle: Account<'info, Raffle>,

    // Proceeds token account
    #[account(
        mut,
        seeds = [b"proceeds".as_ref(), raffle.key().as_ref()],
        bump,
        token::mint = currency,
        token::authority = raffle,
    )]
    pub proceeds: Account<'info, TokenAccount>,
    
    // Proceeds mint
    pub currency: Account<'info, Mint>,

    // Admin account
    #[account(
        mut,
        constraint = admin_settings.admin == authority.key()
    )]
    pub authority: Signer<'info>,

    // Token program
    pub token_program: Program<'info, Token2022>,
}


pub fn close(ctx: Context<CloseRaffle>) -> Result<()> {
    let raffle = &ctx.accounts.raffle;

    raffle.assert_closeable()?;
    
    let cpi_accounts = token_2022::CloseAccount {
        account: ctx.accounts.proceeds.to_account_info(),
        destination: ctx.accounts.authority.to_account_info(),
        authority: raffle.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    token_2022::close_account(
        CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            &[&[b"raffle".as_ref(), raffle.seed.as_ref(), &[raffle.bump]]]
        )
    )?;

    Ok(())
}
    
    
    
