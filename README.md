# Mélodium

Mélodium is a dataflow-oriented language, focusing on treatments applied on data, allowing high scalability and massive parallelization safely.

## Introduction

Mélodium is a tool and language for manipulation of large amount of data, using the definition of treatments that applies on data through connections, with a track approach that makes any script higly scalable and implicitly parallelizable.

Mélodium is **under development** and still being defined and improved to become fully operationnal. The development documentation is available at <https://qvignaud.gitlab.io/melodium-rust/>.

This project was started as tool for experiments for signal analysis and musical informations retrieval. The first implementation was in C++ and is still available at <https://gitlab.com/qvignaud/Melodium>.

### Example

The following code computes spectrum of audio files contained in the path given as `directory` and make pictures of them:
```
use core/file::FileManager
use core/file::FlatFile

use core/audio::AudioManager
use core/audio::Decoder
use core/audio::Signal

use core/signal::FrameCutter
use core/signal::Windowing
use core/signal::Spectrum

use core/image::SimpleImageRender

model Files(directory: String): FileManager
{
    directory = directory
}

model AudioEngine(): AudioManager
{
    sampleRate = 44100
}

sequence Main(directory: String)
    model Files: Files(directory=directory)
    model Audio: AudioEngine()
{
    ReadAudioFiles[Files=Files, Audio=Audio]()

    AudioToImage[AudioManager=Audio]()
}

sequence ReadAudioFiles[Files: FileManager, Audio: AudioManager]()
    origin File: FlatFile[Files=Files]()
{
    Decoder[AudioManager=Audio]()
    
    File.data -> Decoder.data
}

sequence ComputeSpectrum(frameSize: Int, hopSize: Int, windowingType: String)
    input signal: Vec<Int>
    output spectrum: Mat<Int>
{
    FrameCutter(frameSize=frameSize, hopSize=hopSize, startFromZero=true, lastFrameToEndOfFile=true)
    Windowing(type=windowingType, size=frameSize)
    Spectrum(size=frameSize)

    Self.signal -> CoreFrameCutter.signal,frame -> CoreWindowing.frame,frame -> CoreSpectrum.frame,spectrum -> Self.spectrum
}

sequence AudioToImage[AudioManager: AudioManager](frameSize: Int = 4096, hopSize: Int = 2048, windowingType: String = "blackmanharris92")
    origin AudioSignal: Signal[AudioManager=AudioManager]()
    require @File
    require @Signal
{
    ComputeSpectrum(frameSize=frameSize, hopSize=hopSize, windowingType=windowingType)
    Image: SimpleImageRender(fileName=@File[name], format="png")
    
    AudioSignal.signal -> Spectrum.signal,spectrum -> Image.input
}
```

A more complete and commented version is available under [examples/semantic](examples/semantic/simple_build.mel).

## Compilation

Mélodium is fully written in Rust, and just need usual `cargo build` and `cargo test`.
```shell
git clone https://gitlab.com/qvignaud/melodium-rust.git
cd melodium-rust
cargo build
```

## Licence

This software is free and open-source, under the EUPL licence.

Why this one specifically? Well, as this project have a particular relationship with cultural world, probably more than most other softwares, it is important to have a strong legal basis covering also the notion of artwork.
In the same way, as *no culture is more important than another*, it was important to have a licence readable and understanble by most of people. The EUPL is available and *legally valid* in 23 languages, covering a large number of people.

Then, the legal part:
> Licensed under the EUPL, Version 1.1 or - as soon they will be approved by the European Commission - subsequent versions of the EUPL (the "Licence"); You may not use this work except in compliance with the Licence. You may obtain a copy of the Licence at: https://joinup.ec.europa.eu/software/page/eupl
>
>Unless required by applicable law or agreed to in writing, software distributed under the Licence is distributed on an "AS IS" basis, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the Licence for the specific language governing permissions and limitations under the Licence.

And do not worry, this licence is explicitly compatible with the ones mentionned in its appendix, including most of the common open-source licences.

