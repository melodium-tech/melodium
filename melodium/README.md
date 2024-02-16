# Mélodium

Mélodium is a dataflow-oriented language, focusing on treatments applied on data, allowing high scalability and massive parallelization safely.

## Introduction

Mélodium is a tool and language for manipulation of large amount of data, using the definition of treatments that applies on data through connections, with a track approach that makes any script higly scalable and implicitly parallelizable.

For more exhaustive explanations, please refer to [the Mélodium Language book](https://doc.melodium.tech/book/).

Mélodium is _under development_ and continously being defined and improved. Released documentation is available on [docs.rs](https://docs.rs/melodium/latest/melodium/) and standard reference on [melodium.tech](https://doc.melodium.tech/latest/).

## Download & installation

Mélodium releases are available for multiple platforms at [Mélodium.tech](https://melodium.tech/).

## Command-line usage

Mélodium is distributed as a program that have extensive and growing CLI, here are very basic examples.

Launch a Mélodium program:
```shell
melodium <FILE>
```
or
```shell
melodium run <FILE>
```
or launch a specific command in Mélodium program:
```shell
melodium run <FILE> <CMD> [ARGS…]
```

See the commands and options of a program:
```shell
melodium info <FILE>
```

Check a Mélodium program validity:
```shell
melodium check <FILE>
```

To see the exhaustive commands and options list:
```shell
melodium help
```
Please refer to the [Mélodium Project](https://melodium.tech/), [Mélodium Book](https://doc.melodium.tech/book/),
or [Mélodium Documentation](https://doc.melodium.tech/latest/) for usage and more examples.

## Compilation

### Compile from source

Mélodium is fully written in Rust, and just need usual `cargo build`.
```shell
git clone https://gitlab.com/melodium/melodium.git
cd melodium
cargo build --package melodium
```
### Install from crates.io

Mélodium can also be directly installed from [crates.io](https://crates.io/crates/melodium).
```shell
cargo install melodium
```

## Development

The development of Mélodium project is hosted by [GitLab](https://gitlab.com/melodium/melodium).
Direct channels and news are available on [Discord](https://discord.gg/GQmckruKNx).

## Origin

Mélodium were first developed during research in signal analysis and musical information retrieval, in need of a tool to manage large amount of records and easily write experimentations, without being concerned of underlying technical operations. It has been presented in [this thesis](https://www.researchgate.net/publication/344327676_Detection_et_classification_des_notes_d'une_piste_audio_musicale) (in French).

The first implementation was in C++ and ran well on high performance computers, such as those of Compute Canada. That tool appeared to be really useful, and the concepts used within its configuration language to deserve more attention. This first experimental design is still available at <https://gitlab.com/qvignaud/Melodium>.

The current project is the continuation of that work, rewritten from ground in Rust, and redesigned with a general approach of massively multithreaded data flows in mind.


## Licence

This software is free and open-source, under the EUPL licence.

Why this one specifically? Well, as this project have a particular relationship with cultural world, probably more than most other softwares, it is important to have a strong legal basis covering also the notion of artwork.
In the same way, as *no culture is more important than another*, it was important to have a licence readable and understanble by most of people. The EUPL is available and *legally valid* in 23 languages, covering a large number of people.

Then, the legal part:
> Licensed under the EUPL, Version 1.2 or - as soon they will be approved by the European Commission - subsequent versions of the EUPL (the "Licence"); You may not use this work except in compliance with the Licence. You may obtain a copy of the Licence at: <https://joinup.ec.europa.eu/software/page/eupl>
>
>Unless required by applicable law or agreed to in writing, software distributed under the Licence is distributed on an "AS IS" basis, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the Licence for the specific language governing permissions and limitations under the Licence.

And do not worry, this licence is explicitly compatible with the ones mentionned in its appendix, including most of the common open-source licences.

