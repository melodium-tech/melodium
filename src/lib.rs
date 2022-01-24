
//! Mélodium is a dataflow-oriented language, focusing on treatments applied on data, allowing high scalability and massive parallelization safely.
//! 
//! ## Introduction
//! 
//! Mélodium is a tool and language for manipulation of large amount of data, using the definition of treatments that applies on data through connections, with a track approach that makes any script higly scalable and implicitly parallelizable.
//! 
//! For more exhaustive explanations, please refer to [the Mélodium Language book](https://melodium.gitlab.io/book/).
//! 
//! Mélodium is _under development_ and continously being defined and improved. The development documentation is available at <https://melodium.gitlab.io/melodium/melodium/>, and the core reference at <https://melodium.gitlab.io/melodium/reference/>.
//! 
//! ## Origin
//! 
//! Mélodium were first developed during research in signal analysis and musical information retrieval, in need of a tool to manage large amount of records and easily write experimentations, without being concerned of underlying technical operations. It has been presented in [this thesis](https://www.researchgate.net/publication/344327676_Detection_et_classification_des_notes_d'une_piste_audio_musicale) (in French).
//! 
//! The first implementation was in C++ and ran well on high performance computers, such as those of Compute Canada. That tool appeared to be really useful, and the concepts used within its configuration language to deserve more attention.
//! 
//! The current project is the continuation of that work, rewritten from ground in Rust, and redesigned with a general approach of massively multithreaded data flows in mind.


#[macro_use]
extern crate lazy_static;

pub mod core;
pub mod executive;
pub mod logic;
pub mod script;