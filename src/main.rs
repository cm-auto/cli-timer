use std::{
    borrow::Cow, error::Error, fs::File, io::Cursor, num::ParseIntError, thread, time::Duration,
};

use clap::Parser;
use rodio::{OutputStream, Sink, Source};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "media"]
struct MediaAssets;

#[derive(Debug, Parser)]
struct Arguments {
    #[clap(
        help = "The duration of the alarm in either minutes (e.g. 5) or [hours:]minutes:seconds (e.g. 1:30:10)"
    )]
    duration: String,
    #[clap(long, short, help = "The path to the alarm sound file. Optional")]
    alarm_path: Option<String>,
}

fn parse_duration_string(duration: &str) -> Result<Duration, ParseIntError> {
    // if string is just a plain number treat it as minutes
    if let Ok(duration) = duration.parse::<u64>() {
        return Ok(Duration::from_secs(duration * 60));
    }
    let mut duration = duration.split(':').rev();
    let seconds = duration.next();
    let minutes = duration.next();
    let hours = duration.next();
    // this also allows values like "1:76:99"
    Ok(Duration::from_secs(
        match hours.map(|raw| raw.parse::<u64>()) {
            Some(parse_result) => parse_result? * 3600,
            None => 0,
        } + match minutes.map(|raw| raw.parse::<u64>()) {
            Some(parse_result) => parse_result? * 60,
            None => 0,
        } + match seconds.map(|raw| raw.parse::<u64>()) {
            Some(parse_result) => parse_result?,
            None => 0,
        },
    ))
}

fn duration_to_string(duration: &Duration) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        duration.as_secs() / 3600,
        (duration.as_secs() % 3600) / 60,
        duration.as_secs() % 60
    )
}
fn main() {
    let arguments = Arguments::parse();
    let mut duration = parse_duration_string(&arguments.duration).unwrap();
    // TODO: as soon as "Duration::SECOND" is stable, use it
    let one_second = Duration::from_secs(1);

    // TODO: make cursor invisible

    // _output_stream can't be just '_', otherwise it would be immediately dropped
    // and the sound wouldn't play
    // TODO: should this make use of OnceCell?
    let (_output_stream, sink_option) = get_output_stream_and_sink(arguments.alarm_path.as_deref())
        .expect("Failed to get output stream and sink");
    let mut alarm_started = false;
    loop {
        let sign = if alarm_started { "-" } else { " " };
        // TODO: should negative be red?
        eprint!("\r{}{}", sign, duration_to_string(&duration));
        if duration == Duration::from_secs(0) {
            sink_option.play();
            alarm_started = true;
        }
        thread::sleep(one_second);
        if alarm_started {
            duration += one_second;
        } else {
            duration -= one_second;
        }
    }
}

fn get_output_stream_and_sink(
    audio_file_path: Option<&str>,
) -> Result<(OutputStream, Sink), Box<dyn Error>> {
    // Create a sink to play audio
    let (output_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    // Load an audio file

    // to make the Decoder be able to take its data either from a file or from a buffer
    // we need a trait that implements several other traits
    // TODO: check if there is a crate to make this easier
    trait ReadAndSeek: std::io::Read + std::io::Seek + std::marker::Send + std::marker::Sync {}
    impl ReadAndSeek for File {}
    impl ReadAndSeek for Cursor<Cow<'_, [u8]>> {}

    let data_source: Box<dyn ReadAndSeek> = match audio_file_path {
        None => {
            // uwraping here will not fail, IF the file exists
            // TODO: assert during compile time that the file exists
            // or even better check if there is some kind of macro in rust-embed that ensures the file exists
            // implicitly including the file does not help
            let embedded_file = MediaAssets::get("Fire_pager-jason-1283464858.mp3").unwrap();
            Box::new(Cursor::new(embedded_file.data))
        }
        Some(path) => Box::new(File::open(path)?),
    };
    let source = rodio::Decoder::new(data_source)?;

    sink.append(source.repeat_infinite());
    sink.pause();

    Ok((output_stream, sink))
}
