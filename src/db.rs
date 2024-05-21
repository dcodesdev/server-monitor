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
    pub uptime_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Incident {
    pub url: String,
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
        self.endpoints.insert(
            url.to_string(),
            Endpoint {
                status: Status::Up,
                uptime_at: Some(Local::now().to_utc()),
            },
        );

        self.incidents.retain(|incident| incident.url != url);
    }

    pub fn set_status_down(&mut self, url: &str) {
        self.endpoints.insert(
            url.to_string(),
            Endpoint {
                status: Status::Down,
                uptime_at: None,
            },
        );

        self.incidents.push(Incident {
            url: url.to_string(),
            message: format!("{} Was down!", url),
            created_at: Local::now().to_utc(),
        })
    }

    pub fn get(&mut self, url: &str) -> Endpoint {
        let endpoint = self.endpoints.get(url).cloned();

        let res = endpoint.unwrap_or_else(|| {
            let endpoint = Endpoint {
                status: Status::Pending,
                uptime_at: None,
            };

            self.endpoints.insert(url.to_string(), endpoint.clone());

            endpoint
        });

        res
    }
}
