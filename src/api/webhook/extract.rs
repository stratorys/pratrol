use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{
    FromRef,
    FromRequest,
    Request,
};
use hmac::{
    Hmac,
    KeyInit,
    Mac,
};
use sha2::Sha256;

use crate::api::error::ApiError;

type HmacSha256 = Hmac<Sha256>;

pub struct VerifiedBody(pub Bytes);

impl<S> FromRequest<S> for VerifiedBody
where
    S: Send + Sync,
    Arc<str>: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let secret = Arc::<str>::from_ref(state);

        let (parts, body) = req.into_parts();
        let signature = parts
            .headers
            .get("X-Hub-Signature-256")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
            .ok_or(ApiError::InvalidSignature)?;
        let request = Request::from_parts(parts, body);

        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|_| ApiError::InvalidPayload)?;

        verify_signature(&secret, &signature, &bytes)?;

        Ok(Self(bytes))
    }
}

fn verify_signature(
    secret: &str,
    signature_header: &str,
    body: &[u8],
) -> Result<(), ApiError> {
    let hex_signature = signature_header
        .strip_prefix("sha256=")
        .ok_or(ApiError::InvalidSignature)?;

    let signature_bytes = hex::decode(hex_signature).map_err(|_| ApiError::InvalidSignature)?;

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| ApiError::InvalidSignature)?;
    mac.update(body);
    mac.verify_slice(&signature_bytes)
        .map_err(|_| ApiError::InvalidSignature)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compute_signature(
        secret: &str,
        body: &[u8],
    ) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC key should be valid in test");
        mac.update(body);
        let result = mac.finalize().into_bytes();
        format!("sha256={}", hex::encode(result))
    }

    #[test]
    fn test_verify_signature_valid() {
        let secret = "test-secret";
        let body = b"test body";
        let signature = compute_signature(secret, body);

        let result = verify_signature(secret, &signature, body);
        assert!(result.is_ok(), "valid signature should pass");
    }

    #[test]
    fn test_verify_signature_wrong_digest() {
        let secret = "test-secret";
        let body = b"test body";
        let signature = "sha256=0000000000000000000000000000000000000000000000000000000000000000";

        let result = verify_signature(secret, signature, body);
        assert!(result.is_err(), "invalid signature should fail");
    }

    #[test]
    fn test_verify_signature_missing_prefix() {
        let secret = "test-secret";
        let body = b"test body";

        let result = verify_signature(secret, "deadbeef", body);
        assert!(
            result.is_err(),
            "signature without sha256= prefix should fail"
        );
    }
}
