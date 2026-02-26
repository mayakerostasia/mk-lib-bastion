
use rkyv::{Archive, Deserialize, Serialize, rancor::Error, util::AlignedVec};
use std::fmt::Debug;
use tower::BoxError;
use tracing::trace;

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct Encode<Frame>(pub Frame);

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

pub trait Encoder { }

pub trait Decoder { }
// pub trait Decoder<'de, T> {
//     fn decode(data: &'de [u8]) -> Result<T, BoxError>
//     where
//         T: Deserialize<'de>,
//         T: Debug,
//     {
//         trace!("Decoder Started");
//         let deser = bincode::deserialize::<T>(data)?;
//         trace!("Decoder Finished: {deser:?}");
//         Ok(deser)
//     }
// }
