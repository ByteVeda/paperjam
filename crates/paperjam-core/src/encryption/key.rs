/// PDF encryption key derivation algorithms (PDF spec 1.7, section 3.5).

use md5::{Digest, Md5};

use super::rc4::Rc4;

/// Standard PDF password padding bytes (Table 3.18).
pub const PAD_BYTES: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01,
    0x08, 0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53,
    0x69, 0x7A,
];

/// Pad or truncate a password to exactly 32 bytes using PDF padding.
pub fn pad_password(password: &[u8]) -> [u8; 32] {
    let mut padded = [0u8; 32];
    let len = password.len().min(32);
    padded[..len].copy_from_slice(&password[..len]);
    if len < 32 {
        padded[len..].copy_from_slice(&PAD_BYTES[..32 - len]);
    }
    padded
}

/// Algorithm 3.3: Compute the owner password hash (/O value).
pub fn compute_owner_hash(owner_pw: &[u8], user_pw: &[u8], key_len: usize) -> [u8; 32] {
    // Step a: Pad owner password
    let padded_owner = pad_password(owner_pw);

    // Step b: MD5 hash it
    let mut hash = Md5::digest(padded_owner);

    // Step c: (R>=3) Rehash 50 times
    for _ in 0..50 {
        hash = Md5::digest(&hash[..key_len]);
    }

    // Step d: Use first key_len bytes as RC4 key
    let rc4_key = &hash[..key_len];

    // Step e: Pad user password
    let padded_user = pad_password(user_pw);

    // Step f: RC4-encrypt the padded user password
    let mut result = Rc4::new(rc4_key).process(&padded_user);

    // Step g: (R>=3) 19 additional RC4 rounds with modified keys
    for i in 1..=19u8 {
        let modified_key: Vec<u8> = rc4_key.iter().map(|&b| b ^ i).collect();
        result = Rc4::new(&modified_key).process(&result);
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Algorithm 3.2: Compute the encryption key.
pub fn compute_encryption_key(
    user_pw: &[u8],
    o_hash: &[u8],
    permissions: i32,
    file_id: &[u8],
    key_len: usize,
) -> Vec<u8> {
    let padded = pad_password(user_pw);

    let mut hasher = Md5::new();
    hasher.update(padded);
    hasher.update(o_hash);
    hasher.update(permissions.to_le_bytes());
    hasher.update(file_id);

    let mut hash = hasher.finalize();

    // R>=3: rehash 50 times
    for _ in 0..50 {
        hash = Md5::digest(&hash[..key_len]);
    }

    hash[..key_len].to_vec()
}

/// Algorithm 3.5: Compute the user password hash (/U value) for R=3.
pub fn compute_user_hash(key: &[u8], file_id: &[u8]) -> [u8; 32] {
    // Step a: MD5 of padding + file ID
    let mut hasher = Md5::new();
    hasher.update(PAD_BYTES);
    hasher.update(file_id);
    let hash = hasher.finalize();

    // Step b: RC4-encrypt the 16-byte hash
    let mut result = Rc4::new(key).process(&hash);

    // Step c: 19 additional RC4 rounds
    for i in 1..=19u8 {
        let modified_key: Vec<u8> = key.iter().map(|&b| b ^ i).collect();
        result = Rc4::new(&modified_key).process(&result);
    }

    // Pad to 32 bytes with arbitrary data (per spec)
    let mut out = [0u8; 32];
    out[..16].copy_from_slice(&result[..16]);
    // Fill remaining 16 bytes with zeros (common practice)
    out
}

/// Compute the per-object encryption key and encrypt data.
pub fn encrypt_object(key: &[u8], obj_num: u32, gen_num: u16, data: &[u8]) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(key);
    hasher.update(&obj_num.to_le_bytes()[..3]);
    hasher.update(&gen_num.to_le_bytes()[..2]);

    let obj_key_full = hasher.finalize();
    let obj_key_len = (key.len() + 5).min(16);
    let obj_key = &obj_key_full[..obj_key_len];

    Rc4::new(obj_key).process(data)
}
