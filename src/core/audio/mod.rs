
pub mod input;

use crate::core::prelude::*;

pub fn register(mut c: &mut CollectionPool) {

    c.models.insert(&(input::AudioInputModel::descriptor() as Arc<dyn ModelDescriptor>));
    input::receive_audio_treatment::register(&mut c);
}

