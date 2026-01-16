/// Audio format definitions

/// Sample rate in Hertz
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleRate {
    Hz44100 = 44100,
    Hz48000 = 48000,
    Hz88200 = 88200,
    Hz96000 = 96000,
}

impl SampleRate {
    pub fn hz(self) -> u32 {
        self as u32
    }

    pub fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            44100 => Some(Self::Hz44100),
            48000 => Some(Self::Hz48000),
            88200 => Some(Self::Hz88200),
            96000 => Some(Self::Hz96000),
            _ => None,
        }
    }
}

/// Number of audio channels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelCount {
    Mono = 1,
    Stereo = 2,
}

impl ChannelCount {
    pub fn count(self) -> usize {
        self as usize
    }

    pub fn from_count(count: usize) -> Option<Self> {
        match count {
            1 => Some(Self::Mono),
            2 => Some(Self::Stereo),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_rate() {
        assert_eq!(SampleRate::Hz48000.hz(), 48000);
        assert_eq!(SampleRate::from_hz(48000), Some(SampleRate::Hz48000));
    }

    #[test]
    fn test_channel_count() {
        assert_eq!(ChannelCount::Stereo.count(), 2);
        assert_eq!(ChannelCount::from_count(2), Some(ChannelCount::Stereo));
    }
}
