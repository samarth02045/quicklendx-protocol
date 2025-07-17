use soroban_sdk::{contracterror, symbol_short, Symbol};

/// Custom error types for the QuickLendX contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum QuickLendXError {
    // Invoice errors (1000-1099)
    InvoiceNotFound = 1000,
    InvoiceAlreadyExists = 1001,
    InvoiceNotAvailableForFunding = 1002,
    InvoiceAlreadyFunded = 1003,
    InvoiceAmountInvalid = 1004,
    InvoiceDueDateInvalid = 1005,
    InvoiceNotVerified = 1006,
    InvoiceNotFunded = 1007,
    InvoiceAlreadyPaid = 1008,
    InvoiceAlreadyDefaulted = 1009,

    // Authorization errors (1100-1199)
    Unauthorized = 1100,
    NotBusinessOwner = 1101,
    NotInvestor = 1102,
    NotAdmin = 1103,

    // Validation errors (1200-1299)
    InvalidAmount = 1200,
    InvalidAddress = 1201,
    InvalidCurrency = 1202,
    InvalidTimestamp = 1203,
    InvalidDescription = 1204,

    // Storage errors (1300-1399)
    StorageError = 1300,
    StorageKeyNotFound = 1301,

    // Business logic errors (1400-1499)
    InsufficientFunds = 1400,
    InvalidStatus = 1401,
    OperationNotAllowed = 1402,
}

impl From<QuickLendXError> for Symbol {
    fn from(error: QuickLendXError) -> Self {
        match error {
            QuickLendXError::InvoiceNotFound => symbol_short!("INV_NOT_FOUND"),
            QuickLendXError::InvoiceAlreadyExists => symbol_short!("INV_EXISTS"),
            QuickLendXError::InvoiceNotAvailableForFunding => symbol_short!("INV_NOT_AVAIL"),
            QuickLendXError::InvoiceAlreadyFunded => symbol_short!("INV_FUNDED"),
            QuickLendXError::InvoiceAmountInvalid => symbol_short!("INV_AMT_INV"),
            QuickLendXError::InvoiceDueDateInvalid => symbol_short!("INV_DATE_INV"),
            QuickLendXError::InvoiceNotVerified => symbol_short!("INV_NOT_VER"),
            QuickLendXError::InvoiceNotFunded => symbol_short!("INV_NOT_FUND"),
            QuickLendXError::InvoiceAlreadyPaid => symbol_short!("INV_PAID"),
            QuickLendXError::InvoiceAlreadyDefaulted => symbol_short!("INV_DEFAULT"),
            QuickLendXError::Unauthorized => symbol_short!("UNAUTH"),
            QuickLendXError::NotBusinessOwner => symbol_short!("NOT_OWNER"),
            QuickLendXError::NotInvestor => symbol_short!("NOT_INVESTOR"),
            QuickLendXError::NotAdmin => symbol_short!("NOT_ADMIN"),
            QuickLendXError::InvalidAmount => symbol_short!("INV_AMT"),
            QuickLendXError::InvalidAddress => symbol_short!("INV_ADDR"),
            QuickLendXError::InvalidCurrency => symbol_short!("INV_CURR"),
            QuickLendXError::InvalidTimestamp => symbol_short!("INV_TIME"),
            QuickLendXError::InvalidDescription => symbol_short!("INV_DESC"),
            QuickLendXError::StorageError => symbol_short!("STORAGE_ERR"),
            QuickLendXError::StorageKeyNotFound => symbol_short!("KEY_NOT_FOUND"),
            QuickLendXError::InsufficientFunds => symbol_short!("INSUF_FUNDS"),
            QuickLendXError::InvalidStatus => symbol_short!("INV_STATUS"),
            QuickLendXError::OperationNotAllowed => symbol_short!("OP_NOT_ALLOW"),
        }
    }
} 