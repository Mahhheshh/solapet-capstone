use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Game configuration already initialized.")]
    ConfigAlreadyInitialized,
    #[msg("Invalid admin access.")]
    InvalidAdminAccess,
    #[msg("Fees percentage must be between 0 and 100.")]
    InvalidFeesPercentage,

    #[msg("Incorrect collection mint.")]
    IncorrectCollectionMint,
    #[msg("NFT already deposited.")]
    NftAlreadyDeposited,
    #[msg("NFT not deposited in game.")]
    NftNotDeposited,
    #[msg("Player does not own the NFT.")]
    PlayerDoesNotOwnNft,
    #[msg("Game ATA creation failed.")]
    GameAtaCreationFailed,

    #[msg("Invalid pet interaction.")]
    InvalidPetInteraction,
    #[msg("Pet stat is already at maximum.")]
    StatAlreadyMax,
    #[msg("Interaction not allowed in current pet state.")]
    InteractionNotAllowed,
    #[msg("Pet needs rest. Wait for energy to replenish.")]
    InsufficientPetEnergy,

    #[msg("Duel already challenged.")]
    DuelAlreadyChallenged,
    #[msg("Duel already started.")]
    DuelAlreadyStarted,
    #[msg("Cannot challenge yourself.")]
    CannotChallengeSelf,
    #[msg("Invalid bet amount.")]
    InvalidBetAmount,
    #[msg("Not enough funds to place bet.")]
    NotEnoughFundsForBet,

    #[msg("Not challenger's turn.")]
    NotChallengerTurn,
    #[msg("Not defender's turn.")]
    NotDefenderTurn,
    #[msg("Duel is not challenged yet.")]
    DuelNotChallenged,
    #[msg("Duel is finished.")]
    DuelFinished,
    #[msg("Duel is not finished yet.")]
    DuelNotFinished,
    #[msg("No winner declared for the duel.")]
    NoWinner,
    #[msg("Unknown winner.")]
    UnknownWinner,

    #[msg("Unauthorized action.")]
    UnauthorizedAction,
    #[msg("Only challenger can perform this action.")]
    OnlyChallengerAction,
    #[msg("Only defender can perform this action.")]
    OnlyDefenderAction,

    #[msg("System program transfer failed.")]
    SystemProgramTransferFailed,
    #[msg("Token program transfer failed.")]
    TokenProgramTransferFailed,
    #[msg("Account initialization failed.")]
    AccountInitializationFailed,
    #[msg("Account close failed.")]
    AccountCloseFailed,

    #[msg("Invalid account state.")]
    InvalidAccountState,
    #[msg("Operation overflow.")]
    Overflow,
    #[msg("Generic error.")]
    GenericError,

    #[msg("Account not provided")]
    AccountNotProvided,
    #[msg("Invalid signature format")]
    InvalidSig,
    #[msg("Signature length does not match expected length")]
    InvalidSigLength,
    #[msg("Signature verification failed")]
    SignatureNotVerified,
    #[msg("Signature data does not match expected data")]
    SigDataNoMatch
}
