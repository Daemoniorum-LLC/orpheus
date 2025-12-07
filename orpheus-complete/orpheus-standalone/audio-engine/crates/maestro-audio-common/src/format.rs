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
    fn test_sample_rate_hz48000() {
        assert_eq!(SampleRate::Hz48000.hz(), 48000);
        assert_eq!(SampleRate::from_hz(48000), Some(SampleRate::Hz48000));
    }

    #[test]
    fn test_sample_rate_all_rates() {
        assert_eq!(SampleRate::Hz44100.hz(), 44100);
        assert_eq!(SampleRate::Hz48000.hz(), 48000);
        assert_eq!(SampleRate::Hz88200.hz(), 88200);
        assert_eq!(SampleRate::Hz96000.hz(), 96000);
    }

    #[test]
    fn test_sample_rate_from_hz() {
        assert_eq!(SampleRate::from_hz(44100), Some(SampleRate::Hz44100));
        assert_eq!(SampleRate::from_hz(48000), Some(SampleRate::Hz48000));
        assert_eq!(SampleRate::from_hz(88200), Some(SampleRate::Hz88200));
        assert_eq!(SampleRate::from_hz(96000), Some(SampleRate::Hz96000));
    }

    #[test]
    fn test_sample_rate_from_hz_invalid() {
        assert_eq!(SampleRate::from_hz(22050), None);
        assert_eq!(SampleRate::from_hz(192000), None);
        assert_eq!(SampleRate::from_hz(0), None);
        assert_eq!(SampleRate::from_hz(999999), None);
    }

    #[test]
    fn test_sample_rate_equality() {
        assert_eq!(SampleRate::Hz48000, SampleRate::Hz48000);
        assert_ne!(SampleRate::Hz48000, SampleRate::Hz44100);
    }

    #[test]
    fn test_sample_rate_clone_copy() {
        let rate1 = SampleRate::Hz48000;
        let rate2 = rate1;
        assert_eq!(rate1, rate2);
    }

    #[test]
    fn test_channel_count_stereo() {
        assert_eq!(ChannelCount::Stereo.count(), 2);
        assert_eq!(ChannelCount::from_count(2), Some(ChannelCount::Stereo));
    }

    #[test]
    fn test_channel_count_mono() {
        assert_eq!(ChannelCount::Mono.count(), 1);
        assert_eq!(ChannelCount::from_count(1), Some(ChannelCount::Mono));
    }

    #[test]
    fn test_channel_count_from_count_invalid() {
        assert_eq!(ChannelCount::from_count(0), None);
        assert_eq!(ChannelCount::from_count(3), None);
        assert_eq!(ChannelCount::from_count(8), None);
        assert_eq!(ChannelCount::from_count(100), None);
    }

    #[test]
    fn test_channel_count_equality() {
        assert_eq!(ChannelCount::Mono, ChannelCount::Mono);
        assert_eq!(ChannelCount::Stereo, ChannelCount::Stereo);
        assert_ne!(ChannelCount::Mono, ChannelCount::Stereo);
    }

    #[test]
    fn test_channel_count_clone_copy() {
        let ch1 = ChannelCount::Stereo;
        let ch2 = ch1;
        assert_eq!(ch1, ch2);
    }
}
