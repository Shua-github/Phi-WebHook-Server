use aes::Aes256;
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};

#[allow(dead_code)]
type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

const AES_KEY: &[u8; 32] = &[
    0xe8, 0x96, 0x9a, 0xd2, 0xa5, 0x40, 0x25, 0x9b, 0x97, 0x91, 0x90, 0x8b, 0x88, 0xe6, 0xbf, 0x03,
    0x1e, 0x6d, 0x21, 0x95, 0x6e, 0xfa, 0xd6, 0x8a, 0x50, 0xdd, 0x55, 0xd6, 0x7a, 0xb0, 0x92, 0x4b,
];

const AES_IV: &[u8; 16] = &[
    0x2a, 0x4f, 0xf0, 0x8a, 0xc8, 0x0d, 0x63, 0x07, 0x00, 0x57, 0xc5, 0x95, 0x18, 0xc8, 0x32, 0x53,
];

#[inline]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff: u8 = 0;

    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }

    diff == 0
}

#[allow(dead_code)]
pub fn encrypt(data: &[u8]) -> Vec<u8> {
    let mut buf = data.to_vec();
    let pad_len = 16 - (buf.len() % 16);
    buf.extend(std::iter::repeat(0u8).take(pad_len));

    let ct = Aes256CbcEnc::new(AES_KEY.into(), AES_IV.into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, data.len())
        .unwrap();

    ct.to_vec()
}

pub fn decrypt(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut buf = data.to_vec();

    let pt = Aes256CbcDec::new(AES_KEY.into(), AES_IV.into())
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| format!("解密失败: {:?}", e))?;

    Ok(pt.to_vec())
}
