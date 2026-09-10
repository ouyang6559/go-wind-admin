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