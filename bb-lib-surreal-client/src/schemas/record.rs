use crate::{error::SurrealClientError, Storable};
use bb_lib_nats_streams::{Encoder, Decoder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::error;
use std::fmt::Debug;
use surrealdb::sql::Id;
use bytes::Bytes;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record<D: Send + Clone> {
    // #[serde(skip_if_missing)]
    // id: Option<SurrealId>,
    #[serde(skip)]
    _id: Option<Id>,
    #[serde(skip)]
    _tb: String,
    #[serde(flatten)]
    _data: Option<Box<D>>,
    _meta: Option<Box<D>>,
}

impl<T> Encoder for Record<T> 
where
    T: Into<Bytes> + Clone + Send,
{}
impl<'a, T> Decoder<'a, T> for Record<T> 
where
    T: Into<Bytes> + Clone + Send,
{}

impl<T> From<Record<T>> for Bytes 
where
    T: Clone + Send + Debug + Serialize,
    Record<T>: Encoder,
{
    fn from(value: Record<T>) -> Self {
        value.encode().expect("Failed to encode Bytes").into()
    }
}

impl<T> From<Bytes> for Record<T> 
where
    T: Clone + Send + Debug + for <'de> Deserialize<'de> + Serialize + Into<Bytes>,
{
    fn from(value: Bytes) -> Self {
        match Record::decode(&value) {
            Ok(rec) => rec,
            Err(e) => {
                error!("Whoops! Error! -> {}", e);
                panic!("Couldn't convert Bytes to Record")
            }
        }
    }

}

impl<D: Send + Clone> Record<D> {
    pub fn new(tb: &str, id: Option<Id>, data: Option<Box<D>>, meta: Option<Box<D>>) -> Self {
        Self {
            _tb: tb.to_string(),
            _id: id,
            _data: data,
            _meta: meta,
        }
    }

    pub fn set_id(&mut self, id: &str) -> Result<(), SurrealClientError> {
        self._id = Some(Id::from(id));
        Ok(())
    }

    pub fn set_data(&mut self, data: Box<D>) -> Result<(), SurrealClientError> {
        self._data = Some(data);
        Ok(())
    }

    pub fn set_meta(&mut self, meta: Box<D>) -> Result<(), SurrealClientError> {
        self._meta = Some(meta);
        Ok(())
    }

    pub fn random_id(&mut self) -> Result<(), SurrealClientError> {
        self._id = Some(Id::rand());
        Ok(())
    }

    pub fn id(&self) -> Result<Id, SurrealClientError> {
        match &self._id {
            Some(id) => Ok(id.clone()),
            None => Err(SurrealClientError::NoID),
        }
    }

    pub fn tb(&self) -> &str {
        self._tb.as_str()
    }

    pub fn data(&self) -> Box<D> {
        self._data.clone().expect("No Data!")
    }
}

// impl<D: DBThings + Send> DBThings for Record<D> {}
impl<D> Storable<D> for Record<D> where
    D: Debug + Serialize + DeserializeOwned + Sized + Clone + Send + Sync + 'static
{
}
