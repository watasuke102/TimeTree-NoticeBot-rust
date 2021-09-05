# TimeTree-NoticeBot-Rust

## What's this
TimeTreeの予定を確認し、Discordに通知します
毎朝8:00(JST)に一日の予定を一覧表示、開始時間が指定されている予定は10分前に通知

## How to use
1. Discord Botアカウント及びTimeTreeパーソナルアクセストークンを作成する
1. `git clone https://github.com/watasuke102/TimeTree-NoticeBot-rust`
1. `cp sample-env.json env.json`
1. env.jsonを編集する
    - `discord_token`: Botトークン
    - `channel_id`: 発言させたいチャンネルのID
    - `timetree_key`: TimeTreeパーソナルアクセストークン
    - `timetree_id`: `https://timetreeapp.com/calendars/[id]` [id]の部分を記入

## LICENSE
MIT SUSHI-WARE LICENSE

(C) 2021 わたすけ

