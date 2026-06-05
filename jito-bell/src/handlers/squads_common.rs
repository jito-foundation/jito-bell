//! Shared types for Squads v3/v4 notification handling.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use solana_sdk::pubkey::Pubkey;

const SQUADS_V3_URL: &str = "https://v3.squads.so/transactions/{{transaction}}";
const SQUADS_V4_URL: &str =
    "https://app.squads.so/squads/{{multisig}}/transactions/{{transaction}}";

#[derive(Clone, Copy, Debug)]
pub enum SquadsContext {
    V3Transaction {
        multisig: Pubkey,
        transaction: Pubkey,
    },
    V4Proposal {
        multisig: Pubkey,
        proposal: Pubkey,
        transaction_index: u64,
    },
}

impl SquadsContext {
    pub(crate) fn header(self) -> &'static str {
        match self {
            Self::V3Transaction { .. } => "Squads Transaction Created",
            Self::V4Proposal { .. } => "Squads Proposal Created",
        }
    }

    pub(crate) fn link_label(self) -> &'static str {
        match self {
            Self::V3Transaction { .. } => "View Transaction",
            Self::V4Proposal { .. } => "View Proposal",
        }
    }

    pub(crate) fn account_label(self) -> &'static str {
        match self {
            Self::V3Transaction { .. } => "Squads Transaction",
            Self::V4Proposal { .. } => "Proposal",
        }
    }

    pub(crate) fn multisig(self) -> Pubkey {
        match self {
            Self::V3Transaction { multisig, .. } | Self::V4Proposal { multisig, .. } => multisig,
        }
    }

    pub(crate) fn account(self) -> Pubkey {
        match self {
            Self::V3Transaction { transaction, .. } => transaction,
            Self::V4Proposal { proposal, .. } => proposal,
        }
    }

    fn transaction_template_value(self) -> String {
        match self {
            Self::V3Transaction { transaction, .. } => STANDARD.encode(transaction.to_string()),
            Self::V4Proposal {
                transaction_index, ..
            } => transaction_index.to_string(),
        }
    }

    fn proposal_template_value(self) -> String {
        match self {
            Self::V3Transaction { .. } => String::new(),
            Self::V4Proposal { proposal, .. } => proposal.to_string(),
        }
    }

    pub(crate) fn transaction_index_field(self) -> Option<String> {
        match self {
            Self::V3Transaction { .. } => None,
            Self::V4Proposal {
                transaction_index, ..
            } => Some(transaction_index.to_string()),
        }
    }

    pub(crate) fn squads_url(self) -> String {
        let template = match self {
            Self::V3Transaction { .. } => SQUADS_V3_URL,
            Self::V4Proposal { .. } => SQUADS_V4_URL,
        };
        template
            .replace("{{multisig}}", &self.multisig().to_string())
            .replace("{{transaction}}", &self.transaction_template_value())
            .replace("{{proposal}}", &self.proposal_template_value())
    }

    pub fn build_slack_payload(
        self,
        description: &str,
        transaction_signature: &str,
        explorer_url: &str,
    ) -> serde_json::Value {
        let squads_url = self.squads_url();
        let mut fields = vec![
            serde_json::json!({
                "type": "mrkdwn",
                "text": format!("*Squads:* <{}|{}>", squads_url, self.link_label())
            }),
            serde_json::json!({
                "type": "mrkdwn",
                "text": format!(
                    "*Transaction:* <{}/tx/{}|View on Explorer>",
                    explorer_url, transaction_signature
                )
            }),
            serde_json::json!({
                "type": "mrkdwn",
                "text": format!("*Multisig:* `{}`", self.multisig())
            }),
            serde_json::json!({
                "type": "mrkdwn",
                "text": format!("*{}:* `{}`", self.account_label(), self.account())
            }),
        ];

        if let Some(transaction_index) = self.transaction_index_field() {
            fields.push(serde_json::json!({
                "type": "mrkdwn",
                "text": format!("*Transaction Index:* `{}`", transaction_index)
            }));
        }

        serde_json::json!({
            "blocks": [
                {
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": self.header()
                    }
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": format!("*Description:* {}", description)
                    }
                },
                {
                    "type": "section",
                    "fields": fields
                }
            ]
        })
    }
}
