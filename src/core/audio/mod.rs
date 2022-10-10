
pub mod input;
pub mod output;
pub mod encoding;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    encoding::register(&mut c);

    input::model_host::register(&mut c);
    input::receive_audio_source::register(&mut c);

    output::model_host::register(&mut c);
    output::send_audio_treatment::register(&mut c);
}

