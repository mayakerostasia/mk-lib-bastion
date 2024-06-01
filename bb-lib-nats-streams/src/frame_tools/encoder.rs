use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Encode<T>(pub T);

impl Encoder for Encode<String> {}
impl Encoder for Encode<Vec<u8>> {}

// Hmmmm?
// const key: &'static [u8; 8] =  &[ 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8 ];
// data.iter()
//     .zip(key)
//     .map(|(&x1, &x2)| x1 ^ x2)
//     .collect()
// let ret_buff = data.iter_mut().step_by(8)
//     .zip(_key)
//     .map(move |(x1, x2)| *x1 ^ x2)
//     .collect::<Vec<u8>>();

pub trait Encoder {
    fn encode(&self) -> Vec<u8>
    where
        Self: Serialize,
    {
        bincode::serialize(self).expect("Failed to encode")
    }
}

pub trait Decoder<'de, T> {
    fn decode(data: &'de [u8]) -> T
    where
        T: Deserialize<'de>,
    {
        bincode::deserialize::<T>(data).expect("Failed to decode")
    }
}
