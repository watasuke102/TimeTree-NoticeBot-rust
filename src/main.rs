use chrono::{DateTime, FixedOffset, Utc};
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

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    title: String,
    all_day: bool,
    start_at: String,
    end_at: String,
}

#[tokio::main]
async fn fetch_timetree_event(
    settings: &Settings,
) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
    #[derive(Debug, Serialize, Deserialize)]
    struct TimeTreeEventList {
        data: Vec<TimeTreeAttributes>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    struct TimeTreeAttributes {
        attributes: Event,
    }
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "https://timetreeapis.com/calendars/{}/upcoming_events?timezone=Asia/Tokyo",
            settings.timetree_id
        ))
        .header("Authorization", format!("Bearer {}", settings.timetree_key))
        .header("Accept", "application/vnd.timetree.v1+json")
        .send()
        .await?
        .json::<TimeTreeEventList>()
        .await?;
    let mut events: Vec<Event> = Vec::new();
    for item in resp.data {
        events.push(item.attributes);
    }
    Ok(events)
}
#[derive(Debug, Serialize, Deserialize)]
struct Embed {
    title: String,
    description: String,
    color: u32,
    fields: Vec<Field>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Field {
    name: String,
    value: String,
}

fn create_embeds(events: &Vec<Event>) -> Embed {
    let mut result = Embed {
        title: "今日の予定".to_string(),
        description: format!("今日の予定は{}件です。", events.len()).to_string(),
        color: 0x2ecc87, // TimeTree logo color
        fields: Vec::<Field>::new(),
    };

    for e in events.iter() {
        let mut time = "終日".to_string();
        if !e.all_day {
            if let Ok(start_datetime) = &e.start_at.parse::<DateTime<Utc>>() {
                if let Ok(end_datetime) = &e.end_at.parse::<DateTime<Utc>>() {
                    let start_datetime = start_datetime.with_timezone(&FixedOffset::east(9 * 3600));
                    let end_datetime = end_datetime.with_timezone(&FixedOffset::east(9 * 3600));

                    // 開始日と終了日が一致する場合は日付を表示しない
                    if start_datetime.date() == end_datetime.date() {
                        time = format!(
                            "{}～{}",
                            start_datetime.format("%H:%M"),
                            end_datetime.format("%H:%M")
                        )
                        .to_string();
                    } else {
                        time = format!(
                            "{}～{}",
                            start_datetime.format("%m/%d %H:%M"),
                            end_datetime.format("%m/%d %H:%M")
                        )
                        .to_string();
                    }
                }
            }
        }

        result.fields.push(Field {
            name: format!("{}", e.title).to_string(),
            value: time,
        });
    }
    result
}

#[tokio::main]
async fn send_message(
    settings: &Settings,
    embeds: Embed,
) -> Result<(), Box<dyn std::error::Error>> {
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
                "embeds": [embeds]
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

    let mut events = Vec::<Event>::new();
    match fetch_timetree_event(&settings) {
        Err(why) => println!("[ERR] {:?}", why),
        Ok(e) => events = e,
    }
    match send_message(&settings, create_embeds(&events)) {
        Err(why) => println!("[ERR] {:?}", why),
        _ => (),
    }
}
