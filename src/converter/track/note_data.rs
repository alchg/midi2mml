pub struct NoteData {
    pub key: u8,
    pub tick: u32,
    pub sub_channel: u8,
}

impl NoteData {
    pub fn new(key: u8, tick: u32, sub_channel: u8) -> Self {
        NoteData {
            key,
            tick,
            sub_channel,
        }
    }
}
