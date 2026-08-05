//! AES-256 encryption for PDF objects (PDF 2.0, V=5, R=6).

use aes::cipher::{block_padding::Pkcs7, BlockEncrypt, BlockEncryptMut, KeyInit, KeyIvInit};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

/// Encrypt data using AES-256-CBC with PKCS#7 padding.
///
/// Returns IV (16 bytes) prepended to the ciphertext.
pub fn encrypt_aes256_cbc(iv: &[u8; 16], key: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let cipher = Aes256CbcEnc::new(key.into(), iv.into());

    // Allocate buffer with room for padding (up to 16 extra bytes)
    let mut buf = vec![0u8; data.len() + 16];
    buf[..data.len()].copy_from_slice(data);

    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buf, data.len())
        .expect("buffer is large enough for PKCS7 padding");

    let mut result = Vec::with_capacity(16 + ciphertext.len());
    result.extend_from_slice(iv);
    result.extend_from_slice(ciphertext);
    result
}

/// Encrypt a single 16-byte block using AES-256-ECB (for /Perms entry).
pub fn encrypt_aes256_ecb_block(key: &[u8; 32], block: &[u8; 16]) -> [u8; 16] {
    use aes::cipher::generic_array::GenericArray;

    let cipher = aes::Aes256Enc::new(key.into());
    let mut block_arr = GenericArray::clone_from_slice(block);
    cipher.encrypt_block(&mut block_arr);
    block_arr.into()
}
