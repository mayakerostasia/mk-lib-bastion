use std::fmt::Debug;

use tower::BoxError;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Serialize, Deserialize)]
pub struct Encode<T>(pub T);

impl Encoder for Encode<String> {}
impl Encoder for Encode<Vec<u8>> {}
impl Encoder for bytes::Bytes {}

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
    fn encode(&self) -> Result<Vec<u8>, BoxError>
    where
        Self: Serialize + Debug
    {
        debug!("Encoder Started");
        let ser = bincode::serialize(self)?;
        debug!("Encoder Finished");
        Ok(ser)
    }
}

pub trait Decoder<'de, T> {
    fn decode(data: &'de [u8]) -> Result<T, BoxError>
    where
        T: Deserialize<'de>,
        T: Debug,
    {
        debug!("Decoder Started");
        let deser = bincode::deserialize::<T>(data)?;
        debug!("Decoder Finished: {deser:?}");
        Ok(deser)
    }
}
