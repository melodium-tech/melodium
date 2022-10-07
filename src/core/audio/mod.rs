
pub mod input;
pub mod output;
pub mod encoding;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    encoding::register(&mut c);

    c.models.insert(&(input::AudioInputModel::descriptor() as std::sync::Arc<dyn ModelDescriptor>));
    input::receive_audio_source::register(&mut c);

    c.models.insert(&(output::AudioOutputModel::descriptor() as std::sync::Arc<dyn ModelDescriptor>));
    output::send_audio_treatment::register(&mut c);
}

