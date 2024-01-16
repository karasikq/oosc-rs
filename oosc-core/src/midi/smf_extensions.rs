use midly::live::LiveEvent;
use midly::Header;
use midly::MidiMessage;
use midly::Smf;
use midly::Track;
use midly::TrackEvent;
use midly::TrackEventKind;
use midly::{MetaMessage, SmpteTime};

pub enum OwnedMetaMessage {
    TrackNumber(Option<u16>),
    Text(Vec<u8>),
    Copyright(Vec<u8>),
    TrackName(Vec<u8>),
    InstrumentName(Vec<u8>),
    Lyric(Vec<u8>),
    Marker(Vec<u8>),
    CuePoint(Vec<u8>),
    ProgramName(Vec<u8>),
    DeviceName(Vec<u8>),
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

impl<'a> TryFrom<&MetaMessage<'a>> for OwnedMetaMessage {
    type Error = crate::error::Error;
    fn try_from(value: &MetaMessage) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            MetaMessage::TrackNumber(v) => Self::TrackNumber(*v),
            MetaMessage::Text(v) => Self::Text(v.to_vec()),
            MetaMessage::Copyright(v) => Self::Copyright(v.to_vec()),
            MetaMessage::TrackName(v) => Self::TrackName(v.to_vec()),
            MetaMessage::InstrumentName(v) => Self::InstrumentName(v.to_vec()),
            MetaMessage::Lyric(v) => Self::Lyric(v.to_vec()),
            MetaMessage::Marker(v) => Self::Marker(v.to_vec()),
            MetaMessage::CuePoint(v) => Self::CuePoint(v.to_vec()),
            MetaMessage::ProgramName(v) => Self::ProgramName(v.to_vec()),
            MetaMessage::DeviceName(v) => Self::DeviceName(v.to_vec()),
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

impl<'a> TryFrom<&LiveEvent<'a>> for OwnedTrackEventKind {
    type Error = crate::error::Error;

    fn try_from(value: &LiveEvent<'a>) -> Result<Self, Self::Error> {
        Ok(match value {
            LiveEvent::Midi { channel, message } => Self::Midi {
                channel: (*channel).into(),
                message: *message,
            },
            LiveEvent::Common(_) => todo!(),
            LiveEvent::Realtime(_) => todo!(),
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

impl<'a> TryFrom<&LiveEvent<'a>> for OwnedTrackEvent {
    type Error = crate::error::Error;

    fn try_from(value: &LiveEvent<'a>) -> Result<Self, Self::Error> {
        let kind: OwnedTrackEventKind = TryFrom::try_from(value)?;
        Ok(Self { delta: 0, kind })
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
