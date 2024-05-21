use crate::{
    bot::{notify, NotifyOpts},
    db::{Db, Status},
    request::url_lookup,
};
use chrono::Local;
use std::{sync::Arc, time::Duration};
use teloxide::Bot;
use tokio::sync::Mutex;

/// Gets the incidents from the db and creates a Telegram message and returns the String
async fn server_update_message(db: &Mutex<Db>) -> String {
    let mut message = String::from("Server status:\n\n");
    let db = db.lock().await;

    let all_up = db.incidents.is_empty()
        && db
            .endpoints
            .values()
            .all(|value| value.status == Status::Up);

    if all_up {
        message.push_str("✅ No new incidents have happened so far.\n\n");
    }

    for (url, value) in &db.endpoints {
        let emoji = match value.status {
            Status::Up => "✅",
            Status::Down => "❌",
            Status::Pending => "🕒",
        };

        message.push_str(&format!(
            "URL: {}\nStatus: {} {:?}\n",
            url, emoji, value.status
        ));

        let uptime = match value.uptime_at {
            Some(uptime_at) => {
                let now = Local::now();
                let duration = now.signed_duration_since(uptime_at);
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;

                format!("{:?} days and {:?} hours", days, hours)
            }
            None => "Uptime: N/A".to_string(),
        };

        message.push_str(&format!("Up for: {}\n\n", uptime));
    }

    message
}

async fn incidents_update_message(db: &Mutex<Db>) -> String {
    let db = db.lock().await;
    let mut message = String::new();

    if !db.incidents.is_empty() {
        message.push_str("Incidents:\n\n");
    }

    for (i, incident) in db.incidents.iter().enumerate() {
        let is_last = i == db.incidents.len() - 1;
        let time = incident.created_at.format("%d/%m/%Y %I:%M %p").to_string();
        message.push_str(&format!("Message: {}\nTime: {}\n", incident.message, time));

        if !is_last {
            message.push_str("\n");
        }
    }

    message
}

pub fn server_update_cron(db: Arc<Mutex<Db>>, interval: u64, bot: Arc<Bot>) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(interval)).await;
        loop {
            let status_message = server_update_message(&db).await;
            let incidents_message = incidents_update_message(&db).await;
            let message = format!("{}{}", status_message, incidents_message);

            if let Err(err) = notify(&NotifyOpts { bot: &bot, message }).await {
                eprintln!("Error sending notification: {}", err);
            }

            tokio::time::sleep(Duration::from_millis(interval)).await;
        }
    });
}

pub async fn check_url_status(url: &str, bot: &Bot, db: &Arc<Mutex<Db>>) -> anyhow::Result<()> {
    let is_success = url_lookup(url).await?;
    let mut db = db.lock().await;
    let endpoint = db.get(url);

    if is_success {
        if endpoint.status == Status::Down {
            notify(&NotifyOpts {
                message: format!("✅ {} is up again!", url),
                bot,
            })
            .await?;
        }
        db.set_status_up(url);
    } else {
        if endpoint.status != Status::Down {
            notify(&NotifyOpts {
                message: format!("❌ {} is down!", url),
                bot,
            })
            .await?;
        }
        db.set_status_down(url);
    }

    Ok(())
}
