use anchor_lang::prelude::*;

#[error_code]
pub enum EditionsError {
    #[msg("Ticker too long")]
    TickerTooLong,

    #[msg("Mint template too long")]
    MintTemplateTooLong,


    #[msg("Deployment template too long")]
    DeploymentTemplateTooLong,

    #[msg("Root type too long")]
    RootTypeTooLong,

    #[msg("Minted out")]
    MintedOut,

    #[msg("Legacy migrations are minted out")]
    LegacyMigrationsAreMintedOut,

    #[msg("Global tree delegate is missing")]
    MissingGlobalTreeDelegate,

    #[msg("Incorrect mint type")]
    IncorrectMintType,

    #[msg("Invalid Metadata")]
    InvalidMetadata,

    #[msg("Creator fee too high")]
    CreatorFeeTooHigh,

    #[msg("Platform fee calculation failed.")]
    FeeCalculationError,

    #[msg("Total fee exceeds the price amount.")]
    FeeExceedsPrice,

    #[msg("Total fee shares must equal 100.")]
    InvalidFeeShares,

    #[msg("Too many platform fee recipients. Maximum allowed is 5.")]
    TooManyRecipients,

    #[msg("Recipient account does not match the expected address.")]
    RecipientMismatch,
}
