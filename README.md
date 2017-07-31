Waveformrs Waveform Image Generator 
===

A waveform generator for WAV files (Mp3 and FLAC support will be added)

Install
===

```
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
    waveformrs [OPTIONS] --input <wav file name> --output <image file name>

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --height <image-height>                     [default: 250]
    -w, --width <image-width>                       [default: 800]
    -i, --input <wav file name>                    Name of the wav file to be processed - full path.
    -o, --output <image file name>                 Name of the waveform image file to be generated.
    -s, --samples-per-pixel <samples per pixel>     [default: 256]
    -t, --theme <waveform-theme>                    [values: Dot, Line]
```

Usage
===

```bash

$ waveformrs -i ./resources/a2002011001-e02-16kHz.wav -o a2002011001-e02-16kHz.png
wav file summary has written to the 'a2002011001-e02-16kHz.json' file.
a2002011001-e02-16kHz.png successfully created.

```

![generated_waveform_image](./examples/a2002011001-e02-16kHz.png)

### Running the same Wav file with `Dot` theme
```bash

$ waveformrs -i ./resources/a2002011001-e02-16kHz.wav -o a2002011001-e02-16kHz-dot.png -t Dot
wav file summary has written to the 'a2002011001-e02-16kHz-dot.json' file.
a2002011001-e02-16kHz-dot.png successfully created.

```

![generated_waveform_image](./examples/a2002011001-e02-16kHz-dot.png)


TODOs
===

1. Mp3 and FLAC file support will be added.
2. Waveform borders will be modified to give offset.
3. Wav file time info added as an option to the generated image file.
4. Just create waveform data options will be added to serve as a JavaScript waveform generator application backend.


