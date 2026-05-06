use melodium_core::{executive::*, *};
use melodium_macro::mel_data;

/// Metadata describing an audio stream.
///
/// `AudioInfo` is produced by decoding treatments and carries the properties of the source
/// audio that were detected during format probing and codec initialisation.
///
/// - `codec` — short name of the audio codec (e.g. `"pcm_s16le"`, `"flac"`, `"mp3"`, `"aac"`).
/// - `channels` — number of audio channels in the original stream (e.g. `1` for mono, `2` for stereo).
/// - `sample_rate` — number of samples per second (e.g. `44100`, `48000`).
/// - `duration_seconds` — total duration of the stream in seconds, when known. `none` if the
///   container does not report a frame count (e.g. raw streams, some network sources).
#[mel_data(traits(ToString Display Serialize Deserialize))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub codec: string,
    pub channels: u32,
    pub sample_rate: u32,
    pub duration_seconds: Option<f64>,
}

impl ToString for AudioInfo {
    fn to_string(&self) -> string {
        format!(
            "AudioInfo {{ codec: {}, channels: {}, sample_rate: {}, duration_seconds: {:?} }}",
            self.codec, self.channels, self.sample_rate, self.duration_seconds
        )
    }
}

impl Display for AudioInfo {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "codec={} channels={} sample_rate={}Hz",
            self.codec, self.channels, self.sample_rate
        )?;
        if let Some(d) = self.duration_seconds {
            write!(f, " duration={d:.3}s")?;
        }
        Ok(())
    }
}
