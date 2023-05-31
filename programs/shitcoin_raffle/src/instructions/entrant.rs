use anchor_lang::prelude::*;
// use anchor_spl::{
//     token::{self, Mint, Token, TokenAccount, Transfer, MintTo},
//     associated_token::AssociatedToken
// };
use anchor_spl::{
    token::{Mint, TokenAccount},
    token_2022::{self, Token2022, TransferChecked, MintTo},
    associated_token::AssociatedToken
};

use crate::states::*;


/*
 * Initialize a new entrant account
 */

#[derive(Accounts)]
pub struct InitEntrant<'info> {
 
    // Raffle account
    pub raffle: Account<'info, Raffle>,    
 
    // Entrant account
    #[account(
       init,
       seeds = [b"entrant".as_ref(), raffle.key().as_ref(), user.key().as_ref()],
       bump,
       payer = user,
       space = 8 + std::mem::size_of::<Entrant>(),
    )]
    pub entrant: Account<'info, Entrant>,

    // User
    #[account(mut)]
    pub user: Signer<'info>,

    // System program
    pub system_program: Program<'info, System>,
}
 
pub fn initialize(ctx: Context<InitEntrant>) -> Result<()> {
     
    let raffle = &ctx.accounts.raffle;
    let entrant = &mut ctx.accounts.entrant;

    raffle.assert_active()?;

    entrant.bump = ctx.bumps["entrant"];
    entrant.user = ctx.accounts.user.key();
    entrant.raffle = ctx.accounts.raffle.key();
    entrant.tickets = 0;
    entrant.rewards = 0;
     
    Ok(())
}


/*
 * Claim rewards / proceeds and close entrant
 */

#[derive(Accounts)]
pub struct CloseEntrant<'info> {

    // Raffle account
    #[account(mut)]
    pub raffle: Box<Account<'info, Raffle>>,
 
    // Entrant account
    #[account(
        mut,
        seeds = [b"entrant".as_ref(), raffle.key().as_ref(), user.key().as_ref()],
        bump,
        close = user,
    )]
    pub entrant: Box<Account<'info, Entrant>>,

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

    // User's reward token account
    #[account(
        mut,
        associated_token::mint = reward,
        associated_token::authority = user,
    )]
    pub user_reward: Account<'info, TokenAccount>,

    // Rewawrd mint
    #[account(
        mut,
        constraint = reward.mint_authority == anchor_lang::solana_program::program_option::COption::Some(raffle.key()),
    )]
    pub reward: Account<'info, Mint>,

    // User
    #[account(mut)]
    pub user: Signer<'info>,

    // Token program
    pub token_program: Program<'info, Token2022>,

    // Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn close(ctx: Context<CloseEntrant>) -> Result<()> {

    let raffle = &mut ctx.accounts.raffle;
    let entrant = &ctx.accounts.entrant;
    let currency = &ctx.accounts.currency;

    raffle.assert_awarded()?;

    let refundable_tickets = entrant.get_refundable_tickets()?;
    let refundable_amount = raffle.get_refunable_proceeds(refundable_tickets)?;
    if refundable_amount > 0 {
        let cpi_transfer_accounts = TransferChecked {
            from: ctx.accounts.proceeds.to_account_info(),
            mint: ctx.accounts.currency.to_account_info(),
            to: ctx.accounts.user_proceeds.to_account_info(),
            authority: raffle.to_account_info(),
        };
    
        let cpi_transfer_program = ctx.accounts.token_program.to_account_info();
        token_2022::transfer_checked(
            CpiContext::new_with_signer(
                cpi_transfer_program,
                cpi_transfer_accounts,
                &[&[b"raffle".as_ref(), raffle.seed.as_ref(), &[raffle.bump]]]
            ), 
            refundable_amount,
            currency.decimals
        )?;
    }

    let reward_amount = raffle.get_reward_amount(entrant.rewards)?;
    if reward_amount > 0 {
        raffle.claim_rewards(entrant.rewards)?;
    
        let cpi_mint_accounts = MintTo {
            mint: ctx.accounts.reward.to_account_info(),
            to: ctx.accounts.user_reward.to_account_info(),
            authority: raffle.to_account_info(),
        };

        let cpi_mint_program = ctx.accounts.token_program.to_account_info();
        token_2022::mint_to(
            CpiContext::new_with_signer(
                cpi_mint_program,
                cpi_mint_accounts,
                &[&[b"raffle".as_ref(), raffle.seed.as_ref(), &[raffle.bump]]]
            ),
            reward_amount
        )?;
    }

    Ok(())
}


