pub mod errors;
pub mod states;
pub mod instructions;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("DTXiWKJEs8DKd1K1Ex4TpyMNSyUdtxmoe7JjXM2gBzf5");

#[program]
pub mod shitcoin_raffle {
    use super::*;

    // ----- Raffle program config functions -----

    pub fn init_admin(ctx: Context<InitAdmin>, admin: Pubkey) -> Result<()> {
        instructions::config::init_admin(ctx, admin)?;
        Ok(())
    }

    pub fn set_admin(ctx: Context<SetAdmin>, admin: Pubkey) -> Result<()> {
        instructions::config::set_admin(ctx, admin)?;
        Ok(())
    }


    // ----- Entrant functions -----

    pub fn init_entrant(ctx: Context<InitEntrant>) -> Result<()> {
        instructions::entrant::initialize(ctx)?;
        Ok(())
    }

    pub fn close_entrant(ctx: Context<CloseEntrant>) -> Result<()> {
        instructions::entrant::close(ctx)?;
        Ok(())
    }


    // ----- Raffle functions -----

    pub fn init_raffle(
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
        instructions::raffle::initialize(
            ctx,
            seed,
            price,
            rewards_num,
            rewards_amount,
            start_timestamp,
            end_timestamp,
            fee,
            tickets,
            limit,
        )?;
        Ok(())
    }

    pub fn buy_tickets(ctx: Context<BuyTickets>, amount: u64) -> Result<()> {
        instructions::raffle::buy_tickets(ctx, amount)?;
        Ok(())
    }

    pub fn set_reward(ctx: Context<SetRewards>, user: Pubkey, amount: u64) -> Result<()> {
        instructions::raffle::set_rewards(ctx, user, amount)?;
        Ok(())
    }

    pub fn claim_proceeds(ctx: Context<ClaimProceeds>) -> Result<()> {
        instructions::raffle::claim_proceeds(ctx)?;
        Ok(())
    }

    pub fn close_raffle(ctx: Context<CloseRaffle>) -> Result<()> {
        instructions::raffle::close(ctx)?;
        Ok(())
    }
}