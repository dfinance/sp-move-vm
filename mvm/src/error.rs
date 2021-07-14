#[derive(Debug)]
pub struct SubStatus {
    pub category: Category,
    pub reason: u64,
}

impl SubStatus {
    pub fn new(code: u64) -> SubStatus {
        let category = code as u8;
        let reason = code >> 8;

        let category = match category {
            1 => Category::INVALID_STATE,
            2 => Category::REQUIRES_ADDRESS,
            3 => Category::REQUIRES_ROLE,
            4 => Category::REQUIRES_CAPABILITY,
            5 => Category::NOT_PUBLISHED,
            6 => Category::ALREADY_PUBLISHED,
            7 => Category::INVALID_ARGUMENT,
            8 => Category::LIMIT_EXCEEDED,
            10 => Category::INTERNAL,
            _ => Category::CUSTOM,
        };

        SubStatus { category, reason }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Category {
    /// The system is in a state where the performed operation is not allowed. Example: call to a function only allowed
    /// in genesis.
    INVALID_STATE,
    /// The signer of a transaction does not have the expected address for this operation. Example: a call to a function
    /// which publishes a resource under a particular address.
    REQUIRES_ADDRESS,
    /// The signer of a transaction does not have the expected  role for this operation. Example: a call to a function
    /// which requires the signer to have the role of treasury compliance.
    REQUIRES_ROLE,
    /// The signer of a transaction does not have a required capability.
    REQUIRES_CAPABILITY,
    /// A resource is required but not published. Example: access to non-existing AccountLimits resource.
    NOT_PUBLISHED,
    /// Attempting to publish a resource that is already published. Example: calling an initialization function
    /// twice.
    ALREADY_PUBLISHED,
    /// An argument provided to an operation is invalid. Example: a signing key has the wrong format.
    INVALID_ARGUMENT,
    /// A limit on an amount, e.g. a currency, is exceeded. Example: withdrawal of money after account limits window
    /// is exhausted.
    LIMIT_EXCEEDED,
    /// An internal error (bug) has occurred.
    INTERNAL,
    /// A custom error category for extension points.
    CUSTOM,
}
