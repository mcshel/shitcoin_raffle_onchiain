use anchor_lang::prelude::*;

use crate::errors::*;

#[account]
pub struct Entrant {

    // Bump
    pub bump: u8,

    // User
    pub user: Pubkey,

    // Raffle
    pub raffle: Pubkey,

    // Number of entry tickets
    pub tickets: u64,

    // Number of winning tickets
    pub rewards: u64,
}


impl Entrant {

    pub fn assert_not_awarded(&self) -> Result<()> {
        require!(
            self.rewards == 0,
            RaffleError::EntrantAlreadyAwarded
        );

        Ok(())
    }

    pub fn get_rewards(&self) -> Result<u64> {
        
        Ok(self.rewards)
    }

    pub fn get_refundable_tickets(&self) -> Result<u64> {
        let refundable_tickets = self.tickets.checked_sub(self.rewards).ok_or(RaffleError::InvalidCalculation)?;

        Ok(refundable_tickets)
    }
}