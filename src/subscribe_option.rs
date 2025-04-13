use yellowstone_grpc_proto::geyser::CommitmentLevel;

pub struct SubscribeOption {
    /// Endpoint
    pub endpoint: String,

    /// X-Token
    pub x_token: Option<String>,

    /// Commitment
    pub commitment: CommitmentLevel,

    /// Vote
    pub vote: Option<bool>,

    /// Failed
    pub failed: Option<bool>,

    /// Signature
    pub signature: Option<String>,

    /// Acount include
    pub account_include: Vec<String>,

    /// Account exclude
    pub account_exclude: Vec<String>,

    /// Account required
    pub account_required: Vec<String>,
}

impl SubscribeOption {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        endpoint: String,
        x_token: Option<String>,
        commitment: CommitmentLevel,
        vote: Option<bool>,
        failed: Option<bool>,
        signature: Option<String>,
        account_include: Vec<String>,
        account_exclude: Vec<String>,
        account_required: Vec<String>,
    ) -> Self {
        Self {
            endpoint,
            x_token,
            commitment,
            vote,
            failed,
            signature,
            account_include,
            account_exclude,
            account_required,
        }
    }
}
