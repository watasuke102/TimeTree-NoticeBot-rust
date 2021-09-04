use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
struct Settings {
    discord_token: String,
    channel_id: String,
    timetree_key: String,
    timetree_id: String,
}

#[tokio::main]
async fn send_message(settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    println!("[info] Sending message...");
    let client = reqwest::Client::new();
    let resp = client
        .post(format!(
            "https://discord.com/api/channels/{}/messages",
            settings.channel_id
        ))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {}", settings.discord_token))
        .body(
            r#"{
  "content": "おはようございます。2021/09/04の予定をお知らせします。",
  "tts": false,
  "embeds": [{
    "title": "今日の予定",
    "description": "2件の予定があります。",
    "color": 3067015,
    "fields": [
        {
            "name": "予定その1",
            "value": "00:00～00:00"
        },
        {
            "name": "予定その2",
            "value": "00:00～23:59"
        }
    ]
  }]
}"#,
        )
        .send()
        .await?;
    println!("[info] Sending was finished");
    Ok(())
}

fn main() {
    let file = File::open("env.json")
        .expect("cannot read `env.json`: did you create this file? try `cp sample-env.json env.json` and edit it.");
    let settings: Settings = serde_json::from_reader(BufReader::new(file)).unwrap();
    println!("{}", settings.discord_token);
    match send_message(settings) {
        Err(why) => println!("[ERR] {:?}", why),
        _ => (),
    }
}
