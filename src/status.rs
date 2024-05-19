// This module sends a message every 24 hrs
// about the state of the servers

use std::sync::Arc;

use crate::db::Db;

/// Gets the incidents from the db and creates
/// a Telegram message and returns the String
pub fn server_update(db: &Arc<Db>) -> String {
    let mut message = String::new();

    db.incidents.iter().enumerate().for_each(|(i, incident)| {
        let is_last = (db.incidents.len() - 1) == i;
        message.push_str(incident.message);

        if !is_last {
            message.push_str("\n");
        }
    });

    message
}
