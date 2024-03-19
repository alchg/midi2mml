use super::event::Event;
use super::key_data::KeyData;
use super::TempoEvent;

mod note_data;
use note_data::NoteData;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DataKind {
    NoteOn(KeyData),
    NoteOff(KeyData),
    ChangeTimbre(u8),
    ChangeTempo(u32),
}

pub struct Track {
    pub track_index: usize,
    pub events: Vec<Event>,
    pub channels: Vec<u8>,
    channel_data_list: Vec<ChannelData>,
}

struct ChannelData {
    channel: u8,
    sub_channel_max: u8,
}

impl ChannelData {
    fn new(channel: u8, sub_channel_max: u8) -> Self {
        ChannelData {
            channel,
            sub_channel_max,
        }
    }
}

struct MmlStatus {
    ticks_per_beat: u32,
    line_ticks: u32,
    tick: u32,
}

impl MmlStatus {
    fn new(ticks_per_beat: u32) -> Self {
        MmlStatus {
            ticks_per_beat,
            line_ticks: 0,
            tick: 0,
        }
    }

    fn add_line_ticks(&mut self, ticks: u32) {
        self.line_ticks += ticks;
        self.tick += ticks;

        while self.line_ticks >= self.ticks_per_beat * 4 {
            self.line_ticks -= self.ticks_per_beat * 4;
            print!("\n");
        }
    }
}

impl Track {
    pub fn new(track_index: usize) -> Self {
        Track {
            track_index,
            events: Vec::new(),
            channels: Vec::new(),
            channel_data_list: Vec::new(),
        }
    }

    pub fn push_event(&mut self, channel: u8, tick: u32, data_kind: DataKind) {
        self.events.push(Event::new(channel, tick, data_kind));
        if !self.channels.contains(&channel) {
            self.channels.push(channel);
        }
    }

    fn get_new_channel(notes: &Vec<NoteData>) -> Option<u8> {
        for index in 0..u8::max_value() {
            if !notes.iter().any(|note| note.sub_channel == index) {
                return Some(index);
            }
        }
        None
    }

    fn get_channel(notes: &Vec<NoteData>, key: &u8) -> Option<u8> {
        for note in notes {
            if note.key == *key {
                return Some(note.sub_channel);
            }
        }
        None
    }

    fn divsion_tick(ticks_per_beat: u32, mut tick: u32) -> u32 {
        while tick > 0 {
            if tick >= ticks_per_beat * 4 {
                // 1
                tick -= ticks_per_beat * 4
            } else if tick >= ticks_per_beat * 2 {
                // 2
                tick -= ticks_per_beat * 2
            } else if tick >= ticks_per_beat {
                // 4
                tick -= ticks_per_beat
            } else if tick % (ticks_per_beat / 3) == 0 {
                // 4-3
                tick -= ticks_per_beat / 3;
            } else if tick >= ticks_per_beat / 2 {
                // 8
                tick -= ticks_per_beat / 2;
            } else if tick % (ticks_per_beat / 6) == 0 {
                // 8-3
                tick -= ticks_per_beat / 6;
            } else if tick >= ticks_per_beat / 4 {
                // 16
                tick -= ticks_per_beat / 4
            } else if tick % (ticks_per_beat / 12) == 0 {
                // 16-3
                tick -= ticks_per_beat / 12;
            } else if tick >= ticks_per_beat / 8 {
                // 32
                tick -= ticks_per_beat / 8
            } else if tick % (ticks_per_beat / 24) == 0 {
                // 32-3
                tick -= ticks_per_beat / 24;
            } else if tick >= ticks_per_beat / 16 {
                // 64
                tick -= ticks_per_beat / 16
            } else if tick % (ticks_per_beat / 48) == 0 {
                // 64-3
                tick -= ticks_per_beat / 48;
            } else if tick >= ticks_per_beat / 32 {
                // 128
                tick -= ticks_per_beat / 32
            } else {
                return tick;
            }
        }
        tick
    }

    pub fn parse1(&mut self, tempo_events: &mut Vec<TempoEvent>) -> Result<(), String> {
        println!("Track:{} Channel:{:?}", self.track_index, self.channels);

        self.events.sort_by(|e1, e2| {
            let order = e1.tick.cmp(&e2.tick);
            if order == std::cmp::Ordering::Equal {
                match (&e1.data_kind, &e2.data_kind) {
                    (DataKind::NoteOn(key_data1), DataKind::NoteOn(key_data2)) => {
                        key_data1.key.cmp(&key_data2.key)
                    }
                    (DataKind::NoteOff(key_data1), DataKind::NoteOff(key_data2)) => {
                        key_data2.key.cmp(&key_data1.key)
                    }
                    _ => std::cmp::Ordering::Equal,
                }
            } else {
                order
            }
        });

        self.events.sort_by(|e1, e2| e1.channel.cmp(&e2.channel));

        let mut notes: Vec<NoteData> = Vec::new();
        let mut pre_channel: u8 = 0;
        let mut valid_channels: Vec<u8> = Vec::new();
        let mut change_timbre_events: Vec<Event> = Vec::new();
        let mut sub_channel_max: u8 = 0;
        let mut note_off_events_delete: Vec<Event> = Vec::new();
        let mut note_on_events_delete: Vec<Event> = Vec::new();
        for index in 0..self.events.len() {
            if self.events[index].channel != pre_channel {
                if valid_channels.contains(&pre_channel) {
                    self.channel_data_list
                        .push(ChannelData::new(pre_channel, sub_channel_max));
                }
                pre_channel = self.events[index].channel;
                notes = Vec::new();
                sub_channel_max = 0;
            }

            match self.events[index].data_kind {
                DataKind::NoteOn(key_data) => {
                    let sub_channel: u8;
                    match Track::get_channel(&notes, &key_data.key) {
                        Some(_) => {
                            println!(
                                "Warning! Tick:{} Channel:{} key:{} Continuous NoteOn is not supported.",
                                self.events[index].tick,self.events[index].channel,key_data.key
                            );
                            note_on_events_delete.push(self.events[index])
                        }
                        None => {
                            match Track::get_new_channel(&notes) {
                                Some(new_channel) => sub_channel = new_channel,
                                None => return Err(format!("Failed to get subchannel.")),
                            }

                            if sub_channel > sub_channel_max {
                                sub_channel_max = sub_channel;
                            }

                            self.events[index].sub_channel = sub_channel;

                            notes.push(NoteData::new(
                                key_data.key,
                                self.events[index].tick,
                                sub_channel,
                            ));

                            if !valid_channels.contains(&(self.events[index].channel)) {
                                valid_channels.push(self.events[index].channel);
                            }
                        }
                    }
                    /*
                    println!(
                        "Tick:{} Channel:{} Sub:{} Kind:{:?} Key:{}",
                        self.events[index].tick,
                        self.events[index].channel,
                        self.events[index].sub_channel,
                        self.events[index].data_kind,
                        key_data.key
                    )
                    */
                }
                DataKind::NoteOff(key_data) => {
                    match Track::get_channel(&notes, &(key_data.key)) {
                        Some(channel) => self.events[index].sub_channel = channel,
                        None => {
                            /*
                            return Err(format!(
                                "The key {} for which NoteOff should be performed was not found.",
                                key_data.key
                            ))
                            */
                            println!(
                                "Warning! Tick:{} Channel:{} Key:{} No Key to NoteOff was found.",
                                self.events[index].tick, self.events[index].channel, key_data.key
                            );
                            note_off_events_delete.push(self.events[index]);
                        }
                    }
                    notes.retain(|e1| e1.key != key_data.key);
                    /*
                    println!(
                        "Tick:{} Channel:{} Sub:{} Kind:{:?} Key:{}",
                        self.events[index].tick,
                        self.events[index].channel,
                        self.events[index].sub_channel,
                        self.events[index].data_kind,
                        key_data.key
                    )
                    */
                }
                DataKind::ChangeTimbre(_) => {
                    change_timbre_events.push(self.events[index]);
                    /*
                    println!(
                        "Tick:{} Channel:{} Sub:{} Kind:{:?}",
                        self.events[index].tick,
                        self.events[index].channel,
                        self.events[index].sub_channel,
                        self.events[index].data_kind,
                    )
                    */
                }
                DataKind::ChangeTempo(_) => (),
            }
        }
        self.events
            .retain(|event| !note_off_events_delete.contains(event));
        self.events
            .retain(|event| !note_on_events_delete.contains(event));
        if valid_channels.contains(&pre_channel) {
            self.channel_data_list
                .push(ChannelData::new(pre_channel, sub_channel_max));
        }
        for channel_data in self.channel_data_list.iter() {
            println!(
                "Channel:{} SubChannels:{}",
                channel_data.channel, channel_data.sub_channel_max
            );
        }

        for change_timbre_event in change_timbre_events.iter() {
            for channel_data in self.channel_data_list.iter() {
                if channel_data.channel == change_timbre_event.channel {
                    if channel_data.sub_channel_max > 0 {
                        for index in 1..channel_data.sub_channel_max + 1 {
                            self.events.insert(
                                0,
                                Event::new_with_sub(
                                    change_timbre_event.channel,
                                    index,
                                    change_timbre_event.tick,
                                    change_timbre_event.data_kind,
                                ),
                            );
                        }
                    }
                }
            }
        }

        for channel_data in self.channel_data_list.iter() {
            for tempo_event in tempo_events.iter() {
                if tempo_event.enable == true {
                    for index in 0..channel_data.sub_channel_max + 1 {
                        self.events.insert(
                            0,
                            Event::new_with_sub(
                                channel_data.channel,
                                index,
                                tempo_event.tick,
                                DataKind::ChangeTempo(tempo_event.tempo),
                            ),
                        );
                    }
                }
            }
        }

        self.events.sort_by(|e1, e2| e1.tick.cmp(&e2.tick));
        self.events
            .sort_by(|e1, e2| e1.sub_channel.cmp(&e2.sub_channel));
        self.events.sort_by(|e1, e2| e1.channel.cmp(&e2.channel));

        self.events
            .retain(|e1| valid_channels.contains(&e1.channel));

        let mut note_on = false;
        let mut timbre_events_delete: Vec<Event> = Vec::new();
        for event in self.events.iter() {
            match event.data_kind {
                DataKind::NoteOn(_) => note_on = true,
                DataKind::NoteOff(_) => note_on = false,
                DataKind::ChangeTempo(_) => {
                    if note_on == true {
                        for tempo_event in tempo_events.iter_mut() {
                            if tempo_event.tick == event.tick {
                                tempo_event.enable = false;
                            }
                        }
                        /*
                        return Err(format!(
                            "Tempo changes in the middle of a sound are not supported."
                        ));
                        */
                        println!(
                            "Warning! Tick:{} Channel:{} Sub:{} Kind:{:?} Tempo changes in the middle of a sound are not supported."
                            ,event.tick,event.channel,event.sub_channel,event.data_kind
                        );
                    }
                }
                DataKind::ChangeTimbre(_) => {
                    if note_on == true {
                        timbre_events_delete.push(*event);
                        println!(
                            "Warning! Tick:{} Channel:{} Sub:{} Kind:{:?} Timbre changes in the middle of a sound are not supported."
                            ,event.tick,event.channel,event.sub_channel,event.data_kind
                        )
                    }
                }
            }
        }
        self.events.retain(|event| match event.data_kind {
            DataKind::ChangeTempo(_) => false,
            _ => true,
        });
        self.events
            .retain(|event| !timbre_events_delete.contains(event));

        if self.events.len() >= 2 {
            for index in 0..self.events.len() - 1 {
                if let DataKind::NoteOn(key_data_on) = self.events[index].data_kind {
                    let mut result = false;
                    if let DataKind::NoteOff(key_data_off) = self.events[index + 1].data_kind {
                        if key_data_on.key == key_data_off.key
                            && self.events[index].channel == self.events[index + 1].channel
                            && self.events[index].sub_channel == self.events[index + 1].sub_channel
                            && self.events[index].tick < self.events[index + 1].tick
                        {
                            result = true;
                        }
                    }
                    if result == false {
                        println!(
                            "Tick:{} Channel:{} Sub:{} Kind:{:?}",
                            self.events[index].tick,
                            self.events[index].channel,
                            self.events[index].sub_channel,
                            self.events[index].data_kind,
                        );
                        println!(
                            "Tick:{} Channel:{} Sub:{} Kind:{:?}",
                            self.events[index + 1].tick,
                            self.events[index + 1].channel,
                            self.events[index + 1].sub_channel,
                            self.events[index + 1].data_kind,
                        );
                        return Err(format!("Sound integrity failed."));
                    }
                }
            }
        }

        println!("Valid Channel:{:?}", valid_channels);

        Ok(())
    }

    pub fn parse2(
        &mut self,
        ticks_per_beat: u32,
        tempo_events: &mut Vec<TempoEvent>,
    ) -> Result<(), String> {
        for event in tempo_events.iter() {
            if event.enable {
                for i in 0..self.channel_data_list.len() {
                    for j in 0..self.channel_data_list[i].sub_channel_max + 1 {
                        self.events.insert(
                            0,
                            Event::new_with_sub(
                                self.channel_data_list[i].channel,
                                j,
                                event.tick,
                                DataKind::ChangeTempo(event.tempo),
                            ),
                        );
                    }
                }
            }
        }
        self.events.sort_by(|e1, e2| e1.tick.cmp(&e2.tick));
        self.events
            .sort_by(|e1, e2| e1.sub_channel.cmp(&e2.sub_channel));
        self.events.sort_by(|e1, e2| e1.channel.cmp(&e2.channel));

        let mut channel = self.events[0].channel;
        let mut sub_channel = self.events[0].sub_channel;
        let mut pre_tick: u32 = 0;
        for event in self.events.iter_mut() {
            /*
            println!(
                "Tick:{} Channel:{} Sub:{} Kind:{:?}",
                event.tick, event.channel, event.sub_channel, event.data_kind
            );
            */
            if event.channel != channel || event.sub_channel != sub_channel {
                channel = event.channel;
                sub_channel = event.sub_channel;
                pre_tick = 0;
            }

            match event.data_kind {
                DataKind::NoteOn(_) => {
                    let remainder = Self::divsion_tick(ticks_per_beat, event.tick - pre_tick);
                    if remainder != 0 {
                        println!(
                            "Warning! Tick:{} Channel:{} Sub:{} Kind:{:?} Corrects NoteOn timing.",
                            event.tick, event.channel, event.sub_channel, event.data_kind
                        );
                        event.tick -= remainder;
                    }
                    pre_tick = event.tick;
                }
                DataKind::NoteOff(_) => {
                    let remainder = Self::divsion_tick(ticks_per_beat, event.tick - pre_tick);
                    if remainder != 0 {
                        println!(
                            "Warning! Tick:{} Channel:{} Sub:{} Kind:{:?} Corrects NoteOff timing.",
                            event.tick, event.channel, event.sub_channel, event.data_kind
                        );
                        event.tick -= remainder;
                    }
                    pre_tick = event.tick;
                }
                DataKind::ChangeTimbre(_) => {}
                DataKind::ChangeTempo(_) => {
                    let remainder = Self::divsion_tick(ticks_per_beat, event.tick - pre_tick);
                    if remainder != 0 {
                        for tempo_event in tempo_events.iter_mut() {
                            if tempo_event.tick == event.tick {
                                tempo_event.enable = false;
                            }
                        }
                        println!(
                            "Warning! Tick:{} Channel:{} Sub:{} Kind:{:?} Unsupport Change Tempo timing.",
                            event.tick, event.channel, event.sub_channel, event.data_kind
                        );
                    } else {
                        pre_tick = event.tick;
                    }
                }
            }
        }

        self.events.retain(|event| {
            if let DataKind::ChangeTempo(_) = event.data_kind {
                false
            } else {
                true
            }
        });

        for index in 0..self.events.len() - 1 {
            if self.events[index].channel == self.events[index + 1].channel
                && self.events[index].sub_channel == self.events[index + 1].sub_channel
            {
                if self.events[index].tick > self.events[index + 1].tick {
                    return Err(format!("Sound correction failed."));
                }
            }
        }

        Ok(())
    }

    pub fn parse3(&mut self, tempo_events: &mut Vec<TempoEvent>) -> Result<(), String> {
        for event in tempo_events.iter() {
            if event.enable {
                for i in 0..self.channel_data_list.len() {
                    for j in 0..self.channel_data_list[i].sub_channel_max + 1 {
                        self.events.insert(
                            0,
                            Event::new_with_sub(
                                self.channel_data_list[i].channel,
                                j,
                                event.tick,
                                DataKind::ChangeTempo(event.tempo),
                            ),
                        );
                    }
                }
            }
        }

        self.events.sort_by(|e1, e2| e1.tick.cmp(&e2.tick));
        self.events
            .sort_by(|e1, e2| e1.sub_channel.cmp(&e2.sub_channel));
        self.events.sort_by(|e1, e2| e1.channel.cmp(&e2.channel));

        for index in 0..self.events.len() - 1 {
            if self.events[index].channel == self.events[index + 1].channel
                && self.events[index].sub_channel == self.events[index + 1].sub_channel
            {
                if self.events[index].tick > self.events[index + 1].tick {
                    return Err(format!("Music integrity failed."));
                }
            }
        }
        Ok(())
    }

    fn calc_rest(mut ticks: u32, mml_status: &mut MmlStatus) {
        let ticks_per_beat = mml_status.ticks_per_beat;
        while ticks > 0 {
            if ticks >= ticks_per_beat * 4 {
                print!("r1");
                ticks -= ticks_per_beat * 4;
                mml_status.add_line_ticks(ticks_per_beat * 4);
            } else if ticks >= ticks_per_beat * 2 {
                print!("r2");
                ticks -= ticks_per_beat * 2;
                mml_status.add_line_ticks(ticks_per_beat * 2);
            } else if ticks >= ticks_per_beat {
                print!("r4");
                ticks -= ticks_per_beat;
                mml_status.add_line_ticks(ticks_per_beat);
            } else if ticks % (ticks_per_beat / 3) == 0 {
                print!("r12");
                ticks -= ticks_per_beat / 3;
                mml_status.add_line_ticks(ticks_per_beat / 3);
            } else if ticks >= ticks_per_beat / 2 {
                print!("r8");
                ticks -= ticks_per_beat / 2;
                mml_status.add_line_ticks(ticks_per_beat / 2);
            } else if ticks % (ticks_per_beat / 6) == 0 {
                print!("r24");
                ticks -= ticks_per_beat / 6;
                mml_status.add_line_ticks(ticks_per_beat / 6);
            } else if ticks >= ticks_per_beat / 4 {
                print!("r16");
                ticks -= ticks_per_beat / 4;
                mml_status.add_line_ticks(ticks_per_beat / 4);
            } else if ticks % (ticks_per_beat / 12) == 0 {
                print!("r48");
                ticks -= ticks_per_beat / 12;
                mml_status.add_line_ticks(ticks_per_beat / 12);
            } else if ticks >= ticks_per_beat / 8 {
                print!("r32");
                ticks -= ticks_per_beat / 8;
                mml_status.add_line_ticks(ticks_per_beat / 8);
            } else if ticks % (ticks_per_beat / 24) == 0 {
                print!("r96");
                ticks -= ticks_per_beat / 24;
                mml_status.add_line_ticks(ticks_per_beat / 24);
            } else if ticks >= ticks_per_beat / 16 {
                print!("r64");
                ticks -= ticks_per_beat / 16;
                mml_status.add_line_ticks(ticks_per_beat / 16);
            } else if ticks % (ticks_per_beat / 48) == 0 {
                print!("r192");
                ticks -= ticks_per_beat / 48;
                mml_status.add_line_ticks(ticks_per_beat / 24);
            } else if ticks >= ticks_per_beat / 32 {
                print!("r128");
                ticks -= ticks_per_beat / 32;
                mml_status.add_line_ticks(ticks_per_beat / 32);
            }
        }
    }

    fn calc_note(mut ticks: u32, note: String, mml_status: &mut MmlStatus) {
        let ticks_per_beat = mml_status.ticks_per_beat;
        let line_ticks = ticks;

        while ticks > 0 {
            print!("{}", note);
            if ticks >= ticks_per_beat * 4 {
                print!("1");
                ticks -= ticks_per_beat * 4;
            } else if ticks >= ticks_per_beat * 2 {
                print!("2");
                ticks -= ticks_per_beat * 2;
            } else if ticks >= ticks_per_beat {
                print!("4");
                ticks -= ticks_per_beat;
            } else if ticks % (ticks_per_beat / 3) == 0 {
                print!("12");
                ticks -= ticks_per_beat / 3;
            } else if ticks >= ticks_per_beat / 2 {
                print!("8");
                ticks -= ticks_per_beat / 2;
            } else if ticks % (ticks_per_beat / 6) == 0 {
                print!("24");
                ticks -= ticks_per_beat / 6;
            } else if ticks >= ticks_per_beat / 4 {
                print!("16");
                ticks -= ticks_per_beat / 4;
            } else if ticks % (ticks_per_beat / 12) == 0 {
                print!("48");
                ticks -= ticks_per_beat / 12;
            } else if ticks >= ticks_per_beat / 8 {
                print!("32");
                ticks -= ticks_per_beat / 8;
            } else if ticks % (ticks_per_beat / 24) == 0 {
                print!("96");
                ticks -= ticks_per_beat / 24;
            } else if ticks >= ticks_per_beat / 16 {
                print!("64");
                ticks -= ticks_per_beat / 16;
            } else if ticks % (ticks_per_beat / 48) == 0 {
                print!("192");
                ticks -= ticks_per_beat / 48;
            } else if ticks >= ticks_per_beat / 32 {
                print!("128");
                ticks -= ticks_per_beat / 32;
            }

            if ticks > 0 {
                print!("&");
            }
        }
        mml_status.add_line_ticks(line_ticks);
    }

    fn get_note(key: u8) -> String {
        let note_names = [
            "c", "c+", "d", "d+", "e", "f", "f+", "g", "g+", "a", "a+", "b",
        ];
        let index = (key % 12) as usize;
        return note_names[index].to_string();
    }

    pub fn convert(&self, ticks_per_beat: u32) {
        let mut mml_status = MmlStatus::new(ticks_per_beat);
        let mut channel = self.events[0].channel;
        let mut sub_channel = self.events[0].sub_channel;
        let mut pre_tick = 0;
        let mut volume = 75;
        let mut octave = 4;
        println!(
            ";########## Track:{} Channel:{} Sub:{} ##########",
            self.track_index, channel, sub_channel
        );
        for event in self.events.iter() {
            if event.channel != channel || event.sub_channel != sub_channel {
                print!("\n");
                mml_status = MmlStatus::new(ticks_per_beat);
                channel = event.channel;
                sub_channel = event.sub_channel;
                println!(
                    ";########## Track:{} Channel:{} Sub:{} ##########",
                    self.track_index, channel, sub_channel
                );
                pre_tick = 0;
                volume = 75;
                octave = 4;
            }
            match event.data_kind {
                DataKind::NoteOn(key_data) => {
                    Self::calc_rest(event.tick - pre_tick, &mut mml_status);
                    if volume != key_data.vol {
                        volume = key_data.vol;
                        print!("v{}", volume);
                    }
                    if octave != key_data.key / 12 {
                        octave = key_data.key / 12;
                        print!("o{}", octave);
                    }
                    pre_tick = event.tick;
                }
                DataKind::NoteOff(key_data) => {
                    Self::calc_note(
                        event.tick - pre_tick,
                        Self::get_note(key_data.key),
                        &mut mml_status,
                    );
                    pre_tick = event.tick;
                }
                DataKind::ChangeTempo(tempo) => {
                    Self::calc_rest(event.tick - pre_tick, &mut mml_status);
                    print!("t{}", tempo);
                    pre_tick = event.tick;
                }
                DataKind::ChangeTimbre(timbre) => {
                    print!("@{}", timbre);
                }
            }
        }
        print!("\n");
    }
}
