use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde_json::Value;
use thiserror::Error;

use crate::services::settings::Config;

#[derive(Error, Debug)]
pub(crate) enum AuthError {
    #[error("missing token")]
    MissingToken,
    #[error("invalid token")]
    InvalidToken,
    #[error("jwt config missing")]
    ConfigMissing,
}

pub(crate) async fn verify_jwt(config: &Config, token: &str) -> Result<Value, AuthError> {
    let alg = match config.jwt_alg.as_str() {
        "RS256" => Algorithm::RS256,
        "RS384" => Algorithm::RS384,
        "RS512" => Algorithm::RS512,
        "HS256" => Algorithm::HS256,
        "HS384" => Algorithm::HS384,
        "HS512" => Algorithm::HS512,
        _ => Algorithm::RS256,
    };

    let mut validation = Validation::new(alg);
    if !config.jwt_issuer.is_empty() {
        validation.set_issuer(&[config.jwt_issuer.clone()]);
    }
    if !config.jwt_audience.is_empty() {
        validation.set_audience(&[config.jwt_audience.clone()]);
    }
    if config.jwt_leeway > 0 {
        validation.leeway = config.jwt_leeway as u64;
    }

    if !config.jwt_jwks_url.is_empty() {
        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
        let kid = header.kid.ok_or(AuthError::InvalidToken)?;
        let jwks = fetch_jwks(&config.jwt_jwks_url)
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        let jwk = jwks
            .keys
            .into_iter()
            .find(|key| key.common.key_id.as_deref() == Some(&kid));
        if let Some(jwk) = jwk {
            let decoding_key = DecodingKey::from_jwk(&jwk).map_err(|_| AuthError::InvalidToken)?;
            let token_data =
                decode::<Value>(token, &decoding_key, &validation).map_err(|_| AuthError::InvalidToken)?;
            return Ok(token_data.claims);
        }
        return Err(AuthError::InvalidToken);
    }

    if !config.jwt_public_key.is_empty() {
        let decoding_key = match alg {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                DecodingKey::from_secret(config.jwt_public_key.as_bytes())
            }
            _ => DecodingKey::from_rsa_pem(config.jwt_public_key.as_bytes())
                .map_err(|_| AuthError::InvalidToken)?,
        };
        let token_data =
            decode::<Value>(token, &decoding_key, &validation).map_err(|_| AuthError::InvalidToken)?;
        return Ok(token_data.claims);
    }

    Err(AuthError::ConfigMissing)
}

async fn fetch_jwks(url: &str) -> Result<jsonwebtoken::jwk::JwkSet, reqwest::Error> {
    let client = reqwest::Client::builder().build()?;
    client.get(url).send().await?.json::<jsonwebtoken::jwk::JwkSet>().await
}
