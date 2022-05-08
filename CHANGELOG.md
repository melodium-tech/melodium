# Changelog

## Unreleased

### Bug fixes
- Avoiding panic on absent parameter having default value

### Features
- Extending standard library
- Proposing documentation generator #5
- Providing text encoding and decoding #12

## [v0.2.0] (2022-04-25)
### Bug fixes
- Cleaning all warnings present in code

### Features
- Providing a standard library
- Adding host system sound connection
- Adding Wave audio format decoding and encoding
- Providing multiple text formats decoding
- Improving errors reporting and diagnostic #2
- Making tracks checks, reporting infinite loop and missing input #8
- Adding new syntax for `const`/`var` parameters #9

### Changed
- Improving transmission implementation between treatments #4
- Improving tracks scheduling and execution #10
- Providing generic model structure #7
- Providing generic treatment structure #4
- Refactoring core implementations #3 #4 #6 #7
- Using Rust 1.60 as minimal version #3
- Keeping tracks inheritance informations #10
