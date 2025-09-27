use std::io::Cursor;
use async_channel::bounded;
use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_treatment};
use symphonia::core::{codecs::{audio::AudioDecoderOptions, registry::CodecRegistry}, formats::{probe::{Hint, Probe}, FormatOptions}, io::{AsyncMediaSourceStream, MediaSourceStreamOptions}, meta::MetadataOptions};
use crate::channels::*;

#[mel_treatment(
    default format "wave"
    input data Stream<byte>
    output signal Stream<f32>
)]
pub async fn decode_mono(format: string) {

    let (sender, receiver) = bounded(1024);

    let media_source = AsyncMediaSourceStream::new(Box::pin(ChannelReaderMediaSource::new(receiver)), MediaSourceStreamOptions::default());

    let probe = Probe::new();
    if let Ok(mut format_reader) = probe.probe_async(&Hint::new(), media_source, FormatOptions::default(), MetadataOptions::default()).await {
        let track = format_reader.default_track(symphonia::core::formats::TrackType::Audio).cloned().unwrap();
        let mut decoder = CodecRegistry::new().make_audio_decoder(track.codec_params.as_ref().map(|codec| codec.audio()).flatten().unwrap(), &AudioDecoderOptions::default()).unwrap();

        while let Ok(Some(packet)) = format_reader.next_packet().await {
            let audio = decoder.decode(&packet).unwrap();
            audio.
        }
        
    }
    //let symphonia::default::formats::wave::AsyncWavReader::new

    while let Ok(data) = data.recv_many().await.map(|values| TryInto::<Vec<u8>>::try_into(values).unwrap()) {
        sender.send(data).await;
    }
    /*let decoder = match format.as_str() {
        "wave" | "wav" => {}
    };*/


}
