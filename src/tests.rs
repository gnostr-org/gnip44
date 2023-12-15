use crate::*;
use secp256k1::{SecretKey, XOnlyPublicKey, SECP256K1};

// We use the test vectors from Paul Miller's javascript so we don't accidently
// mistype anything
const JSON_VECTORS: &'static str = include_str!("nip44.vectors.json");

#[test]
fn test_valid_get_conversation_key() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    // v2.valid.get_conversation_key[]
    for vectorobj in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("valid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("get_conversation_key")
        .unwrap()
        .as_array()
        .unwrap()
    {
        let vector = vectorobj.as_object().unwrap();

        let sec1 = {
            let sec1hex = vector.get("sec1").unwrap().as_str().unwrap();
            let sec1bytes = hex::decode(sec1hex).unwrap();
            SecretKey::from_slice(&sec1bytes).unwrap()
        };
        let pub2 = {
            let pub2hex = vector.get("pub2").unwrap().as_str().unwrap();
            let pub2bytes = hex::decode(pub2hex).unwrap();
            XOnlyPublicKey::from_slice(&pub2bytes).unwrap()
        };
        let conversation_key: [u8; 32] = {
            let ckeyhex = vector.get("conversation_key").unwrap().as_str().unwrap();
            hex::decode(ckeyhex).unwrap().try_into().unwrap()
        };
        let note = vector.get("note").unwrap().as_str().unwrap();

        let computed_conversation_key = get_conversation_key(sec1, pub2);

        assert_eq!(
            conversation_key, computed_conversation_key,
            "Conversation key failure on {}",
            note
        );
    }
}

#[test]
fn test_valid_calc_padded_len() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    for elem in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("valid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("calc_padded_len")
        .unwrap()
        .as_array()
        .unwrap()
    {
        let len = elem[0].as_number().unwrap().as_u64().unwrap() as usize;
        let pad = elem[1].as_number().unwrap().as_u64().unwrap() as usize;
        assert_eq!(calc_padding(len), pad);
    }
}

#[test]
fn test_valid_encrypt_decrypt() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    for (i, vectorobj) in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("valid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("encrypt_decrypt")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let vector = vectorobj.as_object().unwrap();

        let sec1 = {
            let sec1hex = vector.get("sec1").unwrap().as_str().unwrap();
            let sec1bytes = hex::decode(sec1hex).unwrap();
            SecretKey::from_slice(&sec1bytes).unwrap()
        };
        let sec2 = {
            let sec2hex = vector.get("sec2").unwrap().as_str().unwrap();
            let sec2bytes = hex::decode(sec2hex).unwrap();
            SecretKey::from_slice(&sec2bytes).unwrap()
        };
        let conversation_key: [u8; 32] = {
            let ckeyhex = vector.get("conversation_key").unwrap().as_str().unwrap();
            hex::decode(ckeyhex).unwrap().try_into().unwrap()
        };
        let nonce: [u8; 32] = {
            let noncehex = vector.get("nonce").unwrap().as_str().unwrap();
            hex::decode(noncehex).unwrap().try_into().unwrap()
        };
        let plaintext = vector.get("plaintext").unwrap().as_str().unwrap();
        let ciphertext = vector.get("ciphertext").unwrap().as_str().unwrap();

        // Test conversation key
        let computed_conversation_key =
            get_conversation_key(sec1, sec2.x_only_public_key(&SECP256K1).0);
        assert_eq!(
            computed_conversation_key, conversation_key,
            "Conversation key failure on ValidSec #{}",
            i
        );

        // Test encryption with an overridden nonce
        let computed_ciphertext =
            encrypt_inner(&conversation_key, &plaintext, Some(&nonce)).unwrap();
        assert_eq!(
            computed_ciphertext, ciphertext,
            "Encryption does not match on ValidSec #{}",
            i
        );

        // Test decryption
        let computed_plaintext = decrypt(&conversation_key, &ciphertext).unwrap();
        assert_eq!(
            computed_plaintext, plaintext,
            "Decryption does not match on ValidSec #{}",
            i
        );
    }
}

//TBD?
//#[test]
//fn test_valid_encrypt_decrypt_long_msg() {
//}

//TBD?
//#[test]
//fn test_invalid_encrypt_msg_lengths() {
//}

//TBD?
//#[test]
//fn test_invalid_decrypt_msg_lengths() {
//}

#[test]
fn test_invalid_get_conversation_key() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    for vectorobj in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("invalid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("get_conversation_key")
        .unwrap()
        .as_array()
        .unwrap()
    {
        let vector = vectorobj.as_object().unwrap();

        let sec1result = {
            let sec1hex = vector.get("sec1").unwrap().as_str().unwrap();
            let sec1bytes = hex::decode(sec1hex).unwrap();
            SecretKey::from_slice(&sec1bytes)
        };
        let pub2result = {
            let pub2hex = vector.get("pub2").unwrap().as_str().unwrap();
            let pub2bytes = hex::decode(pub2hex).unwrap();
            XOnlyPublicKey::from_slice(&pub2bytes)
        };
        let note = vector.get("note").unwrap().as_str().unwrap();

        assert!(
            sec1result.is_err() || pub2result.is_err(),
            "One of the keys should have failed: {}",
            note
        );
    }
}

#[test]
fn test_invalid_decrypt() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    let known_errors = [
        Error::UnsupportedFutureVersion,
        Error::UnknownVersion,
        Error::Base64Decode(base64::DecodeError::InvalidLength),
        Error::InvalidMac,
        Error::InvalidMac,
        Error::InvalidPadding,
        Error::MessageIsEmpty,
        Error::InvalidPadding,
        Error::InvalidPadding,
    ];

    for (i, vectorobj) in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("invalid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("decrypt")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let vector = vectorobj.as_object().unwrap();
        let conversation_key: [u8; 32] = {
            let ckeyhex = vector.get("conversation_key").unwrap().as_str().unwrap();
            hex::decode(ckeyhex).unwrap().try_into().unwrap()
        };
        //let nonce: [u8; 32] = {
        //    let noncehex = vector.get("nonce").unwrap().as_str().unwrap();
        //    hex::decode(noncehex).unwrap().try_into().unwrap()
        //};
        // let plaintext = vector.get("plaintext").unwrap().as_str().unwrap();
        let ciphertext = vector.get("ciphertext").unwrap().as_str().unwrap();
        let note = vector.get("note").unwrap().as_str().unwrap();

        let result = decrypt(&conversation_key, &ciphertext);
        assert!(result.is_err(), "Should not have decrypted: {}", note);

        let err = result.unwrap_err();
        assert_eq!(
            err, known_errors[i],
            "Unexpected error in invalid decrypt #{}",
            i
        );
    }
}
