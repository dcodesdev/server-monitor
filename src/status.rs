use crate::{
    bot::{notify, NotifyOpts},
    db::{endpoint::Status, url::Url, Db},
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

    for endpoint in endpoints.iter() {
        let emoji = match endpoint.status {
            Status::Up => "✅",
            Status::Down => "❌",
            Status::Pending => "🕒",
        };

        message.push_str(&format!(
            "URL: {}\nStatus: {} {:?}\n",
            endpoint.url.strip_prefix(),
            emoji,
            endpoint.status
        ));

        if let Some(uptime_at) = endpoint.uptime_at {
            let now = Local::now().naive_local();
            let duration = now.signed_duration_since(uptime_at);
            let days = duration.num_days();
            let hours = duration.num_hours() % 24;

            message.push_str(&format!("Uptime: {:?} days and {:?} hours\n", days, hours));
        }

        let max_latency = endpoint.max_latency;

        if let Some(max_latency) = max_latency {
            message.push_str(&format!("Max latency: {}ms\n", max_latency));
        }

        message.push_str("\n");

        db.endpoint.reset_max_latency(endpoint.url.as_str()).await?;
    }

    message.push_str("\n");

    Ok(message)
}

async fn incidents_update_message(db: &Arc<Db>) -> anyhow::Result<(String, Vec<String>)> {
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
        } else {
            message.push_str("\n\n");
        }
    }

    let ids = incidents
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();

    Ok((message, ids))
}

pub async fn create_server_update_cron(db: Arc<Db>, bot: Arc<Bot>) -> anyhow::Result<()> {
    let interval = db.metadata.interval().await?;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(interval)).await;
        loop {
            let result = server_update(&db, &bot).await;

            match result {
                Ok(_) => {}
                Err(e) => eprintln!("Server update Error: {}", e),
            }

            tokio::time::sleep(Duration::from_millis(UPDATE_INTERVAL)).await;
        }
    });

    Ok(())
}

async fn server_update(db: &Arc<Db>, bot: &Arc<Bot>) -> anyhow::Result<()> {
    let status_message = server_update_message(&db).await?;
    let (incidents_message, ids) = incidents_update_message(&db).await?;
    let message = format!("{}{}", incidents_message, status_message);
    let incidents: Vec<_> = ids.iter().map(|id| id.as_ref()).collect();

    notify(&NotifyOpts { bot: &bot, message }).await?;
    db.metadata.update_last_sent_at().await?;
    db.incident.delete_many(incidents).await?;

    Ok(())
}

pub async fn check_url_status(url: &Url, bot: &Bot, db: &Arc<Db>) -> anyhow::Result<()> {
    let result = tokio::join!(url_lookup(url, db), db.endpoint.get(url.as_str()));

    let is_success = result.0?;
    let endpoint = result.1?;

    if is_success {
        if endpoint.status != Status::Up {
            db.set_status_up(url.as_str()).await?;

            if endpoint.status == Status::Down {
                notify(&NotifyOpts {
                    message: format!("✅ {} is up again!", url.as_str()),
                    bot,
                })
                .await?;
            }
        }
    } else {
        if endpoint.status != Status::Down {
            notify(&NotifyOpts {
                message: format!("❌ {} is down!", url.as_str()),
                bot,
            })
            .await?;
            db.set_status_down(url).await?;
        }
    }

    Ok(())
}
