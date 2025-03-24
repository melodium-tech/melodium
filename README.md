
# Mélodium

Mélodium is a stream-oriented language, focusing on events and treatments, enabling strong scalability and parallelization safely.

## Description

Mélodium is a tool and language for data stream manipulation, event reaction, building software shared across different machines.  
This is accomplished using the concept of treatments that applies on data, with a track approach that makes any program scalable, implicitly parallelizable, and asynchronous.

Fitting uses cases are, among others:
- managing many small requests;
- optimizing execution schedule;
- managing large amount of data coming as stream or chunked;
- dealing with limited resources;
- sharing program load across different machines.

For more exhaustive explanations, please refer to [the Mélodium Language book](https://doc.melodium.tech/book/en/).

Mélodium is _under development_ and continously being defined and improved. Released documentation is available on [docs.rs](https://docs.rs/melodium/latest/melodium/) and standard reference on [melodium.tech](https://doc.melodium.tech/latest/en/).

## Run and execute

Please refer to the [Mélodium crate](melodium/) for informations about how to build and run Mélodium.

## Development

Mélodium development repository is hosted on [GitLab](https://gitlab.com/melodium/melodium), with mirroring on [GitHub](https://github.com/melodium-tech/melodium).
Social accounts can be found on [BlueSky](https://bsky.app/profile/melodium.tech), [Mastodon](https://mastodon.social/@melodium) and [LinkedIn](https://www.linkedin.com/company/m%C3%A9lodium/).
Development chats and related Mélodium stuff can be discussed on [Discord](https://discord.gg/GQmckruKNx).

## Repository organization

### melodium-*

Those folders corresponds to the different crates composing the Mélodium main program.
They follow the cargo pattern with internal `src/`, `tests/`, etc. They only contains Rust code.

### libs/*-mel

Those sub-folders each correspond to a standard Mélodium library/package.
While they are technically Rust libraries, they don't make sense without Mélodium context, so the `-mel` suffix is added.
They can contain both Rust and Mélodium code (located in `mel/`); apart from that, they are usual Rust crates.

### tests

Here are Mélodium integration tests.
Those tests are managed by crates in `testers/` subfolder, and are living in their own workspace.
They should not rely directly on any Mélodium Rust implementation, and only use it as external executable.

## CI/CD

The CI/CD runs on GitLab, and is configured to respond to main variables:
- `CHECK`: when set to `true` this will run `cargo check` for all supported platforms and all registered combinations of features for every crate;
- `TEST`: when set to `true` this will build debug versions of Mélodium and testers for all supported platforms and test them[^1];
- `BUILD_DEBUG`: when set to `true` this will build debug versions of Mélodium for all supported platforms (implied when using `TEST`);
- `BUILD_RELEASE`: when set to `true` this will build release versions of Mélodium for all supported platforms;
- `BUILD_PACKAGES`: when set to `true` this will build the release webassembly version of every `lib/` crate;
- `BUILD_CONTAINERS`: when set to `true` this will build the container images of Mélodium and push it to internal container registry.

As example, to run checks or tests on a specific pushed commit:
- `git push -o ci.variable="CHECK=true"`
- `git push -o ci.variable="TEST=true"`
- `git push -o ci.variable="CHECK=true" -o ci.variable="TEST=true"`

`CHECK` and `TEST` are considered `true` for every commit on `master` branch, and _everything_ is considered `true` for tagged commits.

[^1]: while build is made for all supported platforms, tests are only processed on some platorms where GitLab CI have matching native runners, such as Linux aarch64 and x86_64 (testing aarch64, i686 and x86_64 versions, GNU and MUSL), Windows x86_64 (testing i686 and x86_64 versions, GNU and MSVC), and Mac OS ARM64 (testing arm64 version).

## Licence

This software is free and open-source, under the EUPL licence.

Why this one specifically? Well, as this project have a particular relationship with cultural world, probably more than most other softwares, it is important to have a strong legal basis covering also the notion of artwork.
In the same way, as *no culture is more important than another*, it was important to have a licence readable and understanble by most of people. The EUPL is available and *legally valid* in 23 languages, covering a large number of people.

Then, the legal part:
> Licensed under the EUPL, Version 1.2 or - as soon they will be approved by the European Commission - subsequent versions of the EUPL (the "Licence"); You may not use this work except in compliance with the Licence. You may obtain a copy of the Licence at: https://joinup.ec.europa.eu/software/page/eupl
>
> Unless required by applicable law or agreed to in writing, software distributed under the Licence is distributed on an "AS IS" basis, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the Licence for the specific language governing permissions and limitations under the Licence.

And do not worry, this licence is explicitly compatible with the ones mentionned in its appendix, including most of the common open-source licences.

