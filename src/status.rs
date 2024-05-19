// This module sends a message every 24 hrs
// about the state of the servers

use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use crate::db::Db;

/// Gets the incidents from the db and creates
/// a Telegram message and returns the String
async fn server_update(db: &Mutex<Db>) -> String {
    let mut message = String::new();

    let db = db.lock().await;
    if db.incidents.len() == 0 {
        return "✅ No incidents have happened so far.".to_string();
    }

    db.incidents.iter().enumerate().for_each(|(i, incident)| {
        let is_last = (db.incidents.len() - 1) == i;
        let time = format!("{}", incident.created_at.format("%d/%m/%Y %H:%M"));
        message.push_str(incident.message);
        message.push_str(" | ");
        message.push_str(&time);

        if !is_last {
            message.push_str("\n--------------------");
        }
    });

    message
}

pub fn server_update_cron(db: &Arc<Mutex<Db>>, interval: u64) {
    let db = db.clone();
    tokio::spawn(async move {
        loop {
            let message = server_update(&db).await;
            println!("Message: {}", message);
            // TODO: implement

            tokio::time::sleep(Duration::from_millis(interval)).await;
        }
    });
}
