use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;

pub fn generate_token(user_id: String) -> Result<String, String> {
    let key: Hmac<Sha384> = Hmac::new_from_slice(b"some-secret").map_err(|err| err.to_string())?;
    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };
    let mut claims = BTreeMap::new();
    claims.insert("sub", user_id);
    let binding = Token::new(header, claims)
        .sign_with_key(&key)
        .map_err(|err| err.to_string())?;
    let token_string = binding.as_str();
    Ok(token_string.to_owned())
}

pub fn decode_token(user_id: String, token: String) -> Result<String, String> {
    let key: Hmac<Sha384> = Hmac::new_from_slice(b"some-secret").unwrap();
    let token: Token<Header, BTreeMap<String, String>, _> =
        VerifyWithKey::verify_with_key(&*token, &key).unwrap();
    let claims = token.claims();
    if claims["sub"] != user_id {
        log::error!("token mismatch with given user_id");
        return Err("token mismatch with given user_id".to_string());
    }
    Ok("token matched with given user_id".to_string())
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_token_generate_and_decode() {
        let user_id = String::from("user_id1");
        let token = generate_token(user_id.clone()).unwrap();
        let decode_res: String = decode_token(user_id.clone(), token).unwrap();
        assert_eq!("token matched with given user_id".to_string(), decode_res)

    }
}