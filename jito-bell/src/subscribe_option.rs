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

    /// Account include
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

impl std::fmt::Display for SubscribeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Subscribe Options:")?;
        writeln!(f, "  Endpoint: {}", self.endpoint)?;

        // Handle x_token securely - don't print actual token
        match &self.x_token {
            Some(_) => writeln!(f, "  X-Token: [REDACTED]")?,
            None => writeln!(f, "  X-Token: None")?,
        }

        // Display commitment level
        let commitment_str = match self.commitment {
            CommitmentLevel::Processed => "Processed",
            CommitmentLevel::Confirmed => "Confirmed",
            CommitmentLevel::Finalized => "Finalized",
        };
        writeln!(f, "  Commitment: {}", commitment_str)?;

        // Optional filter settings
        if let Some(vote) = self.vote {
            writeln!(f, "  Vote Filter: {}", vote)?;
        }

        if let Some(failed) = self.failed {
            writeln!(f, "  Failed Filter: {}", failed)?;
        }

        if let Some(sig) = &self.signature {
            writeln!(f, "  Signature Filter: {}", sig)?;
        }

        // Account filters
        if !self.account_include.is_empty() {
            writeln!(f, "  Account Include Filters:")?;
            for account in &self.account_include {
                writeln!(f, "    - {}", account)?;
            }
        }

        if !self.account_exclude.is_empty() {
            writeln!(f, "  Account Exclude Filters:")?;
            for account in &self.account_exclude {
                writeln!(f, "    - {}", account)?;
            }
        }

        if !self.account_required.is_empty() {
            writeln!(f, "  Account Required Filters:")?;
            for account in &self.account_required {
                writeln!(f, "    - {}", account)?;
            }
        }

        Ok(())
    }
}

// Add a Debug implementation that doesn't reveal sensitive information
impl std::fmt::Debug for SubscribeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Reuse the Display implementation for Debug
        write!(f, "{}", self)
    }
}
