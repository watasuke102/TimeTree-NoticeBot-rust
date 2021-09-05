// TimeTree-NoticeBot-rust
// main.rs
//
// CopyRight (c) 2021 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT SUSHI-WARE License.
use chrono::{DateTime, Duration, FixedOffset, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::BufReader;
use std::ops::Add;
use std::{thread, time};

#[derive(Serialize, Deserialize)]
struct Settings {
    discord_token: String,
    channel_id: String,
    timetree_key: String,
    timetree_id: String,
    disable_everyone: bool,
    silent_mode: bool,
}

#[derive(Debug)]
enum LogType {
    Info,
    Error,
}
fn log(settings: &Settings, log_type: LogType, body: &str) {
    if !settings.silent_mode {
        let body = format!("[{:?}] {}", log_type, body);
        match log_type {
            LogType::Info => println!("{}", body),
            LogType::Error => eprintln!("{}", body),
        }
    }
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

#[derive(Debug)]
struct Event {
    title: String,
    all_day: bool,
    start_at: DateTime<FixedOffset>,
    end_at: DateTime<FixedOffset>,
}

#[tokio::main]
async fn send_message(
    settings: &Settings,
    title: String,
    embeds: Embed,
) -> Result<(), Box<dyn std::error::Error>> {
    log(
        settings,
        LogType::Info,
        &format!("Sending message '{}'", title),
    );
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
                "content": format!("{}{}",
                    if settings.disable_everyone{""} else {"@everyone\n"},
                    title
                ),
                "tts": false,
                "embeds": [embeds]
            })
            .to_string(),
        )
        .send()
        .await?;
    log(settings, LogType::Info, "Sending was finished");
    Ok(())
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
        attributes: EventFromApi,
    }
    #[derive(Debug, Serialize, Deserialize)]
    struct EventFromApi {
        title: String,
        all_day: bool,
        start_at: String,
        end_at: String,
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
    let jst = FixedOffset::east(9 * 3600);
    for item in resp.data {
        let item = item.attributes;
        events.push(Event {
            title: item.title,
            all_day: item.all_day,
            start_at: if let Ok(start_datetime) = &item.start_at.parse::<DateTime<Utc>>() {
                start_datetime.with_timezone(&jst)
            } else {
                Utc::now().with_timezone(&jst)
            },
            end_at: if let Ok(end_datetime) = &item.end_at.parse::<DateTime<Utc>>() {
                end_datetime.with_timezone(&jst)
            } else {
                Utc::now().with_timezone(&jst)
            },
        });
    }
    Ok(events)
}

fn create_eventlist_embeds(events: &Vec<Event>) -> Embed {
    let mut result = Embed {
        title: "今日の予定".to_string(),
        description: format!("今日の予定は{}件です。", events.len()).to_string(),
        color: 0x2ecc87, // TimeTree logo color
        fields: Vec::<Field>::new(),
    };

    for e in events.iter() {
        let mut time = "終日".to_string();
        if !e.all_day {
            // 開始日と終了日が一致する場合は日付を表示しない
            if e.start_at.date() == e.end_at.date() {
                time = format!(
                    "{}～{}",
                    e.start_at.format("%H:%M"),
                    e.end_at.format("%H:%M")
                )
                .to_string();
            } else {
                time = format!(
                    "{}～{}",
                    e.start_at.format("%m/%d %H:%M"),
                    e.end_at.format("%m/%d %H:%M")
                )
                .to_string();
            }
        }

        result.fields.push(Field {
            name: format!("{}", e.title).to_string(),
            value: time,
        });
    }
    result
}

fn check_event_after_10min(settings: &Settings, events: &Vec<Event>) {
    if events.len() == 0 {
        return;
    }
    let mut embed = Embed {
        title: "まもなく開始".to_string(),
        description: "".to_string(),
        color: 0x2ecc87, // TimeTree logo color
        fields: Vec::<Field>::new(),
    };

    let now = Utc::now()
        .with_timezone(&FixedOffset::east(9 * 3600))
        .time()
        .add(Duration::minutes(10));
    for e in events.iter() {
        if now.hour() == e.start_at.time().hour() && now.minute() == e.start_at.time().minute() {
            embed.fields.push(Field {
                name: format!("{}", e.title).to_string(),
                value: format!("{}～", e.start_at.format("%H:%M")).to_string(),
            });
        }
    }

    if embed.fields.len() != 0 {
        if let Err(why) = send_message(
            settings,
            "10分後に以下の予定があります。".to_string(),
            embed,
        ) {
            log(&settings, LogType::Error, &format!("{:?}", why));
        }
    }
}

fn main() {
    let file = File::open("env.json")
        .expect("cannot read `env.json`: did you create this file? try `cp sample-env.json env.json` and edit it.");
    let settings: Settings = serde_json::from_reader(BufReader::new(file)).unwrap();
    let mut events = Vec::<Event>::new();

    log(&settings, LogType::Info, "Bot is running...");
    loop {
        let now = Utc::now().with_timezone(&FixedOffset::east(9 * 3600));
        log(
            &settings,
            LogType::Info,
            &format!("{}: checking events", now.format("%Y/%m/%d %H:%M:%S")),
        );
        match fetch_timetree_event(&settings) {
            Err(why) => log(&settings, LogType::Error, &format!("{:?}", why)),
            Ok(e) => events = e,
        }
        if now.time().hour() == 8 && now.time().minute() == 0 {
            if let Err(why) = send_message(
                &settings,
                format!(
                    "おはようございます。{}の予定をお知らせします。",
                    Local::now().format("%Y/%m/%d")
                ),
                create_eventlist_embeds(&events),
            ) {
                log(&settings, LogType::Error, &format!("{:?}", why));
            }
        }
        check_event_after_10min(&settings, &events);
        thread::sleep(time::Duration::from_millis(1000 * 60));
    }
}
