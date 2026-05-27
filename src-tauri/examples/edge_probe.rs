use msedge_tts::{tts::client::connect, tts::SpeechConfig, voice::get_voices_list};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let voices = get_voices_list()?;
    let voice = voices
        .iter()
        .find(|voice| {
            voice.short_name.as_deref() == Some("en-US-EmmaMultilingualNeural")
                || voice.name.contains("EmmaMultilingualNeural")
        })
        .or_else(|| {
            voices.iter().find(|voice| {
                voice
                    .locale
                    .as_deref()
                    .map(|locale| locale.starts_with("en-"))
                    .unwrap_or(false)
            })
        })
        .ok_or("No English voice found")?;

    let mut config = SpeechConfig::from(voice);
    config.rate = 15;

    let mut client = connect()?;
    let audio = client.synthesize("Readtis Edge TTS probe.", &config)?;
    println!("audio_bytes={}", audio.audio_bytes.len());
    Ok(())
}
