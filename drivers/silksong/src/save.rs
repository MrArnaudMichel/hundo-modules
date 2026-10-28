//! The `userN.dat` format: a .NET BinaryFormatter string record holding
//! base64 of the JSON save, AES-256-ECB encrypted with a key shared by every
//! copy of the game.

use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};
use base64::Engine;
use hundo_sdk::DriverError;
use serde_json::Value;
use std::collections::HashMap;

const KEY: &[u8; 32] = b"UKu52ePUBwetZ9wNX88o54dnfKRu0T1l";
/// The BinaryFormatter header before the string record.
const HEADER: usize = 22;

pub fn decode(bytes: &[u8]) -> Result<Value, DriverError> {
    // A save shorter than its own header is one the game is still writing.
    if bytes.len() <= HEADER + 1 {
        return Err(DriverError::Incomplete("fichier tronqué".into()));
    }
    let mut data = &bytes[HEADER..bytes.len() - 1];
    // The string's length, as a 7-bit encoded integer.
    let len_bytes = data.iter().position(|b| b & 0x80 == 0).ok_or_else(corrupt)? + 1;
    data = &data[len_bytes..];

    let mut plain = base64::engine::general_purpose::STANDARD.decode(data).map_err(|_| corrupt())?;
    if plain.is_empty() || plain.len() % 16 != 0 {
        return Err(corrupt());
    }
    let cipher = aes::Aes256::new(GenericArray::from_slice(KEY));
    for block in plain.as_chunks_mut::<16>().0 {
        cipher.decrypt_block(GenericArray::from_mut_slice(block));
    }
    let pad = *plain.last().unwrap() as usize;
    if pad == 0 || pad > 16 {
        return Err(corrupt());
    }
    plain.truncate(plain.len() - pad);
    serde_json::from_slice(&plain).map_err(|e| DriverError::Corrupt(e.to_string()))
}

/// One of the game's language sheets (`FR_Journal`): base64 of the same
/// cipher, around `<entry name="KEY">text</entry>` lines. Read from the
/// installed game by the host; empty if it cannot be read.
pub fn sheet(bytes: &[u8]) -> HashMap<String, String> {
    let Ok(mut plain) = base64::engine::general_purpose::STANDARD.decode(bytes.trim_ascii()) else { return HashMap::new() };
    if plain.is_empty() || plain.len() % 16 != 0 {
        return HashMap::new();
    }
    let cipher = aes::Aes256::new(GenericArray::from_slice(KEY));
    for block in plain.as_chunks_mut::<16>().0 {
        cipher.decrypt_block(GenericArray::from_mut_slice(block));
    }
    let pad = *plain.last().unwrap() as usize;
    plain.truncate(plain.len().saturating_sub(pad.min(16)));
    let text = String::from_utf8_lossy(&plain);
    let mut out = HashMap::new();
    for entry in text.split("<entry name=\"").skip(1) {
        let Some((key, rest)) = entry.split_once("\">") else { continue };
        let Some((value, _)) = rest.split_once("</entry>") else { continue };
        out.insert(key.to_string(), unescape(value).trim().to_string());
    }
    out
}

/// XML entities, numeric ones included (`&#8217;`).
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let Some(end) = rest.find(';').filter(|&e| e <= 10) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let entity = &rest[1..end];
        let c = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            e => e.strip_prefix('#').and_then(|n| n.parse().ok()).and_then(char::from_u32),
        };
        match c {
            Some(c) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn corrupt() -> DriverError {
    DriverError::Corrupt("ce n'est pas une sauvegarde Silksong".into())
}
