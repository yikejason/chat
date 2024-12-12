use std::path::{Path, PathBuf};

use super::ChatFile;
use sha1::{Digest, Sha1};

impl ChatFile {
    pub fn new(filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ext: filename.split('.').last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self, ws_id: u64) -> String {
        format!("/files/{ws_id}/{}", self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    // split hash into 3 parts, first 2 with 3 chars
    pub fn hash_to_path(&self) -> String {
        let (parts1, parts2) = self.hash.split_at(3);
        let (parts2, parts3) = parts2.split_at(3);
        format!("{}/{}/{}.{}", parts1, parts2, parts3, self.ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_file_new_should_work() {
        let file = ChatFile::new("hello.txt", b"txt");
        assert_eq!(file.ext, "txt");
        assert_eq!(file.hash, "3a9f3478bc9a9ec348ea30534618d4592ad5a519");
    }
}
