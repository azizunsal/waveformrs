#[macro_use]
extern crate clap;
extern crate hound;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::io;
use std::fs::File;
use std::io::BufWriter;

use clap::{Arg, App};
use hound::WavReader;

arg_enum! {
    #[derive(Debug)]
    enum WaveformThemes {
        Dot,
        Line
    }
}

#[derive(Debug)]
struct ApplicationConfig {
    theme: WaveformThemes,
    image_width: u16,
    image_height: u16,
    samples_per_pixel: u16,
    source_file: String,
    target_filename_prefix: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WavFileSummary {
    source_file: String,
    sample_rate: u32,
    bits: u16,
    samples_per_pixel: u16,
    time_duration: f64,
    processed_time_duration: f64,
    samples_length: usize,
    samples: Vec<SampleOverview>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SampleOverview {
    min: i32,
    max: i32,
    rms: f32,
}


fn parse_configuration_params() -> ApplicationConfig {
    let matches = App::new("Waveform Generator")
                    .version("0.1.0")
                    .author("Aziz Unsal - unsal.aziz@gmail.com")
                    .arg(Arg::with_name("input")
                                .short("i")
                                .long("input")
                                .value_name("wav file name")
                                .help("Name of the wav file to be processed - full path.")
                                .takes_value(true)
                                .required(true))
                    .arg(Arg::with_name("output")
                                .short("o")
                                .long("output")
                                .value_name("image file name")
                                .help("Name of the waveform image file to be generated.")
                                .takes_value(true)
                                .required(true))
                    .arg(Arg::with_name("samples-per-pixel")
                                .short("s")
                                .long("samples-per-pixel")
                                .value_name("samples per pixel")
                                .takes_value(true)
                                .default_value("256")
                                .required(false))
                    .arg(Arg::with_name("image-width")
                            .short("w")
                            .long("width")
                            .takes_value(true)
                            .required(false)
                            .default_value("800"))
                    .arg(Arg::with_name("image-height")
                            .short("h")
                            .long("height")
                            .takes_value(true)
                            .required(false)
                            .default_value("250"))
                    .arg(Arg::with_name("waveform-theme")
                            .short("t")
                            .long("theme")
                            .takes_value(true)
                            .required(false)
                            .possible_values(&WaveformThemes::variants()))
                    .get_matches();

    let source_filename = matches.value_of("input").unwrap();
    let target_filename = matches.value_of("output").unwrap();
    let samples_per_pixel = matches.value_of("samples-per-pixel").unwrap().parse::<u16>().unwrap();
    let width = matches.value_of("image-width").unwrap().parse::<u16>().unwrap();
    let height = matches.value_of("image-height").unwrap().parse::<u16>().unwrap();
    let theme = value_t!(matches.value_of("waveform-theme"), WaveformThemes).unwrap_or_else(|_e| WaveformThemes::Line);

    // println!("Configurations [input={}, output={}, samples-per-pixel={}, width={}, height={}, theme={}]", 
    //     source_filename, target_filename, samples_per_pixel, width, height, theme);
    
    let target_filename_parts: Vec<&str> = target_filename.split(".").collect();
    let target_filename_prefix = target_filename_parts[0];
    // println!("Target file name prefix is {}", target_filename_prefix);

    ApplicationConfig {
        theme: theme,
        image_width: width,
        image_height: height,
        samples_per_pixel: samples_per_pixel,
        source_file: source_filename.to_owned(),
        target_filename_prefix: target_filename_prefix.to_owned(),
    }
}

fn calculate_rms(samples: &Vec<i32>) -> f32 {
    let sqr_sum = samples.iter().fold(0.0, |sqr_sum, s| {
        let sample = *s as f32;
        sqr_sum + sample * sample
    });
    (sqr_sum / samples.len() as f32).sqrt()
}

fn extract_samples(filename: &str, samples_per_pixel: &u16, width: &u16) -> WavFileSummary {
    let mut reader: WavReader<io::BufReader<File>> = hound::WavReader::open(filename).unwrap();

    let samples: Vec<i32> = reader
                        .samples::<i32>()
                        .map(|s| s.unwrap())
                        .collect();
    
    let sample_length = reader.len();
    let file_duration = reader.duration() as f64;
    let spec = reader.spec();
    let total_time = file_duration / spec.sample_rate as f64;

    let (mut min, mut max) = (0, 0);

    let mut samples_overview: Vec<SampleOverview> = Vec::new();

    let mut count: u16 = 0;
    let mut rms_range: Vec<i32> = Vec::new();

    for i in 0..sample_length {
        let index: usize = i as usize;
        let sample = samples[index];
        rms_range.push(sample);
        if sample < min {min = sample}
        if sample > max {max = sample}

        count += 1;
        if count == *samples_per_pixel {
            let rms = calculate_rms(&rms_range);
            // println!("[min ={} max= {}, rms = {}]", min, max, rms);
            samples_overview.push(SampleOverview {min: min, max: max, rms: rms});
            count = 0;
            min = 0;
            max = 0;
            rms_range = Vec::new();
        }
    }

    let image_duration = total_time as f64/ samples_overview.len() as f64 * *width as f64;

    WavFileSummary {
        source_file: filename.to_owned(),
        sample_rate: spec.sample_rate,
        bits: spec.bits_per_sample,
        samples_per_pixel: samples_per_pixel.to_owned(),
        time_duration: total_time,
        processed_time_duration: image_duration,
        samples_length: samples_overview.len(),
        samples: samples_overview,
    }
}

fn write_to_file(filename: &str, summary: &WavFileSummary) {
    let file = File::create(filename).expect("Unable to create file!");
    let bw = BufWriter::new(file);
    serde_json::to_writer(bw, summary).expect("Unable to write!");
}

fn main() {
    let config = parse_configuration_params();
    let summary: WavFileSummary = extract_samples(&config.source_file, &config.samples_per_pixel, &config.image_width);
    write_to_file(&(config.target_filename_prefix.to_owned() + ".json"), &summary);
}
