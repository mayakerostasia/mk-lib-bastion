use std::fmt::Display;

use futures_lite::StreamExt;
use anyhow::Error;
use bb_lib_surreal_client::{connect, live_select, setup, SurrealId};
use surrealdb::Notification;
use serde::{Serialize, Deserialize};
use serde_json::Value;
// Handle the result of the live query notification
//{
// 	id: event:[
// 		'2024-06-22T06:53:15.424Z'
// 	],
// 	issue_id: 'SOARALERTS-1722414',
// 	issuetype: NONE
// }

// Definition
#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Event {
    id: SurrealId,
    eventType: String,
    issue_id: String,
    info: Option<Value>,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {} @ {}",  self.issue_id, self.eventType, self.id.0.id)
    }
}


async fn handle_created(event: Event) -> Result<(), Error> {
    println!("Issue Created! -> {}", event);
    // let resp = query(format!("SELECT * FROM issue:`{}`", event.issue_id).as_str()).await?;
    // println!("DB Issue -> {:#?}", resp);
    Ok(())
}

async fn handle_comment(event: Event) -> Result<(), Error> {
    println!("New Comment! -> {}", event);
    Ok(())
}

async fn handle(result: Result<Notification<Event>, surrealdb::Error>) {
    match result {
        Ok(notification) => {
            let event = notification.data.clone();

            match event.eventType.as_ref() {
                "IssueCreated" => {
                    handle_created(event).await.expect("Couldn't handle_splunk");
                },
                "CommentCreated" => {
                    handle_comment(event).await.expect("Couldn't handle_comment");
                }
                _ => {
                    println!("{event:}");
                }
            }
        },
        Err(error) => eprintln!("{error}"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cfg = setup();
    connect(&cfg).await?;
    let query = "event";
    let mut stream = live_select(query, "test").await?;
    while let Some(event) = stream.next().await {
        handle(event).await
    }

    Ok(())
}
