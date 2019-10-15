#[macro_use]
extern crate clap;
extern crate hound;
#[macro_use]
extern crate serde_derive;
extern crate image;
extern crate imageproc;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::fs::File;
use std::io;
use std::io::BufWriter;

use clap::{App, Arg};
use hound::WavReader;
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;

use env_logger::Env;

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
    start_time: u32,
    end_time: i32,
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u32,
    source_file: String,
    target_filename_prefix: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WavFileSummary {
    source_file: String,
    sample_rate: u32,
    bits: u16,
    samples_per_pixel: u32,
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
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("WAV_FILE_NAME")
                .help("Name of the wav file to be processed - full path.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("GENERATED_IMAGE_FILE_NAME")
                .help("Name of the waveform image file to be generated.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("zoom")
                .short("z")
                .long("zoom")
                .value_name("SAMPLES_PER_PIXEL")
                .takes_value(true)
                .required(false)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("start-time")
                .short("s")
                .long("start")
                .value_name("START_TIME")
                .takes_value(true)
                .required(false)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("end-time")
                .short("e")
                .long("end")
                .value_name("END_TIME")
                .help("Not valid if zoom is specified.")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("image-width")
                .short("w")
                .long("width")
                .value_name("IMAGE_WIDTH")
                .takes_value(true)
                .required(false)
                .default_value("1335"),
        )
        .arg(
            Arg::with_name("image-height")
                .short("h")
                .long("height")
                .value_name("IMGE_HEIGHT")
                .takes_value(true)
                .required(false)
                .default_value("220"),
        )
        .arg(
            Arg::with_name("waveform-theme")
                .short("t")
                .long("theme")
                .value_name("THEME")
                .takes_value(true)
                .required(false)
                .possible_values(&WaveformThemes::variants()),
        )
        .get_matches();

    let source_filename = matches.value_of("input").unwrap();
    let target_filename = matches.value_of("output").unwrap();
    let end_time = matches.value_of("end-time");
    let samples_per_pixel = matches.value_of("zoom").unwrap().parse::<u32>().unwrap();
    let start_time = matches.value_of("start-time").unwrap().parse::<u32>().unwrap();
    let width = matches.value_of("image-width").unwrap().parse::<u32>().unwrap();
    let height = matches.value_of("image-height").unwrap().parse::<u32>().unwrap();
    let theme = value_t!(matches.value_of("waveform-theme"), WaveformThemes).unwrap_or_else(|_e| WaveformThemes::Line);

    if samples_per_pixel > 0 && end_time.is_some() {
        panic!("Zoom and end-time cannot be specified at the same time!");
    }

    let filename_wo_extension = get_filename_without_extension(&target_filename);

    let end_time = match end_time {
        None => {
            debug!("End time was not specified, assigned to -1.");
            -1
        }
        _ => end_time.unwrap().parse::<i32>().unwrap(),
    };

    let app_config: ApplicationConfig = ApplicationConfig {
        theme,
        start_time,
        end_time,
        image_width: width,
        image_height: height,
        samples_per_pixel,
        source_file: source_filename.to_owned(),
        target_filename_prefix: filename_wo_extension.to_owned(),
    };

    debug!("Current configuration is {:?}", app_config);
    app_config
}

fn get_filename_without_extension(filename: &str) -> &str {
    let index: Option<usize> = get_filename(filename, '.');
    match index {
        Some(index) => &filename[..index],
        None => filename,
    }
}

fn get_filename(filename: &str, seperator: char) -> Option<usize> {
    for (index, c) in filename.char_indices() {
        if c == seperator {
            return Some(index);
        }
    }
    None
}

// See what the RMS stand for https://manual.audacityteam.org/man/glossary.html#rms
fn calculate_rms(samples: &Vec<i32>) -> f32 {
    let sqr_sum = samples.iter().fold(0.0, |sqr_sum, s| {
        let sample = *s as f32;
        sqr_sum + sample * sample
    });
    (sqr_sum / samples.len() as f32).sqrt()
}

fn extract_samples(filename: &str, mut samples_per_pixel: u32, width: &u32) -> WavFileSummary {
    let mut reader: WavReader<io::BufReader<File>> = hound::WavReader::open(filename).unwrap();

    let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();
    let sample_length = reader.len();
    // println!("sample_length is {}", sample_length);
    let file_duration = reader.duration() as f64;
    // println!("Reader [duration='{}', length='{}'", reader.duration(), reader.len());
    // println!("file_duration is {}", file_duration);
    let spec = reader.spec();
    // println!("Spec is {:?}", spec);
    let total_time = file_duration / spec.sample_rate as f64;
    // println!("total_time is {}", total_time);

    if samples_per_pixel == 0 {
        warn!("No zoom specified, the whole file will be printed.");
        let temp_val = &(sample_length / width);
        samples_per_pixel = *temp_val;
        debug!(
            "Calculated samples per pixel(=zoom) according to the image width(='{}'px.) is {}",
            width, samples_per_pixel
        );
    }

    let (mut min, mut max) = (0, 0);

    let mut samples_overview: Vec<SampleOverview> = Vec::new();

    let mut count: u32 = 0;
    let mut rms_range: Vec<i32> = Vec::new();

    for i in 0..sample_length {
        let index: usize = i as usize;
        let sample = samples[index];
        rms_range.push(sample);
        if sample < min {
            min = sample
        }
        if sample > max {
            max = sample
        }

        count += 1;
        // println!("Count = {}, samples_per_pixel = {}", count, samples_per_pixel);
        if count == samples_per_pixel {
            let rms = calculate_rms(&rms_range);
            // println!("[min ={} max= {}, rms = {}]", min, max, rms);
            samples_overview.push(SampleOverview { min, max, rms });
            count = 0;
            min = 0;
            max = 0;
            rms_range = Vec::new();
        }
    }

    let image_duration = total_time as f64 / samples_overview.len() as f64 * *width as f64;
    debug!(
        "Processed time duration is '{}' secs. / Overall time is '{}' secs.",
        image_duration, total_time
    );

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
    debug!("The wav file summary has written to '{}'.", &filename);
}

fn draw_waveform(samples: &Vec<SampleOverview>, filename: &str, width: u32, height: u32, theme: &WaveformThemes) {
    let audocity_waveform_color = Rgb([63, 77, 155]);
    let audocity_rms_color = Rgb([121, 128, 225]);
    let mut img: RgbImage = RgbImage::new(width as u32, height as u32);

    for x in 0..width {
        let index: usize = x as usize;

        if index == samples.len() {
            error!("There is not enough samples!");
            break;
        }

        let sample_overview = &samples[index];
        let mut min = sample_overview.min;
        let mut max = sample_overview.max;

        // Convert values from [-32768, 32767] to [0, 65536].
        if min < -32768 {
            min = -32768;
        }
        min += 32768;
        if max > 32767 {
            max = 32767;
        }
        max += 32768;

        let mut rms = sample_overview.rms;

        if rms < -32768f32 {
            rms = -32768f32;
        }
        if rms > 32767f32 {
            rms = 32767f32;
        }
        rms += 32768f32;

        // Scale to fit the bitmap
        let low_y = height as i32 - min * height as i32 / 65536;
        let high_y = height as i32 - max * height as i32 / 65536;
        let rms_y = height as f32 - rms * height as f32 / 65536f32;
        let low_rms_y = height as f32 - rms_y;

        match theme {
            &WaveformThemes::Line => {
                draw_line_segment_mut(
                    &mut img,
                    (x as f32, low_y as f32),
                    (x as f32, high_y as f32),
                    audocity_waveform_color,
                );
                // Draw RMS for this sample group.
                draw_line_segment_mut(&mut img, (x as f32, low_rms_y), (x as f32, rms_y), audocity_rms_color);
            }
            &WaveformThemes::Dot => {
                draw_line_segment_mut(
                    &mut img,
                    (x as f32, low_y as f32),
                    (x as f32, low_y as f32),
                    Rgb([255, 255, 0]),
                );
                draw_line_segment_mut(
                    &mut img,
                    (x as f32, high_y as f32),
                    (x as f32, high_y as f32),
                    Rgb([255, 255, 0]),
                );
                // Draw RMS for this sample group.
                draw_line_segment_mut(
                    &mut img,
                    (x as f32, low_rms_y),
                    (x as f32, low_rms_y),
                    Rgb([255, 0, 255]),
                );
                draw_line_segment_mut(&mut img, (x as f32, rms_y), (x as f32, rms_y), Rgb([255, 0, 255]));
            }
        };
    }
    img.save(&filename).unwrap();
    info!("The waveform image has successfully been created. '{}'", filename);
}

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let config = parse_configuration_params();
    let summary: WavFileSummary = extract_samples(&config.source_file, config.samples_per_pixel, &config.image_width);
    let processing_percentage = ((&summary.processed_time_duration / &summary.time_duration) * 100_f64).round();
    let file_name = &(config.target_filename_prefix.to_owned()
        + "-w"
        + &config.image_width.to_string()
        + "-z"
        + &summary.samples_per_pixel.to_string()
        + "-per"
        + &processing_percentage.to_string());
    write_to_file(&(file_name.to_owned() + ".json"), &summary);
    draw_waveform(
        &summary.samples,
        &(file_name.to_owned() + ".png"),
        config.image_width,
        config.image_height,
        &config.theme,
    );
}
