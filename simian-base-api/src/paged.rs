use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Add};

fn default_start_at() -> usize {
    0_usize
}
fn default_total() -> usize {
    0_usize
}
fn default_abs_limit() -> usize {
    100_usize
}
fn default_page_size() -> usize {
    10_usize
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paged {
    #[serde(alias = "total", alias = "total_count", default = "default_total")]
    pub total: usize,
    #[serde(alias = "startAt", alias = "from", default = "default_start_at")]
    pub offset: usize,
    #[serde(alias = "maxResults", alias = "to", default = "default_page_size")]
    pub page_size: usize,
    #[serde(default = "default_abs_limit")]
    pub abs_limit: usize,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Add for Paged {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new = self.clone();
        new.offset = other.offset + self.page_size;
        new
    }
}

impl Iterator for Paged {
    type Item = Paged;

    fn next(&mut self) -> Option<Self> {
        if self.offset >= self.total || self.offset >= self.abs_limit {
            None
        } else {
            let new = self.offset + self.page_size;
            Some(Paged {
                total: self.total,
                offset: new,
                page_size: self.page_size,
                abs_limit: self.abs_limit,
                extra: self.extra.clone(),
            })
        }
    }
}

impl Default for Paged {
    fn default() -> Paged {
        Paged {
            abs_limit: 100_usize,
            page_size: 10_usize,
            offset: 0_usize,
            total: 0_usize,
            extra: HashMap::new(),
        }
    }
}

impl Paged {
    pub fn set_limit(&mut self, limit: Option<usize>) {
        match limit {
            Some(l) => self.abs_limit = l,
            None => self.abs_limit = 100_usize,
        }
    }

    pub fn get_limit(&self) -> usize {
        self.abs_limit
    }

    pub fn set_page_size(&mut self, page_size: Option<usize>) {
        match page_size {
            Some(p) => self.page_size = p,
            None => self.page_size = 10_usize,
        }
    }
}
