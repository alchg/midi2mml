use super::DataKind;

#[derive(Clone, Copy, PartialEq)]
pub struct Event {
    pub channel: u8,
    pub sub_channel: u8,
    pub tick: u32,
    pub data_kind: DataKind,
}

impl Event {
    pub fn new(channel: u8, tick: u32, data_kind: DataKind) -> Self {
        Event {
            channel,
            sub_channel: 0,
            tick,
            data_kind,
        }
    }

    pub fn new_with_sub(channel: u8, sub_channel: u8, tick: u32, data_kind: DataKind) -> Self {
        Event {
            channel,
            sub_channel,
            tick,
            data_kind,
        }
    }
}
