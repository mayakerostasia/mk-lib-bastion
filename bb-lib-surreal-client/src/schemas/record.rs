use crate::{error::SurrealClientError, storable::DBThings, Storable};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Id;

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

// impl DeserializeOwned: for<'de> Deserialize<'de> {

// }

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

impl<D: DBThings + Send> DBThings for Record<D> {}

impl<D> Storable<D> for Record<D> where D: DBThings + Send + 'static {}
