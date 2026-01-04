use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use blake2::{
    Blake2sMac,
    digest::{Mac, consts::U16},
};

pub fn sign(key: &[u8], data: &[u8]) -> String {
    let mut mac =
        Blake2sMac::<U16>::new_with_salt_and_personal(key, &[], &[]).expect("invalid length");

    mac.update(data);

    let result = mac.finalize();
    URL_SAFE.encode(result.into_bytes())
}
