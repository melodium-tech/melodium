
#[derive(Debug)]
pub enum IO {
    Sequence(),
    Treatment(String)
}

#[derive(Debug)]
pub struct Connection {

    pub output_treatment: IO,
    pub output_name: String,

    pub input_treatment: IO,
    pub input_name: String,
    
}
