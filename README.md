Waveformrs Waveform Image Generator 
===

A waveform generator for WAV files (Mp3 and FLAC support will be added)

Techs
===

* [Hound](https://github.com/ruuda/hound) A wav encoding library.
* [Image](https://github.com/PistonDevelopers/image) Encoding and decoding images in Rust.
* [Imageproc](https://github.com/PistonDevelopers/image) An image processing library based on the image library.
* [Serde](https://github.com/serde-rs/serde) Serialization framework.
* [Clap](https://github.com/kbknapp/clap-rs) A command line argument parser.


Prerequisites
===

Rust stable (last tested with rustc 1.77.1 on Feb 8, 2026)

```bash
$ rustup update stable
```


Install
===

```bash
$ git clone https://github.com/azizunsal/waveformrs.git
$ cd waveformrs
$ cargo build --release
$ echo "export PATH=$PWD/target/release:\$PATH" > .waveformrs
$ source .waveformrs

```

Run
===

```bash
$ waveformrs --help
Waveform Generator 0.1.0
Aziz Unsal - unsal.aziz@gmail.com

USAGE:
    waveformrs [OPTIONS] --input <WAV_FILE_NAME> --output <GENERATED_IMAGE_FILE_NAME>

OPTIONS:
    -e, --end <END_TIME>                        Not valid if zoom is specified.
    -h, --height <IMGE_HEIGHT>                  [default: 220]
        --help                                  Print help information
    -i, --input <WAV_FILE_NAME>                 Name of the wav file to be processed - full path.
    -o, --output <GENERATED_IMAGE_FILE_NAME>    Name of the waveform image file to be generated.
    -s, --start <START_TIME>                    [default: 0]
    -t, --theme <THEME>                         [possible values: Dot, Line]
    -V, --version                               Print version information
    -w, --width <IMAGE_WIDTH>                   [default: 1335]
    -z, --zoom <SAMPLES_PER_PIXEL>              [default: 0]
```

Usage
===

```bash

$ waveformrs -i ./resources/a2002011001-e02-16kHz.wav -o a2002011001-e02-16kHz.png
The waveform image has successfully been created. 'a2002011001-e02-16kHz-w1335-z1301-per200.png'

```

Note: `-o/--output` is used as a filename prefix. The actual output file names are generated
by appending `-w{width}-z{zoom}-per{percent}` and the `.png`/`.json` extensions.

![generated_waveform_image](./examples/a2002011001-e02-16kHz.png)

### Running the same Wav file with `Dot` theme
```bash

$ waveformrs -i ./resources/a2002011001-e02-16kHz.wav -o a2002011001-e02-16kHz-dot.png -t Dot
The waveform image has successfully been created. 'a2002011001-e02-16kHz-dot-w1335-z1301-per200.png'

```

![generated_waveform_image](./examples/a2002011001-e02-16kHz-dot.png)



Wav File Data Overview
===

The processed WAV file summary can be found generated json file. This file will be used to send waveform data to the JavaScript application. 
Look at the excerpt below from a generated JSON file. The original file is here : `./examples/a2002011001-e02-16kHz.json`

```json

{
    "source_file": "./resources/a2002011001-e02-16kHz.wav",
    "sample_rate": 16000,
    "bits": 16,
    "samples_per_pixel": 256,
    "time_duration": 54.3115625,
    "processed_time_duration": 6.4008912787271658,
    "samples_length": 6788,
    "samples": [
        {
            "min": -1,
            "max": 49,
            "rms": 24.183947
        },
        {
            "min": -49,
            "max": 67,
            "rms": 28.996902
        },
        {
            "min": -101,
            "max": 73,
            "rms": 34.32445
        },
        {
            "min": -261,
            "max": 81,
            "rms": 87.58449
        },
        {
            "min": -347,
            "max": 0,
            "rms": 194.36104
        }

```


TODOs
---

- [ ] Mp3 and FLAC file support will be added.
- [ ] Waveform borders will be modified to give offset.
- [ ] Wav file time info added as an option to the generated image file.
- [ ] Just create waveform data options will be added to serve as a JavaScript waveform generator application backend.
