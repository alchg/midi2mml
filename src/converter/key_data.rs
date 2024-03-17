#[derive(Clone, Copy, PartialEq, Debug)]
pub struct KeyData {
    pub key: u8,
    pub vol: u8,
}

impl KeyData {
    pub fn new(key: u8, vol: u8) -> Self {
        KeyData { key, vol }
    }
}
