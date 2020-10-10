
//! Mélodium is a dataflow-oriented language, focusing on treatments applied on data, allowing high scalability and massive parallelization safely.
//! 
//! ## Introduction
//! 
//! Mélodium is a tool and language for manipulation of large amount of data, using the definition of treatments that applies on data through connections, with a track approach that makes any script higly scalable and implicitly parallelizable.
//! 
//! Mélodium is **under development** and still being defined and improved to become fully operationnal. Its project is hosted on GitLab at <https://gitlab.com/qvignaud/melodium-rust>.
//! 
//! ## Origin
//! 
//! Mélodium were first developed during research in signal analysis and musical information retrieval, in need of a tool to manage large amount of records and easily write experimentations, without being concerned of underlying technical operations. It has been presented in [this thesis](https://www.researchgate.net/publication/344327676_Detection_et_classification_des_notes_d'une_piste_audio_musicale)(in French).
//! 
//! The first implementation was in C++ and ran well on high performance computers, such as those of Compute Canada. That tool appeared to be really useful, and the concepts used within its configuration language to deserve more attention. This first experimental design is still available at <https://gitlab.com/qvignaud/Melodium>.
//! 
//! The current project is the continuation of that work, rewritten from ground in Rust, and redesigned with a general approach of massively multithreaded data flows in mind.


#[macro_use]
extern crate lazy_static;

pub mod executive;
pub mod logic;
pub mod script;