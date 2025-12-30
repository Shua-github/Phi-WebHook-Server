use crate::utils::decrypt;
use bitvec::prelude::{BitSlice, Lsb0};
use phi_save_codec::game_key::{field::GameKey, serde::SerializableGameKey};
use phi_save_codec::game_progress::{field::GameProgress, serde::SerializableGameProgress};
use phi_save_codec::game_record::{field::GameRecord, serde::SerializableGameRecord};
use phi_save_codec::settings::{field::Settings, serde::SerializableSettings};
use phi_save_codec::user::{field::User, serde::SerializableUser};
use serde::Serialize;
use shua_struct::field::BinaryField;
use std::io::Cursor;
use std::io::Read;
use zip::ZipArchive;

const SAVE_LIST: &[&str] = &["gameKey", "gameProgress", "gameRecord", "user", "settings"];

#[derive(Default)]
pub struct Zip {
    game_progress: Vec<u8>,
    game_record: Vec<u8>,
    user: Vec<u8>,
    game_key: Vec<u8>,
    settings: Vec<u8>,
}

#[derive(Serialize)]
pub struct Save {
    pub game_progress: SerializableGameProgress,
    pub game_record: SerializableGameRecord,
    pub user: SerializableUser,
    pub game_key: SerializableGameKey,
    pub settings: SerializableSettings,
}

pub fn unzip(save_data: Cursor<Vec<u8>>) -> Result<Zip, String> {
    let mut archive =
        ZipArchive::new(save_data).map_err(|e| format!("Failed to open zip: {}", e))?;

    let mut zip = Zip::default();

    for &file_name in SAVE_LIST {
        match archive.by_name(file_name) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .map_err(|_| format!("Failed to read file: {}", file_name))?;

                match file_name {
                    "gameProgress" => zip.game_progress = buf,
                    "gameRecord" => zip.game_record = buf,
                    "user" => zip.user = buf,
                    "gameKey" => zip.game_key = buf,
                    "settings" => zip.settings = buf,
                    _ => {}
                }
            }
            Err(_) => return Err(format!("Failed to read file: {}", file_name)),
        }
    }

    Ok(zip)
}

fn process_field<T, S>(mut raw_data: Vec<u8>) -> Result<S, String>
where
    T: BinaryField<Lsb0>,
    S: From<T>,
{
    if raw_data.is_empty() {
        return Err("数据为空".to_owned());
    }
    raw_data.drain(0..1);
    let decrypted = decrypt(&raw_data).map_err(|e| format!("解密失败: {}", e))?;

    let bits = BitSlice::<u8, Lsb0>::from_slice(&decrypted);
    let (item, _) = T::parse(bits, &None).map_err(|e| format!("解析失败: {}", e))?;

    Ok(S::from(item))
}

fn process_field_named<T, S>(field_name: &str, raw_data: Vec<u8>) -> Result<S, String>
where
    T: BinaryField<Lsb0>,
    S: From<T>,
{
    process_field::<T, S>(raw_data).map_err(|e| format!("字段 '{}': {}", field_name, e))
}

pub fn parse_save(zip: Zip) -> Result<Save, String> {
    Ok(Save {
        game_key: process_field_named::<GameKey, SerializableGameKey>("game_key", zip.game_key)?,
        game_progress: process_field_named::<GameProgress, SerializableGameProgress>(
            "game_progress",
            zip.game_progress,
        )?,
        game_record: process_field_named::<GameRecord, SerializableGameRecord>(
            "game_record",
            zip.game_record,
        )?,
        user: process_field_named::<User, SerializableUser>("user", zip.user)?,
        settings: process_field_named::<Settings, SerializableSettings>("settings", zip.settings)?,
    })
}
