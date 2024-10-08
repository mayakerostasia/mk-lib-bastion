use crate::error::SurrealClientError;
use crate::Error;
use crate::{create_record, delete_record, select, update_record};
use bb_lib_nats_streams::{Decoder, Encoder};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use surrealdb::{RecordId, RecordIdKey};
use tracing::error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record<T>
where
    T: Debug + Serialize + Clone + From<T>,
{
    #[serde(skip)]
    _id: Option<RecordId>,
    _data: Option<T>,
}

impl<T> Encoder for Record<T> where T: Debug + Serialize + Clone {}
impl<'a, T> Decoder<'a, Record<T>> for Record<T> where T: Debug + Serialize + Clone {}

impl<T> From<Record<T>> for Bytes
where
    T: Debug + Serialize + Clone,
{
    fn from(value: Record<T>) -> Self {
        value.encode().expect("Failed to encode Bytes").into()
    }
}

impl<T> From<Bytes> for Record<T>
where
    T: Debug + Serialize + Clone + for<'a> Deserialize<'a>,
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

impl<T> Record<T>
where
    T: Debug + Serialize + Clone + 'static + for<'a> Deserialize<'a> + Into<Record<T>>,
{
    pub async fn save(&self) -> Result<Record<T>, Error> {
        let record: Record<T> = self.clone();
        create_record(record).await
    }

    pub async fn select(&mut self) -> Result<Record<T>, Error> {
        select(self).await
    }

    pub async fn update(&mut self) -> Result<Record<T>, Error> {
        update_record(self.clone()).await
    }

    pub async fn delete(self) -> Result<Option<Record<T>>, Error> {
        delete_record(self).await
    }

    pub fn new(tb: &str, id: Option<RecordIdKey>, data: Option<T>) -> Self {
        if let Some(_id) = id {
            Self {
                _id: Some(RecordId::from_table_key(tb, _id)),
                _data: data,
            }
        } else {
            Self {
                _id: None,
                _data: data,
            }
        }
    }

    pub fn set_data(&mut self, data: &T) -> Result<(), SurrealClientError> {
        self._data = Some(data.clone());
        Ok(())
    }

    pub fn id(&self) -> Result<RecordId, SurrealClientError> {
        match &self._id {
            Some(id) => Ok(id.clone()),
            None => Err(SurrealClientError::NoID),
        }
    }

    pub fn data(&self) -> T {
        self._data.clone().expect("No Data!")
    }
}
