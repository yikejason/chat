use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::AppError;

use super::ChatFile;
use sha1::{Digest, Sha1};

impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.split('.').last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}/{}", self.ws_id, self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    // split hash into 3 parts, first 2 with 3 chars
    pub fn hash_to_path(&self) -> String {
        let (parts1, parts2) = self.hash.split_at(3);
        let (parts2, parts3) = parts2.split_at(3);
        format!(
            "{}/{}/{}/{}.{}",
            self.ws_id, parts1, parts2, parts3, self.ext
        )
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    // covert /files/1/3fb/bc2/2d01ebcd32fda61adb8e48c09111bef621.png to ChatFile
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError(
                "Invalid chat file path".to_string(),
            ));
        };

        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError(
                "File path does not valid".to_string(),
            ));
        }

        let Ok(ws_id) = parts[0].parse::<u64>() else {
            return Err(AppError::ChatFileError(format!(
                "Invalid workspace id: {}",
                parts[0]
            )));
        };

        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::ChatFileError(format!(
                "Invalid file name {}",
                parts[3]
            )));
        };

        let hash = format!("{}{}{}", parts[1], parts[2], part3);
        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_file_new_should_work() {
        let file = ChatFile::new(1, "hello.txt", b"txt");
        assert_eq!(file.ws_id, 1);
        assert_eq!(file.ext, "txt");
        assert_eq!(file.hash, "3a9f3478bc9a9ec348ea30534618d4592ad5a519");
    }
}
