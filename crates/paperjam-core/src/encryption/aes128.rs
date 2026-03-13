//! AES-128-CBC encryption for PDF objects.

use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

/// Encrypt data using AES-128-CBC with PKCS#7 padding.
///
/// Returns IV (16 bytes) prepended to the ciphertext.
pub fn encrypt_aes128(iv: &[u8; 16], key: &[u8], data: &[u8]) -> Vec<u8> {
    let key_arr: [u8; 16] = key[..16].try_into().expect("AES-128 key must be 16 bytes");
    let cipher = Aes128CbcEnc::new(&key_arr.into(), iv.into());

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
