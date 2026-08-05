//! PDF encryption: RC4-128 (V=2, R=3), AES-128 (V=4, R=4), and AES-256 (V=5, R=6).

mod aes128;
mod aes256;
mod key;
mod rc4;

use lopdf::{dictionary, Object, ObjectId};

use crate::document::Document;
use crate::error::{PdfError, Result};

pub use paperjam_model::encryption::*;

/// Encrypt a PDF document, returning the encrypted bytes.
///
/// Supports RC4-128 (V=2, R=3), AES-128 (V=4, R=4), and AES-256 (V=5, R=6).
pub fn encrypt(doc: &Document, options: &EncryptionOptions) -> Result<Vec<u8>> {
    let mut lopdf_doc = doc.inner().clone();

    let file_id = ensure_file_id(&mut lopdf_doc)?;
    let permissions = options.permissions.to_i32();

    match options.algorithm {
        EncryptionAlgorithm::Rc4 | EncryptionAlgorithm::Aes128 => {
            // PDF 1.7: MD5-based key derivation
            let key_len = 16;

            let o_hash = key::compute_owner_hash(
                options.owner_password.as_bytes(),
                options.user_password.as_bytes(),
                key_len,
            );

            let enc_key = key::compute_encryption_key(
                options.user_password.as_bytes(),
                &o_hash,
                permissions,
                &file_id,
                key_len,
            );

            let u_hash = key::compute_user_hash(&enc_key, &file_id);

            let encrypt_dict = match options.algorithm {
                EncryptionAlgorithm::Rc4 => build_rc4_encrypt_dict(&o_hash, &u_hash, permissions),
                EncryptionAlgorithm::Aes128 => {
                    build_aes128_encrypt_dict(&o_hash, &u_hash, permissions)
                }
                _ => unreachable!(),
            };

            let encrypt_id = lopdf_doc.add_object(Object::Dictionary(encrypt_dict));
            lopdf_doc
                .trailer
                .set("Encrypt", Object::Reference(encrypt_id));

            match options.algorithm {
                EncryptionAlgorithm::Rc4 => {
                    encrypt_objects_rc4(&mut lopdf_doc, &enc_key, encrypt_id)?
                }
                EncryptionAlgorithm::Aes128 => {
                    encrypt_objects_aes128(&mut lopdf_doc, &enc_key, encrypt_id)?
                }
                _ => unreachable!(),
            }
        }
        EncryptionAlgorithm::Aes256 => {
            // PDF 2.0: SHA-based key derivation, random file encryption key
            let file_key = key::generate_file_encryption_key();

            let (u_value, ue_value) =
                key::compute_u_value_r6(options.user_password.as_bytes(), &file_key);
            let (o_value, oe_value) =
                key::compute_o_value_r6(options.owner_password.as_bytes(), &file_key, &u_value);
            let perms_value = key::compute_perms_value_r6(permissions, &file_key, true);

            let encrypt_dict = build_aes256_encrypt_dict(
                &u_value,
                &ue_value,
                &o_value,
                &oe_value,
                &perms_value,
                permissions,
            );

            let encrypt_id = lopdf_doc.add_object(Object::Dictionary(encrypt_dict));
            lopdf_doc
                .trailer
                .set("Encrypt", Object::Reference(encrypt_id));

            encrypt_objects_aes256(&mut lopdf_doc, &file_key, encrypt_id)?;
        }
    }

    let mut buf = Vec::new();
    lopdf_doc
        .save_to(&mut buf)
        .map_err(|e| PdfError::Encryption(format!("Failed to serialize encrypted PDF: {}", e)))?;

    Ok(buf)
}

/// Build the /Encrypt dictionary for RC4-128 (V=2, R=3).
fn build_rc4_encrypt_dict(
    o_hash: &[u8; 32],
    u_hash: &[u8; 32],
    permissions: i32,
) -> lopdf::Dictionary {
    dictionary! {
        "Filter" => Object::Name(b"Standard".to_vec()),
        "V" => Object::Integer(2),
        "R" => Object::Integer(3),
        "Length" => Object::Integer(128),
        "O" => Object::String(o_hash.to_vec(), lopdf::StringFormat::Hexadecimal),
        "U" => Object::String(u_hash.to_vec(), lopdf::StringFormat::Hexadecimal),
        "P" => Object::Integer(permissions as i64)
    }
}

/// Build the /Encrypt dictionary for AES-128 (V=4, R=4) with crypt filters.
fn build_aes128_encrypt_dict(
    o_hash: &[u8; 32],
    u_hash: &[u8; 32],
    permissions: i32,
) -> lopdf::Dictionary {
    // Build the crypt filter dictionary
    let std_cf = dictionary! {
        "CFM" => Object::Name(b"AESV2".to_vec()),
        "AuthEvent" => Object::Name(b"DocOpen".to_vec()),
        "Length" => Object::Integer(16)
    };

    let cf = dictionary! {
        "StdCF" => Object::Dictionary(std_cf)
    };

    dictionary! {
        "Filter" => Object::Name(b"Standard".to_vec()),
        "V" => Object::Integer(4),
        "R" => Object::Integer(4),
        "Length" => Object::Integer(128),
        "O" => Object::String(o_hash.to_vec(), lopdf::StringFormat::Hexadecimal),
        "U" => Object::String(u_hash.to_vec(), lopdf::StringFormat::Hexadecimal),
        "P" => Object::Integer(permissions as i64),
        "StmF" => Object::Name(b"StdCF".to_vec()),
        "StrF" => Object::Name(b"StdCF".to_vec()),
        "CF" => Object::Dictionary(cf)
    }
}

/// Get or generate the /ID entry in the document trailer.
///
/// If no /ID exists, generates a deterministic 16-byte identifier.
fn ensure_file_id(doc: &mut lopdf::Document) -> Result<Vec<u8>> {
    if let Ok(Object::Array(ids)) = doc.trailer.get(b"ID") {
        if let Some(Object::String(id, _)) = ids.first() {
            return Ok(id.clone());
        }
    }

    let id: Vec<u8> = (0..16)
        .map(|i| {
            let seed = (doc.max_id as u64)
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407)
                .wrapping_add(i as u64);
            (seed >> 33) as u8
        })
        .collect();

    let id_obj = Object::String(id.clone(), lopdf::StringFormat::Hexadecimal);
    doc.trailer
        .set("ID", Object::Array(vec![id_obj.clone(), id_obj]));

    Ok(id)
}

/// Encrypt all objects in the document using RC4.
fn encrypt_objects_rc4(doc: &mut lopdf::Document, key: &[u8], encrypt_id: ObjectId) -> Result<()> {
    let obj_ids: Vec<ObjectId> = doc.objects.keys().copied().collect();

    for obj_id in obj_ids {
        if obj_id == encrypt_id {
            continue;
        }
        if let Some(obj) = doc.objects.remove(&obj_id) {
            let encrypted = encrypt_object_recursive_rc4(key, obj_id, obj);
            doc.objects.insert(obj_id, encrypted);
        }
    }

    Ok(())
}

/// Recursively encrypt a single PDF object (strings and streams) with RC4.
fn encrypt_object_recursive_rc4(key: &[u8], obj_id: ObjectId, obj: Object) -> Object {
    match obj {
        Object::String(data, _format) => {
            let encrypted = key::encrypt_object(key, obj_id.0, obj_id.1, &data);
            Object::String(encrypted, lopdf::StringFormat::Hexadecimal)
        }
        Object::Stream(mut stream) => {
            let encrypted = key::encrypt_object(key, obj_id.0, obj_id.1, &stream.content);
            stream.content = encrypted;

            let dict_entries: Vec<(Vec<u8>, Object)> = stream
                .dict
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (k, v) in dict_entries {
                let encrypted_val = encrypt_object_recursive_rc4(key, obj_id, v);
                stream.dict.set(k, encrypted_val);
            }

            Object::Stream(stream)
        }
        Object::Dictionary(dict) => {
            let mut new_dict = lopdf::Dictionary::new();
            for (k, v) in dict.iter() {
                new_dict.set(
                    k.clone(),
                    encrypt_object_recursive_rc4(key, obj_id, v.clone()),
                );
            }
            Object::Dictionary(new_dict)
        }
        Object::Array(arr) => {
            let new_arr: Vec<Object> = arr
                .into_iter()
                .map(|item| encrypt_object_recursive_rc4(key, obj_id, item))
                .collect();
            Object::Array(new_arr)
        }
        other => other,
    }
}

/// Encrypt all objects in the document using AES-128-CBC.
fn encrypt_objects_aes128(
    doc: &mut lopdf::Document,
    key: &[u8],
    encrypt_id: ObjectId,
) -> Result<()> {
    let obj_ids: Vec<ObjectId> = doc.objects.keys().copied().collect();

    for obj_id in obj_ids {
        if obj_id == encrypt_id {
            continue;
        }
        if let Some(obj) = doc.objects.remove(&obj_id) {
            let encrypted = encrypt_object_recursive_aes128(key, obj_id, obj);
            doc.objects.insert(obj_id, encrypted);
        }
    }

    Ok(())
}

/// Recursively encrypt a single PDF object (strings and streams) with AES-128-CBC.
fn encrypt_object_recursive_aes128(key: &[u8], obj_id: ObjectId, obj: Object) -> Object {
    match obj {
        Object::String(data, _format) => {
            let encrypted = key::encrypt_object_aes128(key, obj_id.0, obj_id.1, &data);
            Object::String(encrypted, lopdf::StringFormat::Hexadecimal)
        }
        Object::Stream(mut stream) => {
            let encrypted = key::encrypt_object_aes128(key, obj_id.0, obj_id.1, &stream.content);
            stream.content = encrypted;

            let dict_entries: Vec<(Vec<u8>, Object)> = stream
                .dict
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (k, v) in dict_entries {
                let encrypted_val = encrypt_object_recursive_aes128(key, obj_id, v);
                stream.dict.set(k, encrypted_val);
            }

            Object::Stream(stream)
        }
        Object::Dictionary(dict) => {
            let mut new_dict = lopdf::Dictionary::new();
            for (k, v) in dict.iter() {
                new_dict.set(
                    k.clone(),
                    encrypt_object_recursive_aes128(key, obj_id, v.clone()),
                );
            }
            Object::Dictionary(new_dict)
        }
        Object::Array(arr) => {
            let new_arr: Vec<Object> = arr
                .into_iter()
                .map(|item| encrypt_object_recursive_aes128(key, obj_id, item))
                .collect();
            Object::Array(new_arr)
        }
        other => other,
    }
}

/// Build the /Encrypt dictionary for AES-256 (V=5, R=6) with AESV3 crypt filters.
fn build_aes256_encrypt_dict(
    u_value: &[u8; 48],
    ue_value: &[u8; 32],
    o_value: &[u8; 48],
    oe_value: &[u8; 32],
    perms_value: &[u8; 16],
    permissions: i32,
) -> lopdf::Dictionary {
    let std_cf = dictionary! {
        "CFM" => Object::Name(b"AESV3".to_vec()),
        "AuthEvent" => Object::Name(b"DocOpen".to_vec()),
        "Length" => Object::Integer(32)
    };

    let cf = dictionary! {
        "StdCF" => Object::Dictionary(std_cf)
    };

    dictionary! {
        "Filter" => Object::Name(b"Standard".to_vec()),
        "V" => Object::Integer(5),
        "R" => Object::Integer(6),
        "Length" => Object::Integer(256),
        "O" => Object::String(o_value.to_vec(), lopdf::StringFormat::Hexadecimal),
        "U" => Object::String(u_value.to_vec(), lopdf::StringFormat::Hexadecimal),
        "OE" => Object::String(oe_value.to_vec(), lopdf::StringFormat::Hexadecimal),
        "UE" => Object::String(ue_value.to_vec(), lopdf::StringFormat::Hexadecimal),
        "Perms" => Object::String(perms_value.to_vec(), lopdf::StringFormat::Hexadecimal),
        "P" => Object::Integer(permissions as i64),
        "StmF" => Object::Name(b"StdCF".to_vec()),
        "StrF" => Object::Name(b"StdCF".to_vec()),
        "CF" => Object::Dictionary(cf),
        "EncryptMetadata" => Object::Boolean(true)
    }
}

/// Encrypt all objects in the document using AES-256-CBC.
///
/// Unlike RC4/AES-128, AES-256 uses the file encryption key directly
/// (no per-object key derivation via MD5).
fn encrypt_objects_aes256(
    doc: &mut lopdf::Document,
    file_key: &[u8; 32],
    encrypt_id: ObjectId,
) -> Result<()> {
    let obj_ids: Vec<ObjectId> = doc.objects.keys().copied().collect();

    for obj_id in obj_ids {
        if obj_id == encrypt_id {
            continue;
        }
        if let Some(obj) = doc.objects.remove(&obj_id) {
            let encrypted = encrypt_object_recursive_aes256(file_key, obj);
            doc.objects.insert(obj_id, encrypted);
        }
    }

    Ok(())
}

/// Recursively encrypt a single PDF object (strings and streams) with AES-256-CBC.
///
/// Uses the file key directly with a random IV per object — no per-object key derivation.
fn encrypt_object_recursive_aes256(key: &[u8; 32], obj: Object) -> Object {
    use rand::Rng;

    match obj {
        Object::String(data, _format) => {
            let iv: [u8; 16] = rand::thread_rng().gen();
            let encrypted = aes256::encrypt_aes256_cbc(&iv, key, &data);
            Object::String(encrypted, lopdf::StringFormat::Hexadecimal)
        }
        Object::Stream(mut stream) => {
            let iv: [u8; 16] = rand::thread_rng().gen();
            let encrypted = aes256::encrypt_aes256_cbc(&iv, key, &stream.content);
            stream.content = encrypted;

            let dict_entries: Vec<(Vec<u8>, Object)> = stream
                .dict
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (k, v) in dict_entries {
                let encrypted_val = encrypt_object_recursive_aes256(key, v);
                stream.dict.set(k, encrypted_val);
            }

            Object::Stream(stream)
        }
        Object::Dictionary(dict) => {
            let mut new_dict = lopdf::Dictionary::new();
            for (k, v) in dict.iter() {
                new_dict.set(k.clone(), encrypt_object_recursive_aes256(key, v.clone()));
            }
            Object::Dictionary(new_dict)
        }
        Object::Array(arr) => {
            let new_arr: Vec<Object> = arr
                .into_iter()
                .map(|item| encrypt_object_recursive_aes256(key, item))
                .collect();
            Object::Array(new_arr)
        }
        other => other,
    }
}
