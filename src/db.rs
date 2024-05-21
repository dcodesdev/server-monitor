use chrono::{DateTime, Utc};
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
                uptime_at: Some(Utc::now()),
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
            message: format!("{} was down!", url),
            created_at: Utc::now(),
        })
    }

    pub fn get(&mut self, url: &str) -> Endpoint {
        self.endpoints.get(url).cloned().unwrap_or_else(|| {
            let endpoint = Endpoint {
                status: Status::Pending,
                uptime_at: None,
            };

            self.endpoints.insert(url.to_string(), endpoint.clone());
            endpoint
        })
    }
}
