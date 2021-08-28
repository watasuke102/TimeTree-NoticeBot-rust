use serde_json::Value;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let file = File::open("env.json")
        .expect("cannot read `env.json`: did you create this file? try `cp sample-env.json env.json` and edit it.");
    let setting: Value = serde_json::from_reader(BufReader::new(file)).unwrap();
    println!("{}", setting["discord_token"]);
}
