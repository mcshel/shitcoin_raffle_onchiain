use std::cmp;
use anchor_lang::prelude::*;

use crate::errors::*;


#[account]
pub struct Raffle {
    // Bump used in generating the Raffle account
    pub bump: u8,

    // Seed used in deriving the raffle account
    pub seed: Pubkey,

    // Entry price of the raffle
    pub price: u64,

    // Non-refundable fee
    pub fee: u64,

    // Currency mint
    pub currency: Pubkey,

    // Number of rewards
    pub rewards_num: u64,

    // Reward amount
    pub rewards_amount: u64,

    // Reward mint
    pub reward: Pubkey,

    // Raffle start timestamp
    pub start_timestamp: i64,

    // Raffle end timestamp
    pub end_timestamp: i64,

    // Number of tickets
    pub tickets: Option<u64>,

    // Maximum number of tickets that can be purchased by single user
    pub limit: Option<u64>,

    // Number of tickets sold
    pub tickets_sold: u64,

    // Number of awarded rewards
    pub rewards_awarded: u64,

    // Number of claimed rewards
    pub rewards_claimed: u64,

    // Admin has claimed the proceeds
    pub admin_claimed: bool,
}

impl Raffle {

    pub fn assert_active(&self) -> Result<()> {
        let clock = Clock::get()?;

        require!(
            clock.unix_timestamp >= self.start_timestamp,
            RaffleError::RaffleNotStarted
        );

        require!(
            clock.unix_timestamp < self.end_timestamp,
            RaffleError::RaffleEnded
        );

        require!(
            self.tickets_sold < self.tickets.unwrap_or(u64::MAX),
            RaffleError::RaffleSoldOut
        );

        Ok(())
    }

    pub fn assert_ended(&self) -> Result<()> {
        let clock = Clock::get()?;

        require!(
            clock.unix_timestamp >= self.end_timestamp || self.tickets_sold == self.tickets.unwrap_or(u64::MAX),
            RaffleError::RaffleStillActive
        );

        Ok(())
    }

    pub fn assert_awarded(&self) -> Result<()> {
        self.assert_ended()?;

        require!(
            self.rewards_awarded == cmp::min(self.rewards_num, self.tickets_sold),
            RaffleError::RaffleRewardsNotSet
        );

        Ok(())
    }

    pub fn assert_claimable(&self) -> Result<()> {
        self.assert_awarded()?;

        require!(
            self.admin_claimed == false,
            RaffleError::RaffleAdminAlreadyClaimed
        );

        Ok(())
    }

    pub fn assert_closeable(&self) -> Result<()> {
        self.assert_awarded()?;

        require!(
            self.admin_claimed == true,
            RaffleError::RaffleAdminNotClaimed
        );

        require!(
            self.rewards_claimed == self.rewards_num,
            RaffleError::RaffleRewardsNotClaimed
        );

        Ok(())
    }
    
    pub fn claim_rewards(&mut self, tickets: u64) -> Result<()> {
        self.rewards_claimed = self.rewards_claimed.checked_add(tickets).ok_or(RaffleError::InvalidCalculation)?;
        
        Ok(())
    }

    pub fn get_reward_amount(&self, tickets: u64) -> Result<u64> {
        let reward_amount = self.rewards_amount.checked_mul(tickets).ok_or(RaffleError::InvalidCalculation)?;

        Ok(reward_amount)
    }

    pub fn get_refunable_proceeds(&self, tickets: u64) -> Result<u64> {
        let refundable_proceeds = self.price.checked_mul(tickets).ok_or(RaffleError::InvalidCalculation)?;
        let fee_proceeds = self.fee.checked_mul(tickets).ok_or(RaffleError::InvalidCalculation)?;

        Ok(refundable_proceeds - fee_proceeds)
    }

    pub fn get_authority_proceeds(&self) -> Result<u64> {
        let non_refundable_proceeds = self.price.checked_mul(self.rewards_awarded).ok_or(RaffleError::InvalidCalculation)?;
        let refundable_tickets = self.tickets_sold.checked_sub(self.rewards_awarded).ok_or(RaffleError::InvalidCalculation)?;
        let fee_proceeds = self.fee.checked_mul(refundable_tickets).ok_or(RaffleError::InvalidCalculation)?;
        let total_proceeds = non_refundable_proceeds.checked_add(fee_proceeds).ok_or(RaffleError::InvalidCalculation)?;

        Ok(total_proceeds)
    }
}
