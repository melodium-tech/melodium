
pub mod input;
pub mod output;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(input::AudioInputModel::descriptor() as Arc<dyn ModelDescriptor>));
    input::receive_audio_treatment::register(&mut c);

    c.models.insert(&(output::AudioOutputModel::descriptor() as Arc<dyn ModelDescriptor>));
    output::send_audio_treatment::register(&mut c);
}

