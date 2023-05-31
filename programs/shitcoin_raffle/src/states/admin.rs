use anchor_lang::prelude::*;

#[account]
pub struct AdminSettings {
    
    // Bump used in generating the AdminSettings account
    pub bump: u8,
    
    // Address of the admin's account
    pub admin: Pubkey,
} 
