use midly::Header;
use midly::MidiMessage;
use midly::Smf;
use midly::Track;
use midly::TrackEvent;
use midly::TrackEventKind;
use midly::{MetaMessage, SmpteTime};

pub enum OwnedMetaMessage {
    TrackNumber(Option<u16>),
    Text(String),
    Copyright(String),
    TrackName(String),
    InstrumentName(String),
    Lyric(String),
    Marker(Vec<u8>),
    CuePoint(Vec<u8>),
    ProgramName(String),
    DeviceName(String),
    MidiChannel(u8),
    MidiPort(u8),
    EndOfTrack,
    Tempo(u32),
    SmpteOffset(SmpteTime),
    TimeSignature(u8, u8, u8, u8),
    KeySignature(i8, bool),
    SequencerSpecific(Vec<u8>),
    Unknown(u8, Vec<u8>),
}

fn to_owned(bytes: &[u8]) -> Result<String, crate::error::Error> {
    Ok(String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?)
}

impl<'a> TryFrom<&MetaMessage<'a>> for OwnedMetaMessage {
    type Error = crate::error::Error;
    fn try_from(value: &MetaMessage) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            MetaMessage::TrackNumber(v) => Self::TrackNumber(*v),
            MetaMessage::Text(v) => Self::Text(to_owned(v)?),
            MetaMessage::Copyright(v) => Self::Copyright(to_owned(v)?),
            MetaMessage::TrackName(v) => Self::TrackName(to_owned(v)?),
            MetaMessage::InstrumentName(v) => Self::InstrumentName(to_owned(v)?),
            MetaMessage::Lyric(v) => Self::Lyric(to_owned(v)?),
            MetaMessage::Marker(v) => Self::Marker(v.to_vec()),
            MetaMessage::CuePoint(v) => Self::CuePoint(v.to_vec()),
            MetaMessage::ProgramName(v) => Self::ProgramName(to_owned(v)?),
            MetaMessage::DeviceName(v) => Self::DeviceName(to_owned(v)?),
            MetaMessage::MidiChannel(v) => Self::MidiChannel((*v).into()),
            MetaMessage::MidiPort(v) => Self::MidiPort((*v).into()),
            MetaMessage::EndOfTrack => Self::EndOfTrack,
            MetaMessage::Tempo(v) => Self::Tempo((*v).into()),
            MetaMessage::SmpteOffset(v) => Self::SmpteOffset(*v),
            MetaMessage::TimeSignature(a, b, c, d) => Self::TimeSignature(*a, *b, *c, *d),
            MetaMessage::KeySignature(a, b) => Self::KeySignature(*a, *b),
            MetaMessage::SequencerSpecific(v) => Self::SequencerSpecific(v.to_vec()),
            MetaMessage::Unknown(a, b) => Self::Unknown(*a, b.to_vec()),
        })
    }
}

pub enum OwnedTrackEventKind {
    Midi { channel: u8, message: MidiMessage },
    SysEx(Vec<u8>),
    Escape(Vec<u8>),
    Meta(OwnedMetaMessage),
}

impl<'a> TryFrom<&TrackEventKind<'a>> for OwnedTrackEventKind {
    type Error = crate::error::Error;
    fn try_from(value: &TrackEventKind) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            TrackEventKind::Midi { channel, message } => Self::Midi {
                channel: (*channel).into(),
                message: *message,
            },
            TrackEventKind::SysEx(v) => Self::SysEx(v.to_vec()),
            TrackEventKind::Escape(v) => Self::Escape(v.to_vec()),
            TrackEventKind::Meta(v) => Self::Meta(TryFrom::try_from(v)?),
        })
    }
}

pub struct OwnedTrackEvent {
    pub delta: u32,
    pub kind: OwnedTrackEventKind,
}

impl<'a> TryFrom<&TrackEvent<'a>> for OwnedTrackEvent {
    type Error = crate::error::Error;

    fn try_from(value: &TrackEvent<'a>) -> Result<Self, Self::Error> {
        let kind = TryFrom::try_from(&value.kind)?;
        Ok(Self {
            delta: value.delta.into(),
            kind,
        })
    }
}

type OwnedTrack = Vec<OwnedTrackEvent>;

fn to_owned_track(track: &Track<'_>) -> Result<OwnedTrack, crate::error::Error> {
    track.iter().map(OwnedTrackEvent::try_from).collect()
}

pub struct OwnedSmf {
    pub header: Header,
    pub tracks: Vec<OwnedTrack>,
}

impl<'a> TryFrom<&Smf<'a>> for OwnedSmf {
    type Error = crate::error::Error;

    fn try_from(value: &Smf<'a>) -> Result<Self, Self::Error> {
        let tracks: Vec<OwnedTrack> = value
            .tracks
            .iter()
            .map(to_owned_track)
            .collect::<Result<Vec<_>, crate::error::Error>>()?;
        Ok(Self {
            header: value.header,
            tracks,
        })
    }
}
