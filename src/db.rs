use chrono::{Local, NaiveDateTime};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Db {
    endpoints: HashMap<String, Endpoint>,
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub url: String,
    pub status: Status,
    pub last_check: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Up,
    Down,
    Pending,
}

impl Db {
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
        }
    }

    pub fn set_status_up(&mut self, url: &str) {
        self.endpoints.insert(
            url.to_string(),
            Endpoint {
                url: url.to_string(),
                last_check: self.now(),
                status: Status::Up,
            },
        );
    }

    pub fn set_status_down(&mut self, url: &str) {
        self.endpoints.insert(
            url.to_string(),
            Endpoint {
                url: url.to_string(),
                last_check: self.now(),
                status: Status::Down,
            },
        );
    }

    pub fn get(&mut self, url: &str) -> Endpoint {
        let endpoint = self.endpoints.get(url).cloned();

        let res = endpoint.unwrap_or_else(|| {
            let endpoint = Endpoint {
                url: url.to_string(),
                last_check: self.now(),
                status: Status::Pending,
            };

            self.endpoints.insert(url.to_string(), endpoint.clone());

            endpoint
        });

        res
    }

    fn now(&self) -> NaiveDateTime {
        Local::now().naive_local()
    }
}
