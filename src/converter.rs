use midly::MidiMessage::{NoteOff, NoteOn, ProgramChange};
use midly::Timing::{Metrical, Timecode};
use midly::TrackEventKind;

mod event;
mod track;
use track::DataKind;
use track::Track;
mod key_data;
use key_data::KeyData;

pub struct Converter {
    data: Vec<u8>,
    ticks_per_beat: u32,
}

#[derive(Clone, Copy, PartialEq)]
struct TempoEvent {
    tick: u32,
    tempo: u32,
    enable: bool,
}

impl TempoEvent {
    fn new(tick: u32, tempo: u32) -> Self {
        TempoEvent {
            tick,
            tempo,
            enable: true,
        }
    }
}

impl Converter {
    pub fn new(data: Vec<u8>) -> Self {
        Converter {
            data,
            ticks_per_beat: 0,
        }
    }

    fn delete_duplicate_tempo(events: &mut Vec<TempoEvent>) {
        let mut events_delete: Vec<TempoEvent> = Vec::new();
        if events.len() > 0 {
            let mut pre_tempo = events[0].tempo;
            for index in 1..events.len() {
                if events[index].tempo == pre_tempo {
                    events_delete.push(events[index]);
                } else {
                    pre_tempo = events[index].tempo;
                }
            }
        }
        events.retain(|event| !events_delete.contains(event));
    }

    fn get_tempo(microseconds_per_beat: u32) -> u32 {
        let microseconds_per_beat_f = f64::from(microseconds_per_beat);
        let seconds_per_beat = microseconds_per_beat_f / 1_000_000.0;
        let beats_per_minute = 60.0 / seconds_per_beat;
        beats_per_minute as u32
    }

    fn map_range(value: u8, from_low: u8, from_high: u8, to_low: u8, to_high: u8) -> u8 {
        let normalized_value = (value - from_low) as f64 / (from_high - from_low) as f64;
        let mapped_value =
            (normalized_value * (to_high - to_low) as f64 + to_low as f64).round() as u8;

        mapped_value.clamp(to_low, to_high)
    }

    fn get_vol(vel: u8) -> u8 {
        Self::map_range(vel, 0, 127, 0, 100)
    }

    pub fn convert(&mut self) -> Result<(), String> {
        let smf = match midly::Smf::parse(&mut self.data) {
            Ok(result) => result,
            Err(err) => return Err(err.to_string()),
        };

        match smf.header.timing {
            Metrical(ticks_per_beat) => self.ticks_per_beat = ticks_per_beat.as_int() as u32,
            Timecode(ticks_per_frame, frames_per_second) => return Err(format!("Ticks per frame: {}\nFrames per second: {}\nMIDI files with timecode are not supported.",ticks_per_frame.as_f32(),frames_per_second)),
        }
        println!("ticks_per_beat {}", self.ticks_per_beat);
        println!("track num: {}", smf.tracks.len());

        let mut tracks: Vec<Track> = Vec::new();
        let mut tempo_events: Vec<TempoEvent> = Vec::new();
        for (track_num, track_events) in smf.tracks.iter().enumerate() {
            tracks.push(Track::new(track_num));
            println!("track {} has {} events", track_num, track_events.len());
            let mut ticks: u32 = 0;
            for track_event in track_events.iter() {
                ticks += track_event.delta.as_int();
                match track_event.kind {
                    TrackEventKind::Midi { channel, message } => {
                        match message {
                            NoteOff { key, vel } => {
                                tracks[track_num].push_event(
                                    channel.as_int(),
                                    ticks,
                                    DataKind::NoteOff(KeyData::new(key.as_int(), vel.as_int())),
                                );
                                //println!("Ticks:{} NoteOn Key:{} Vel:{}", ticks, key, vel);
                            }
                            NoteOn { key, vel } => {
                                if vel == 0 {
                                    tracks[track_num].push_event(
                                        channel.as_int(),
                                        ticks,
                                        DataKind::NoteOff(KeyData::new(key.as_int(), vel.as_int())),
                                    );
                                } else {
                                    tracks[track_num].push_event(
                                        channel.as_int(),
                                        ticks,
                                        DataKind::NoteOn(KeyData::new(
                                            key.as_int(),
                                            Self::get_vol(vel.as_int()),
                                        )),
                                    );
                                }
                                //println!("Ticks:{} NoteOn Key:{} Vel:{}", ticks, key, vel);
                            }
                            /*
                            Aftertouch { key, vel } => {
                                println!(
                                    "Ticks:{} Aftertouch Key:{} Vel:{}",
                                    ticks, key, vel
                                );
                            }
                            Controller { controller, value } => {
                                println!(
                                    "Ticks:{} Controller Controller:{} Value:{}",
                                    ticks, controller, value
                                );
                            }
                            */
                            ProgramChange { program } => {
                                tracks[track_num].push_event(
                                    channel.as_int(),
                                    ticks,
                                    DataKind::ChangeTimbre(program.as_int()),
                                );
                                //println!("Ticks:{} ProgramChange program:{}", ticks, program);
                            }
                            /*
                            ChannelAftertouch { vel } => {
                                println!("Ticks:{} ChannelAftertouch Vel:{}", ticks, vel);
                            }
                            PitchBend { bend } => {
                                println!("Ticks:{} PitchBend Bend:{:?}", ticks, bend);
                            }
                            */
                            _ => (),
                        }
                    }
                    /*
                    TrackEventKind::SysEx(data) => {
                        println!("SysEx Event - Data: {:?}", data);
                    }
                    TrackEventKind::Escape(data) => {
                        println!("Escape Event - Data: {:?}", data);
                    }
                    */
                    TrackEventKind::Meta(data) => {
                        if let midly::MetaMessage::Tempo(tempo) = data {
                            tempo_events
                                .push(TempoEvent::new(ticks, Self::get_tempo(tempo.as_int())));
                        }
                        //println!("Meta Event - Data: {:?}", data);
                    }
                    _ => (),
                }
            }
        }

        Self::delete_duplicate_tempo(&mut tempo_events);

        for index in 0..tracks.len() {
            if let Err(msg) = tracks[index].parse1(&mut tempo_events) {
                return Err(msg);
            }
        }

        for index in 0..tracks.len() {
            if tracks[index].events.len() > 0 {
                if let Err(msg) = tracks[index].parse2(self.ticks_per_beat, &mut tempo_events) {
                    return Err(msg);
                }
            }
        }

        for index in 0..tracks.len() {
            if tracks[index].events.len() > 0 {
                if let Err(msg) = tracks[index].parse3(&mut tempo_events) {
                    return Err(msg);
                }
            }
        }

        println!("Analysis complete!\n");

        for index in 0..tracks.len() {
            if tracks[index].events.len() > 0 {
                tracks[index].convert(self.ticks_per_beat);
            }
        }

        Ok(())
    }
}
