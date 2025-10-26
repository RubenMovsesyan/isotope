use std::hash::{DefaultHasher, Hash, Hasher};

const BASE64_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/";

pub trait Base64 {
    fn to_base64(self) -> String;
}

impl Base64 for u64 {
    fn to_base64(mut self) -> String {
        if self == 0 {
            return "0".to_string();
        }

        let mut result = Vec::new();

        while self > 0 {
            let remainder = (self % 64) as usize;
            result.push(BASE64_CHARS[remainder] as char);
            self /= 64;
        }

        result.reverse();
        result.into_iter().collect()
    }
}

pub trait ToHash {
    fn to_hash(self) -> String;
}

impl ToHash for &str {
    fn to_hash(self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        hash.to_base64()
    }
}

pub fn compute_work_group_count(size: u32, work_group_size: u32) -> u32 {
    (size + work_group_size - 1) / work_group_size
}
