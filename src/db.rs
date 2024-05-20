use chrono::{DateTime, Local, Utc};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Db {
    pub endpoints: HashMap<String, Endpoint>,
    pub incidents: Vec<Incident>,
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct Incident {
    pub message: String,
    pub created_at: DateTime<Utc>,
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
            incidents: Vec::new(),
        }
    }

    pub fn set_status_up(&mut self, url: &str) {
        self.endpoints
            .insert(url.to_string(), Endpoint { status: Status::Up });
    }

    pub fn set_status_down(&mut self, url: &str) {
        self.endpoints.insert(
            url.to_string(),
            Endpoint {
                status: Status::Down,
            },
        );

        self.incidents.push(Incident {
            message: format!("{} Was down!", url),
            created_at: Local::now().to_utc(),
        })
    }

    pub fn get(&mut self, url: &str) -> Endpoint {
        let endpoint = self.endpoints.get(url).cloned();

        let res = endpoint.unwrap_or_else(|| {
            let endpoint = Endpoint {
                status: Status::Pending,
            };

            self.endpoints.insert(url.to_string(), endpoint.clone());

            endpoint
        });

        res
    }
}
