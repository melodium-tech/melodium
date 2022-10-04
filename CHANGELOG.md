# Changelog

## [Unreleased]

### Bug fixes

- Models parameters assignation
- Standard library visibility for external crates
- Standard library arithmetic area fixes

### Changed

- Standard library documentation

## [v0.5.0] (2022-09-28)

### Features

- Embedding standard library
- Managing `jeu` file format
- Graph generation as SVG
- Internal designer allowing elements removal
- Scripts restitution from designers

### Changed

- New CLI
- Cleaning up obsolete 'empty' connections code

## [v0.4.0] (2022-07-28)

### Bug fixes

- Implemented core components missing in core collection
- Engine behavior with stdio fixed #45
- Builds step stop erasing some futures

### Features

- Supporting `0xFF` syntax for bytes #21
- Supporting `'§'` syntax for chars #21
- Implementing binary operations for `bool` and `byte` #32
- Conversion between `f32` and `f64` types #33
- Trigonometric operations for `f32` and `f64` #34
- Comparison operators for all numeric types #36
- Adding min/max selectors for all numeric types #35
- `StreamBlock` & `StreamVecBlock` able to stream any data type #37
- `StaticVecFill` & `StaticVecBlockFill` available #37
- `Emit`ters available for all types
- Adding filters for all types #40
- Adding counters, sizers, and fitters for all types #41
- Adding conversion from `byte` to any data kind #43
- Adding merge of streams of same types & structures #44
- Generic files reader & writer #27

### Changed

- Standard library reorganized
- Doc generation allowing Mermaid.js graphs
- `BlockStream` renamed `BlockAllStream` to match behavior #37
- `StreamBlock` behavior rectified for scalar values #37
- Data generation made through new standard library sequences #37
- Unused core elements removed #37
- When sigterm is not handled by script it asks for engine end
- Conversion to `byte` now gives vectors #42

## [v0.3.0] (2022-06-10)

### Bug fixes

- Unused outputs are valid
- Fixing Windows compilation (signals removed)

### Features

- Supporting functions calls #11
- Adding arithmetic function #15
- Adding arithmetic treatments #15
- Stdin and Stdout are now reachable #18
- Process signals can be handled #19
- `void` datatype added #25
- Universal scalar conversion to `void` #28
- Stream triggers for all types #28
- Static filling for all scalar types from `void` #29
- Universal vector conversion to `void` #31
- Linearizers and organizers available for all types #30
- Static filling for all vector types from `void` pattern #29

### Changed

- Treatments got internal buffers for processing inputs #23

## [v0.2.1] (2022-05-09)

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
