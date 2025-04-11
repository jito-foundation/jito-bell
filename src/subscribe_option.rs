pub struct SubscribeOptions {
    pub endpoint: String,
    pub x_token: Option<String>,
    pub commitment: CommitmentLevel,
    pub vote: Option<bool>,
    pub failed: Option<bool>,
    pub signature: Option<String>,
    pub account_include: Vec<String>,
    pub account_exclude: Vec<String>,
    pub account_required: Vec<String>,
}
