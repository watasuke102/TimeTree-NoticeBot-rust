use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
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
    let date = Utc::now().format("%Y/%m/%d").to_string();

    let client = reqwest::Client::new();
    let _resp = client
        .post(format!(
            "https://discord.com/api/channels/{}/messages",
            settings.channel_id
        ))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {}", settings.discord_token))
        .body(
            json!({
                "content": format!("おはようございます。{}の予定をお知らせします。", date),
                "tts": false,
                "embeds": [{
                    "title": "今日の予定",
                    "description": "2件の予定があります。",
                    "color": 0x2ecc87, // TimeTree logo color #2ecc87
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
            })
            .to_string(),
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
    match send_message(settings) {
        Err(why) => println!("[ERR] {:?}", why),
        _ => (),
    }
}
