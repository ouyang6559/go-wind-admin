//! 密码传输加密（对齐 Kratos 后端 `go-utils/crypto`）。
//!
//! 前端对密码做 AES-128-CBC（base64）加密后再传输，落库哈希基于**解密后的明文**。
//! key 与 IV 均为 `f51d66a73d8a0927`（16 字节），PKCS#5/PKCS#7 填充。

use aes::Aes128;
use base64::Engine;
use cipher::block_padding::Pkcs7;
use cipher::{BlockDecryptMut, KeyIvInit};

use crate::error::AppError;

type Aes128Cbc = cbc::Decryptor<Aes128>;

/// 传输层 AES 密钥（与 Kratos `crypto.DefaultAESKey` 一致，16 字节）
pub const TRANSPORT_AES_KEY: &[u8; 16] = b"f51d66a73d8a0927";

/// 解密 base64 编码的前端传输秘密（AES-128-CBC，IV=Key，PKCS7）。
/// 失败统一返回 `AppError::Validation`，调用方不得吞掉。
pub fn decrypt_transport_secret(secret: &str) -> Result<String, AppError> {
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(secret)
        .map_err(|e| AppError::Validation(format!("invalid base64 credential: {e}")))?;

    let mut buf = ciphertext.clone();
    // `decrypt_padded_mut` 返回去除填充后的明文切片（含可能的尾部数据，padding 已剥离）
    let plain = Aes128Cbc::new(TRANSPORT_AES_KEY.into(), TRANSPORT_AES_KEY.into())
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| AppError::Validation(format!("decrypt credential failed: {e}")))?;

    Ok(String::from_utf8_lossy(plain).into_owned())
}
// ===================== 渠道密码 AES-256-GCM（对齐 Go pkg/crypto/encryptor.go） =====================

const ENCRYPTED_PREFIX: &str = "enc:";

/// 解密 `enc:` 前缀的 AES-256-GCM 密文（key 为 GOWIND_CRYPTO_KEY 的 SHA-256）。
/// 无 `enc:` 前缀视为明文直存（全局加密未启用），原样返回。
pub fn decrypt_channel_secret(stored: &str) -> Result<String, AppError> {
    let Some(b64) = stored.strip_prefix(ENCRYPTED_PREFIX) else {
        return Ok(stored.to_string());
    };
    let Some(key) = channel_key() else {
        return Err(AppError::Internal {
            context: "GOWIND_CRYPTO_KEY not set but payload is encrypted".into(),
            source: None,
        });
    };
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::Aes256Gcm;
    let data = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| AppError::Internal {
            context: "decode channel secret failed".into(),
            source: Some(Box::new(e)),
        })?;
    let cipher = Aes256Gcm::new((&key).into());
    // Go: ciphertext = nonce || seal(nonce, nonce, plaintext)；nonce 12 字节在前
    let nonce = data
        .get(..12)
        .ok_or_else(|| AppError::Internal {
            context: "channel secret too short".into(),
            source: None,
        })?;
    let payload = &data[12..];
    let plain = cipher
        .decrypt(nonce.into(), payload)
        .map_err(|e| AppError::Internal {
            context: format!("decrypt channel secret failed: {e}"),
            source: None,
        })?;
    String::from_utf8(plain).map_err(|e| AppError::Internal {
        context: "channel secret not utf8".into(),
        source: Some(Box::new(e)),
    })
}

/// 加密为 `enc:` 前缀格式（未配置 GOWIND_CRYPTO_KEY 时明文直存，对齐 EncryptIfNeeded）。
pub fn encrypt_channel_secret(plaintext: &str) -> Result<String, AppError> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let Some(key) = channel_key() else {
        return Ok(plaintext.to_string());
    };
    use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
    use aes_gcm::Aes256Gcm;
    let cipher = Aes256Gcm::new((&key).into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let sealed = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Internal {
            context: format!("encrypt channel secret failed: {e}"),
            source: None,
        })?;
    let mut data = nonce.to_vec();
    data.extend_from_slice(&sealed);
    Ok(format!(
        "{ENCRYPTED_PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(data)
    ))
}

fn channel_key() -> Option<[u8; 32]> {
    use sha2::{Digest, Sha256};
    let raw = std::env::var("GOWIND_CRYPTO_KEY").ok()?;
    if raw.is_empty() {
        return None;
    }
    let hash = Sha256::digest(raw.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    Some(key)
}

// ===================== HMAC-SHA256 签名（对齐 Go pkg/crypto/hmac.go SignData） =====================
// 用途：签名媒体访问 URL（/admin/v1/file/image?path=...&expires=...&sig=...）。
// 签名即凭证：key = SHA-256(GOWIND_CRYPTO_KEY)，与渠道密码派生同一把密钥；未配置时返回 None（签名不可用）。

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// 计算 `data` 的 HMAC-SHA256 十六进制签名。`GOWIND_CRYPTO_KEY` 未配置时返回 None。
pub fn sign_data(data: &str) -> Option<String> {
    let key = channel_key()?;
    let mut mac = HmacSha256::new_from_slice(&key).ok()?;
    mac.update(data.as_bytes());
    Some(hex::encode(mac.finalize().into_bytes()))
}

/// 校验 `signature` 是否与 `data` 的签名一致（恒定时间比较）。
pub fn verify_signature(data: &str, signature: &str) -> bool {
    let Some(expect) = sign_data(data) else {
        return false;
    };
    let expect_bytes = hex::decode(expect).unwrap_or_default();
    let sig_bytes = hex::decode(signature.trim()).unwrap_or_default();
    constant_time_eq(&expect_bytes, &sig_bytes)
}

/// 恒定时间比较（长度不一致直接 false）
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use std::sync::atomic::{AtomicU8, Ordering};
    if a.len() != b.len() {
        return false;
    }
    let mut diff = AtomicU8::new(0);
    for (x, y) in a.iter().zip(b.iter()) {
        diff.fetch_or(x ^ y, Ordering::SeqCst);
    }
    diff.load(Ordering::SeqCst) == 0
}
