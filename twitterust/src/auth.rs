use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::BTreeMap;

type HmacSha1 = Hmac<Sha1>;

/// Twitter API credentials for OAuth 1.0a authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
}

impl TwitterCredentials {
    /// Create new credentials
    pub fn new(
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
        access_token: impl Into<String>,
        access_token_secret: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            access_token: access_token.into(),
            access_token_secret: access_token_secret.into(),
        }
    }

    /// Load credentials from environment variables
    ///
    /// Expected variables:
    /// - TWITTER_API_KEY
    /// - TWITTER_API_SECRET  
    /// - TWITTER_ACCESS_TOKEN
    /// - TWITTER_ACCESS_TOKEN_SECRET
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            api_key: std::env::var("TWITTER_API_KEY")?,
            api_secret: std::env::var("TWITTER_API_SECRET")?,
            access_token: std::env::var("TWITTER_ACCESS_TOKEN")?,
            access_token_secret: std::env::var("TWITTER_ACCESS_TOKEN_SECRET")?,
        })
    }
}

/// OAuth 1.0a signature generator
pub struct OAuthSigner<'a> {
    credentials: &'a TwitterCredentials,
}

impl<'a> OAuthSigner<'a> {
    pub fn new(credentials: &'a TwitterCredentials) -> Self {
        Self { credentials }
    }

    /// Generate OAuth 1.0a authorization header
    pub fn generate_auth_header(&self, method: &str, url: &str) -> String {
        // OAuth percent encoding
        const OAUTH_ENCODE_SET: &AsciiSet = &CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'#')
            .add(b'%')
            .add(b'&')
            .add(b'+')
            .add(b',')
            .add(b'/')
            .add(b':')
            .add(b';')
            .add(b'<')
            .add(b'=')
            .add(b'>')
            .add(b'?')
            .add(b'@')
            .add(b'[')
            .add(b'\\')
            .add(b']')
            .add(b'^')
            .add(b'`')
            .add(b'{')
            .add(b'|')
            .add(b'}')
            .add(b'~')
            .add(b'!')
            .add(b'*')
            .add(b'\'')
            .add(b'(')
            .add(b')');

        let timestamp = chrono::Utc::now().timestamp().to_string();
        let nonce = format!("{:x}{:x}", rand::random::<u64>(), rand::random::<u64>());

        // Build OAuth parameters
        let mut oauth_params = BTreeMap::new();
        oauth_params.insert("oauth_consumer_key", self.credentials.api_key.as_str());
        oauth_params.insert("oauth_nonce", nonce.as_str());
        oauth_params.insert("oauth_signature_method", "HMAC-SHA1");
        oauth_params.insert("oauth_timestamp", timestamp.as_str());
        oauth_params.insert("oauth_token", self.credentials.access_token.as_str());
        oauth_params.insert("oauth_version", "1.0");

        // Create parameter string
        let param_string = oauth_params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    utf8_percent_encode(k, OAUTH_ENCODE_SET),
                    utf8_percent_encode(v, OAUTH_ENCODE_SET)
                )
            })
            .collect::<Vec<_>>()
            .join("&");

        // Create signature base string
        let base_string = format!(
            "{}&{}&{}",
            method.to_uppercase(),
            utf8_percent_encode(url, OAUTH_ENCODE_SET),
            utf8_percent_encode(&param_string, OAUTH_ENCODE_SET)
        );

        // Create signing key
        let signing_key = format!(
            "{}&{}",
            utf8_percent_encode(&self.credentials.api_secret, OAUTH_ENCODE_SET),
            utf8_percent_encode(&self.credentials.access_token_secret, OAUTH_ENCODE_SET)
        );

        // Generate signature
        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes()).unwrap();
        mac.update(base_string.as_bytes());
        let signature = base64::encode(mac.finalize().into_bytes());

        // Build authorization header
        format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_nonce=\"{}\", oauth_signature=\"{}\", oauth_signature_method=\"HMAC-SHA1\", oauth_timestamp=\"{}\", oauth_token=\"{}\", oauth_version=\"1.0\"",
            utf8_percent_encode(&self.credentials.api_key, OAUTH_ENCODE_SET),
            utf8_percent_encode(&nonce, OAUTH_ENCODE_SET),
            utf8_percent_encode(&signature, OAUTH_ENCODE_SET),
            utf8_percent_encode(&timestamp, OAUTH_ENCODE_SET),
            utf8_percent_encode(&self.credentials.access_token, OAUTH_ENCODE_SET)
        )
    }
}
