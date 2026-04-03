//! PDF encryption key derivation algorithms.
//!
//! - PDF 1.7 (section 3.5): MD5-based for RC4/AES-128 (V=2..4, R=3..4).
//! - PDF 2.0 (ISO 32000-2, section 7.6.4.3): SHA-based for AES-256 (V=5, R=6).

use md5::{Digest as Md5Digest, Md5};
use sha2::digest::Digest as Sha2Digest;
use sha2::{Sha256, Sha384, Sha512};

use super::rc4::Rc4;

/// Standard PDF password padding bytes (Table 3.18).
pub const PAD_BYTES: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01, 0x08,
    0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
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

/// Compute the per-object encryption key and encrypt data with RC4.
pub fn encrypt_object(key: &[u8], obj_num: u32, gen_num: u16, data: &[u8]) -> Vec<u8> {
    let obj_key = derive_object_key_rc4(key, obj_num, gen_num);
    Rc4::new(&obj_key).process(data)
}

/// Derive the per-object key for RC4 encryption.
fn derive_object_key_rc4(key: &[u8], obj_num: u32, gen_num: u16) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(key);
    hasher.update(&obj_num.to_le_bytes()[..3]);
    hasher.update(&gen_num.to_le_bytes()[..2]);

    let obj_key_full = hasher.finalize();
    let obj_key_len = (key.len() + 5).min(16);
    obj_key_full[..obj_key_len].to_vec()
}

/// Compute the per-object encryption key and encrypt data with AES-128-CBC.
///
/// Per PDF spec: same MD5 derivation as RC4, but appends "sAlT" bytes before hashing.
/// Generates a random 16-byte IV and returns IV + ciphertext.
pub fn encrypt_object_aes128(key: &[u8], obj_num: u32, gen_num: u16, data: &[u8]) -> Vec<u8> {
    use rand::Rng;

    let mut hasher = Md5::new();
    hasher.update(key);
    hasher.update(&obj_num.to_le_bytes()[..3]);
    hasher.update(&gen_num.to_le_bytes()[..2]);
    // AES-specific: append "sAlT"
    hasher.update([0x73, 0x41, 0x6C, 0x54]);

    let obj_key_full = hasher.finalize();
    // For AES-128, the key is always 16 bytes
    let obj_key = &obj_key_full[..16];

    let iv: [u8; 16] = rand::thread_rng().gen();

    super::aes128::encrypt_aes128(&iv, obj_key, data)
}

// ---------------------------------------------------------------------------
// PDF 2.0 (V=5, R=6) — SHA-based key derivation for AES-256
// ---------------------------------------------------------------------------

/// Generate a random 32-byte file encryption key for AES-256.
pub fn generate_file_encryption_key() -> [u8; 32] {
    use rand::Rng;
    rand::thread_rng().gen()
}

/// ISO 32000-2 Algorithm 2.B — revision 6 iterative hash.
///
/// Used by both U/UE and O/OE computation. The `u_value` parameter is empty
/// for user-password operations and the 48-byte U value for owner-password
/// operations.
pub fn compute_hash_r6(password: &[u8], salt: &[u8], u_value: &[u8]) -> [u8; 32] {
    use aes::cipher::{block_padding::NoPadding, BlockEncryptMut, KeyIvInit};

    type Aes128CbcRaw = cbc::Encryptor<aes::Aes128>;

    // Step a: SHA-256 of password + salt + u_value
    let mut k = {
        let mut h = Sha256::new();
        h.update(password);
        h.update(salt);
        h.update(u_value);
        h.finalize()
    };

    let mut round = 0u32;
    loop {
        // Step b: build K1 = password + K + u_value, repeated 64 times
        let k1_single_len = password.len() + k.len() + u_value.len();
        let k1_len = k1_single_len * 64;
        let mut k1 = Vec::with_capacity(k1_len);
        for _ in 0..64 {
            k1.extend_from_slice(password);
            k1.extend_from_slice(&k);
            k1.extend_from_slice(u_value);
        }

        // Step c: AES-128-CBC encrypt K1 using first 16 bytes of K as key
        // and second 16 bytes of K as IV (no padding)
        let aes_key: [u8; 16] = k[..16].try_into().unwrap();
        let aes_iv: [u8; 16] = k[16..32].try_into().unwrap();

        // Pad K1 to a multiple of 16 for AES-CBC
        let pad_len = (16 - (k1.len() % 16)) % 16;
        let data_len = k1.len();
        k1.resize(data_len + pad_len, 0);
        let buf_len = k1.len();

        let cipher = Aes128CbcRaw::new(&aes_key.into(), &aes_iv.into());
        let encrypted = cipher
            .encrypt_padded_mut::<NoPadding>(&mut k1, buf_len)
            .expect("buffer is block-aligned");

        // Step d: take the first 16 bytes of E, interpret as big-endian u128,
        // mod 3 to select hash algorithm
        let remainder = {
            let mut sum: u32 = 0;
            for &b in encrypted.iter().take(16) {
                sum = sum.wrapping_add(b as u32);
            }
            sum % 3
        };

        // Step e: hash E with the selected algorithm
        k = match remainder {
            0 => Sha256::digest(encrypted),
            1 => {
                let result = Sha384::digest(encrypted);
                let mut out = sha2::digest::Output::<Sha256>::default();
                out.copy_from_slice(&result[..32]);
                out
            }
            _ => {
                let result = Sha512::digest(encrypted);
                let mut out = sha2::digest::Output::<Sha256>::default();
                out.copy_from_slice(&result[..32]);
                out
            }
        };

        // Step f: check termination: round >= 64 and last byte of E <= round - 32
        let last_byte = *encrypted.last().unwrap_or(&0);
        round += 1;
        if round >= 64 && (last_byte as u32) <= round - 32 {
            break;
        }
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&k[..32]);
    out
}

/// Compute /U (48 bytes) and /UE (32 bytes) for R=6.
///
/// - U = SHA-256(password + validation_salt) [32 bytes] + validation_salt [8] + key_salt [8]
/// - UE = AES-256-CBC(intermediate_key, iv=0, file_key) [32 bytes]
pub fn compute_u_value_r6(password: &[u8], file_key: &[u8; 32]) -> ([u8; 48], [u8; 32]) {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let validation_salt: [u8; 8] = rng.gen();
    let key_salt: [u8; 8] = rng.gen();

    // U hash = SHA-256(password + validation_salt)
    let u_hash = {
        let mut h = Sha256::new();
        h.update(password);
        h.update(validation_salt);
        h.finalize()
    };

    let mut u_value = [0u8; 48];
    u_value[..32].copy_from_slice(&u_hash);
    u_value[32..40].copy_from_slice(&validation_salt);
    u_value[40..48].copy_from_slice(&key_salt);

    // UE: intermediate key from Algorithm 2.B with key_salt, then AES-256-CBC encrypt file_key
    let intermediate_key = compute_hash_r6(password, &key_salt, &[]);
    let ue_value = aes256_cbc_no_pad_encrypt(&intermediate_key, &[0u8; 16], file_key);

    let mut ue = [0u8; 32];
    ue.copy_from_slice(&ue_value[..32]);

    (u_value, ue)
}

/// Compute /O (48 bytes) and /OE (32 bytes) for R=6.
///
/// Same structure as U/UE, but hash computations include the 48-byte U value.
pub fn compute_o_value_r6(
    password: &[u8],
    file_key: &[u8; 32],
    u_value: &[u8; 48],
) -> ([u8; 48], [u8; 32]) {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let validation_salt: [u8; 8] = rng.gen();
    let key_salt: [u8; 8] = rng.gen();

    // O hash = SHA-256(password + validation_salt + U)
    let o_hash = {
        let mut h = Sha256::new();
        h.update(password);
        h.update(validation_salt);
        h.update(u_value);
        h.finalize()
    };

    let mut o_value = [0u8; 48];
    o_value[..32].copy_from_slice(&o_hash);
    o_value[32..40].copy_from_slice(&validation_salt);
    o_value[40..48].copy_from_slice(&key_salt);

    // OE: intermediate key from Algorithm 2.B with key_salt and U, then encrypt file_key
    let intermediate_key = compute_hash_r6(password, &key_salt, u_value);
    let oe_value = aes256_cbc_no_pad_encrypt(&intermediate_key, &[0u8; 16], file_key);

    let mut oe = [0u8; 32];
    oe.copy_from_slice(&oe_value[..32]);

    (o_value, oe)
}

/// Compute the /Perms entry for R=6 (16 bytes, AES-256-ECB encrypted).
pub fn compute_perms_value_r6(
    permissions: i32,
    file_key: &[u8; 32],
    encrypt_metadata: bool,
) -> [u8; 16] {
    use rand::Rng;

    let mut block = [0u8; 16];
    // Bytes 0-3: permissions as little-endian u32
    block[..4].copy_from_slice(&(permissions as u32).to_le_bytes());
    // Bytes 4-7: 0xFFFFFFFF
    block[4..8].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    // Byte 8: 'T' or 'F'
    block[8] = if encrypt_metadata { b'T' } else { b'F' };
    // Bytes 9-11: 'adb'
    block[9] = b'a';
    block[10] = b'd';
    block[11] = b'b';
    // Bytes 12-15: random
    let random_bytes: [u8; 4] = rand::thread_rng().gen();
    block[12..16].copy_from_slice(&random_bytes);

    super::aes256::encrypt_aes256_ecb_block(file_key, &block)
}

/// AES-256-CBC encrypt exactly 32 bytes without padding (for UE/OE computation).
fn aes256_cbc_no_pad_encrypt(key: &[u8; 32], iv: &[u8; 16], data: &[u8; 32]) -> Vec<u8> {
    use aes::cipher::{block_padding::NoPadding, BlockEncryptMut, KeyIvInit};

    type Aes256CbcRaw = cbc::Encryptor<aes::Aes256>;

    let cipher = Aes256CbcRaw::new(key.into(), iv.into());
    let mut buf = [0u8; 32];
    buf.copy_from_slice(data);

    let ciphertext = cipher
        .encrypt_padded_mut::<NoPadding>(&mut buf, 32)
        .expect("32 bytes is block-aligned");

    ciphertext.to_vec()
}
