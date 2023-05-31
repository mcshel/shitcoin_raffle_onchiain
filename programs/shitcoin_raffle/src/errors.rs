use anchor_lang::prelude::*;

#[error_code]
pub enum RaffleError {

    // 6000
    #[msg("Invalid calculation")]
    InvalidCalculation,

    // 6001
    #[msg("Start timestamp must be smaller than end timestamp")]
    StartAfterEndTimestamp,

    // 6002
    #[msg("End timestamp already passed")]
    EndTimestampAlreadyPassed,

    // 6003
    #[msg("Fee must be smaller than price")]
    FeeGreaterThanPrice,

    // 6004
    #[msg("Number of rewards must be smaller than number of tickets")]
    RewardsNumGreaterThanTickets,

    // 6005
    #[msg("Limit of available tickets per entrant must be greater than zero")]
    LimitLessThanOne,

    // 6006
    #[msg("The required amount of tickets is not available")]
    RaffleTicketsUnavailable,

    // 6007
    #[msg("The purchase would exceed the maximum amount of allowed tickets per user")]
    EntrantTicketLimitReached,

    // 6008
    #[msg("Raffle has not yet started")]
    RaffleNotStarted,

    // 6009
    #[msg("Raffle has ended")]
    RaffleEnded,

    // 6010
    #[msg("Number of reward tickets must be smaller than number of tickets")]
    RewardsNumGreaterThanTicketsBought,

    // 6011
    #[msg("Number of rewards greater than number of total rewards")]
    RewardsAmountGreaterThanTotal,

    // 6012
    #[msg("Raffle has been sold out")]
    RaffleSoldOut,

    // 6013
    #[msg("Raffle is still active")]
    RaffleStillActive,

    // 6014
    #[msg("Raffle rewards have not been set yet")]
    RaffleRewardsNotSet,

    // 6015
    #[msg("Not enough tickets left")]
    NotEnoughTicketsLeft,

    // 6016
    #[msg("Raffle admin has already claimed the proceeds")]
    RaffleAdminAlreadyClaimed,

    // 6017
    #[msg("Raffle admin has not claimed the proceeds yet")]
    RaffleAdminNotClaimed,

    // 6018
    #[msg("Not all raffle rewards have been claimed yet")]
    RaffleRewardsNotClaimed,

    // 6019
    #[msg("Entrant's reward has not been set yet")]
    EntrantNotAwarded,

    // 6020
    #[msg("Entrant's reward has already been set yet")]
    EntrantAlreadyAwarded,
}
