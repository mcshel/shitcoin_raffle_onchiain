use anchor_lang::prelude::*;

use crate::program::ShitcoinRaffle;
use crate::states::*;


/*
 * Initialize program's AddminSettings account and set a program admin
 */


#[derive(Accounts)]
pub struct InitAdmin<'info> {

    // AdminSettings account
    #[account(
        init,
        seeds = [b"admin".as_ref()], 
        bump, 
        payer = authority,
        space = 8 + std::mem::size_of::<AdminSettings>(),
    )]
    pub admin_settings: Account<'info, AdminSettings>,
    
    // ShitcoinRaffle program
    #[account(
        constraint = program.programdata_address()? == Some(program_data.key())
    )]
    pub program: Program<'info, ShitcoinRaffle>,
    
    // ShitcoinRaffle program data
    #[account(
        constraint = program_data.upgrade_authority_address == Some(authority.key())
    )]
    pub program_data: Account<'info, ProgramData>,
    
    // Authority for creating the AdminSettings account -> upgrade authority of the ShitcoinRaffle program
    #[account(mut)]
    pub authority: Signer<'info>,
    
    // System program
    pub system_program: Program<'info, System>,
}


pub fn init_admin(ctx: Context<InitAdmin>, admin: Pubkey) -> Result<()> {

    let admin_settings = &mut ctx.accounts.admin_settings;
    admin_settings.bump = *ctx.bumps.get("admin_settings").unwrap();
    admin_settings.admin = admin;
    
    Ok(())
}



/*
 *  Set admin by updating the AdminSettings account
 */


#[derive(Accounts)]
pub struct SetAdmin<'info> {

    // AdminSettings account
    #[account(
        mut,
        seeds = [b"admin".as_ref()],
        bump,
    )]
    pub admin_settings: Account<'info, AdminSettings>,
    
    // ShitcoinRaffle program
    #[account(
        constraint = program.programdata_address()? == Some(program_data.key())
    )]
    pub program: Program<'info, ShitcoinRaffle>,
    
    // ShitcoinRaffle program data
    #[account(
        constraint = program_data.upgrade_authority_address == Some(authority.key())
    )]
    pub program_data: Account<'info, ProgramData>,
    
    // Authority for updating the AdminSettings account -> upgrade authority of the ShitcoinRaffle program
    #[account(mut)]
    pub authority: Signer<'info>,
} 


pub fn set_admin(ctx: Context<SetAdmin>, admin: Pubkey) -> Result<()> {

    let admin_settings = &mut ctx.accounts.admin_settings;
    admin_settings.admin = admin;
    
    Ok(())
}
