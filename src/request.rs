use crate::{
    db::{Db, Status},
    notify::{self, NotifyOpts},
};
use std::sync::Arc;
use teloxide::Bot;
use tokio::sync::Mutex;

pub async fn check_status<'a>(url: &str, bot: &'a Bot, db: Arc<Mutex<Db>>) -> anyhow::Result<()> {
    let response = reqwest::get(url).await?;
    let status = response.status();

    let mut db = db.lock().await;
    let endpoint = db.get(url);

    if status.is_success() || status == 429 {
        // If the previous one was "Down" and now it is "Up", then notify it is up again.
        if endpoint.status == Status::Down {
            notify::notify(&NotifyOpts {
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
        notify::notify(&NotifyOpts {
            message: format!("❌ {} is down!", url),
            bot,
        })
        .await?;
    }

    db.set_status_down(url);

    Ok(())
}
