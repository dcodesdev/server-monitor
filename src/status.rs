use crate::{
    bot::{notify, NotifyOpts},
    db::{Db, Status},
    request::url_lookup,
    UPDATE_INTERVAL,
};
use chrono::Local;
use std::{sync::Arc, time::Duration};
use teloxide::Bot;

/// Gets the incidents from the db and creates a Telegram message and returns the String
async fn server_update_message(db: &Db) -> anyhow::Result<String> {
    let mut message = String::from("Server status:\n\n");

    let endpoints = db.endpoint.get_all().await?;

    let all_up =
        db.incident.is_empty().await? && endpoints.iter().all(|value| value.status == Status::Up);

    if all_up {
        message.push_str("✅ No new incidents have happened so far.\n\n");
    }

    endpoints.iter().for_each(|value| {
        let emoji = match value.status {
            Status::Up => "✅",
            Status::Down => "❌",
            Status::Pending => "🕒",
        };

        message.push_str(&format!(
            "URL: {}\nStatus: {} {:?}\n",
            value.url, emoji, value.status
        ));

        let uptime = match value.uptime_at {
            Some(uptime_at) => {
                let now = Local::now().naive_local();
                let duration = now.signed_duration_since(uptime_at);
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;

                format!("{:?} days and {:?} hours", days, hours)
            }
            None => "Uptime: N/A".to_string(),
        };

        message.push_str(&format!("Up for: {}\n\n", uptime));
    });

    Ok(message)
}

async fn incidents_update_message(db: &Arc<Db>) -> anyhow::Result<String> {
    let mut message = String::new();

    if !db.incident.is_empty().await? {
        message.push_str("Incidents:\n\n");
    }

    let incidents = db.incident.get_all().await?;
    for (i, incident) in incidents.iter().enumerate() {
        let is_last = i == incidents.len() - 1;
        let time = incident.created_at.format("%d/%m/%Y %I:%M %p").to_string();
        message.push_str(&format!("Message: {}\nTime: {}\n", incident.message, time));

        if !is_last {
            message.push_str("\n");
        }
    }

    Ok(message)
}

pub fn server_update_cron(db: Arc<Db>, bot: Arc<Bot>) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(UPDATE_INTERVAL)).await;
        loop {
            let status_message = match server_update_message(&db).await {
                Ok(msg) => msg,
                Err(err) => {
                    eprintln!("Error getting status message: {}", err);
                    continue;
                }
            };
            let incidents_message = match incidents_update_message(&db).await {
                Ok(msg) => msg,
                Err(err) => {
                    eprintln!("Error getting incidents message: {}", err);
                    continue;
                }
            };
            let message = format!("{}{}", status_message, incidents_message);

            if let Err(err) = notify(&NotifyOpts { bot: &bot, message }).await {
                eprintln!("Error sending notification: {}", err);
            }

            tokio::time::sleep(Duration::from_millis(UPDATE_INTERVAL)).await;
        }
    });
}

pub async fn check_url_status(url: &str, bot: &Bot, db: &Arc<Db>) -> anyhow::Result<()> {
    let records = sqlx::query!("SELECT * FROM endpoint;")
        .fetch_all(&db.pool)
        .await?;

    println!("{:#?}", records);

    let result = tokio::join!(url_lookup(url), db.get(url));

    let is_success = result.0?;
    let endpoint = result.1?;

    if is_success {
        if endpoint.status != Status::Up {
            db.set_status_up(url).await?;

            if endpoint.status == Status::Down {
                notify(&NotifyOpts {
                    message: format!("✅ {} is up again!", url),
                    bot,
                })
                .await?;
            }
        }
    } else {
        if endpoint.status != Status::Down {
            notify(&NotifyOpts {
                message: format!("❌ {} is down!", url),
                bot,
            })
            .await?;
            db.set_status_down(url).await?;
        }
    }

    Ok(())
}
