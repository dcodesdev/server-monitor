// This module sends a message every 24 hrs
// about the state of the servers

use std::{sync::Arc, time::Duration};
use teloxide::Bot;
use tokio::sync::Mutex;

use crate::{
    bot::{notify, NotifyOpts},
    db::{Db, Status},
    request::url_lookup,
};

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

pub fn server_update_cron(db: &Arc<Mutex<Db>>, interval: u64, bot: &Arc<Bot>) {
    let db = db.clone();
    let bot = bot.clone();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(interval)).await;
        loop {
            let message = server_update(&db).await;
            let result = notify(&NotifyOpts { bot: &bot, message }).await;

            match result {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error sending notification: {}", err)
                }
            }

            tokio::time::sleep(Duration::from_millis(interval)).await;
        }
    });
}

pub async fn check_url(url: &str, bot: &Bot, db: &Arc<Mutex<Db>>) -> anyhow::Result<()> {
    let is_success = url_lookup(url).await?;
    let mut db = db.lock().await;
    let endpoint = db.get(url);

    if is_success {
        // If the previous one was "Down" and now it is "Up", then notify it is up again.
        if endpoint.status == Status::Down {
            notify(&NotifyOpts {
                message: format!("✅ {} is up again!", url),
                bot,
            })
            .await?;
        }

        db.set_status_up(url);
        return Ok(());
    }

    // If it is already set as Down, then don't notify.
    if endpoint.status != Status::Down {
        notify(&NotifyOpts {
            message: format!("❌ {} is down!", url),
            bot,
        })
        .await?;
    }

    db.set_status_down(url);

    Ok(())
}
