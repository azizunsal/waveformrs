#[macro_use]
extern crate clap;
extern crate hound;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate image;
extern crate imageproc;

use std::io;
use std::fs::File;
use std::io::BufWriter;

use clap::{Arg, App};
use hound::WavReader;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_line_segment_mut};

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
    let samples_per_pixel = matches.value_of("samples-per-pixel").unwrap().parse::<u32>().unwrap();
    let width = matches.value_of("image-width").unwrap().parse::<u32>().unwrap();
    let height = matches.value_of("image-height").unwrap().parse::<u32>().unwrap();
    let theme = value_t!(matches.value_of("waveform-theme"), WaveformThemes).unwrap_or_else(|_e| WaveformThemes::Line);

    // println!("Configurations [input={}, output={}, samples-per-pixel={}, width={}, height={}, theme={}]", 
    //     source_filename, target_filename, samples_per_pixel, width, height, theme);
    
    let filename_wo_extension = get_filename_without_extension(&target_filename);

    ApplicationConfig {
        theme: theme,
        image_width: width,
        image_height: height,
        samples_per_pixel: samples_per_pixel,
        source_file: source_filename.to_owned(),
        target_filename_prefix: filename_wo_extension.to_owned(),
    }
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

fn calculate_rms(samples: &Vec<i32>) -> f32 {
    let sqr_sum = samples.iter().fold(0.0, |sqr_sum, s| {
        let sample = *s as f32;
        sqr_sum + sample * sample
    });
    (sqr_sum / samples.len() as f32).sqrt()
}

fn extract_samples(filename: &str, samples_per_pixel: &u32, width: &u32) -> WavFileSummary {
    let mut reader: WavReader<io::BufReader<File>> = hound::WavReader::open(filename).unwrap();

    let samples: Vec<i32> = reader.samples::<i32>().map(|s| s.unwrap()).collect();
    
    let sample_length = reader.len();
    let file_duration = reader.duration() as f64;
    let spec = reader.spec();
    let total_time = file_duration / spec.sample_rate as f64;

    let (mut min, mut max) = (0, 0);

    let mut samples_overview: Vec<SampleOverview> = Vec::new();

    let mut count: u32 = 0;
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
    println!("wav file summary has written to the '{}' file.", &filename);
}

fn draw_waveform(samples: &Vec<SampleOverview>, filename: &str, width: u32, height: u32, theme: &WaveformThemes) {
    let audocity_waveform_color = Rgb([63, 77, 155]);
    let audocity_rms_color = Rgb([121, 128, 225]);
    
    let mut img: RgbImage = RgbImage::new(width as u32, height as u32);
    
    for x in 0..width {
        let index: usize = x as usize;

        if index == samples.len() - 1 {
            eprintln!("There is not enough samples!");
            break;
        }

        let ref sample_overview = samples[index];
        let mut min = sample_overview.min;
        let mut max = sample_overview.max;

        // Convert values from [-32768, 32767] to [0, 65536].
        if min < -32768 {min = -32768;}
        min = min + 32768;
        if max > 32767 {max = 32767;} 
        max = max + 32768;

        let mut rms = sample_overview.rms;

        if rms < -32768f32 {rms = -32768f32;}
        if rms > 32767f32 {rms = 32767f32;}
        
        rms = rms + 32768f32;

        // Scale to fit the bitmap
        let low_y = height  as i32 - min * height as i32 / 65536;
        let high_y = height  as i32 - max * height  as i32/ 65536;
        
        let rms_y = height as f32 - rms * height as f32 / 65536f32;
        let low_rms_y = height as f32 - rms_y;


        match theme {
            &WaveformThemes::Line => {
                draw_line_segment_mut(&mut img, (x as f32, low_y as f32), (x as f32, high_y as f32), audocity_waveform_color);
                // Draw RMS for this sample group.
                draw_line_segment_mut(&mut img, (x as f32, low_rms_y), (x as f32, rms_y), audocity_rms_color);
            },
            &WaveformThemes::Dot => {
                draw_line_segment_mut(&mut img, (x as f32, low_y as f32), (x as f32, low_y as f32), Rgb([255, 255, 0]));
                draw_line_segment_mut(&mut img, (x as f32, high_y as f32), (x as f32, high_y as f32), Rgb([255, 255, 0]));
                // Draw RMS for this sample group.
                draw_line_segment_mut(&mut img, (x as f32, low_rms_y), (x as f32, low_rms_y), Rgb([255,0,255]));
                draw_line_segment_mut(&mut img, (x as f32, rms_y), (x as f32, rms_y), Rgb([255,0,255])); 
            }
        };
    }
    img.save(&filename).unwrap();
    println!("{} successfully created.", filename);
}

fn main() {
    let config = parse_configuration_params();
    let summary: WavFileSummary = extract_samples(&config.source_file, &config.samples_per_pixel, &config.image_width);
    write_to_file(&(config.target_filename_prefix.to_owned() + ".json"), &summary);
    draw_waveform(&summary.samples,
                    &(config.target_filename_prefix.to_owned() + ".png"), 
                    config.image_width,
                    config.image_height,
                    &config.theme);
}
