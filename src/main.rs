use chrono::NaiveDateTime;
use home_dir::*;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
struct Data {
    id: String,
    text: String,
    #[serde(default)]
    date: String,
}

#[derive(Debug, Deserialize)]
struct Hoyoyo {
    success: bool,
    data: Vec<Data>,
}

#[derive(Debug, Deserialize)]
struct Hawawa {
    success: bool,
}

#[derive(Debug, Deserialize)]
struct Api {
    user: String,
    key: String,
}

fn key_check() -> Api {
    let path = Path::new("~/.config/habitica/key.json")
        .expand_home()
        .unwrap();
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);
    let deserialized: Api = serde_json::from_reader(reader).unwrap();
    //println!("{:?}", deserialized);
    deserialized
}

fn print_todos(dat: &Hoyoyo) {
    if !dat.success {
        println!("fail");
        return;
    }
    for (cnt, d) in (0_i32..).zip(dat.data.iter()) {
        let mut simekiri = "";
        if !d.date.is_empty() {
            let a: Vec<&str> = d.date.split('T').collect();
            simekiri = a[0];
        }
        println!("[{}] {} {}", cnt, d.text, simekiri);
    }
}

async fn get_todos(api: &Api) -> Result<()> {
    let client = Client::new();
    let url = "https://habitica.com/api/v3/tasks/user?type=todos";
    let response = client
        .get(url)
        .header("x-api-user", &api.user)
        .header("x-api-key", &api.key)
        .send()
        .await?;
    let body = response.json::<Hoyoyo>().await?;
    print_todos(&body);
    //println!("{:?}", body);
    Ok(())
}

async fn done_todos(num: usize, api: &Api) -> Result<()> {
    let client = Client::new();
    let url = "https://habitica.com/api/v3/tasks/user?type=todos";
    let response = client
        .get(url)
        .header("x-api-user", &api.user)
        .header("x-api-key", &api.key)
        .send()
        .await?;
    let body = response.json::<Hoyoyo>().await?;

    if body.data.len() > num {
        let a = &body.data[num];
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        let client = Client::new();
        let url = format!("https://habitica.com/api/v3/tasks/{}/score/up", a.id);
        println!("{}", url);
        let response = client
            .post(url)
            .header("x-api-user", &api.user)
            .header("x-api-key", &api.key)
            .header("Content-length", 0)
            .send()
            .await?;
        let body = response.json::<Hawawa>().await?;
        if body.success {
            println!("success");
        } else {
            println!("fail");
        }
    }
    Ok(())
}

async fn add_todos(task: String, mut date: String, api: &Api) -> Result<()> {
    let client = Client::new();
    let url = "https://habitica.com/api/v3/tasks/user";

    let mut map = HashMap::new();
    map.insert("text", task);
    map.insert("type", "todo".to_string());
    date.push_str("T00:00:00.000Z");
    let date_fm = NaiveDateTime::parse_from_str(&date, "%Y-%m-%dT%H:%M:%S.000Z");
    let _ = match date_fm {
        Ok(_) => 1,
        Err(err) => return Err(Box::new(err)),
    };
    map.insert("date", date);

    let response = client
        .post(url)
        .header("x-api-user", &api.user)
        .header("x-api-key", &api.key)
        .json(&map)
        .send()
        .await?;
    let body = response.json::<Hawawa>().await?;
    if body.success {
        println!("success");
    } else {
        println!("fail");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let api = key_check();
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        get_todos(&api).await?;
    } else {
        let query = &args[1];
        if query == "add" {
            let text = &args[2];
            let date = &args[3];
            //to do date format
            add_todos(text.to_string(), date.to_string(), &api).await?;
        } else if query == "done" {
            let num: usize = args[2].parse().unwrap();
            done_todos(num, &api).await?;
        }
    }
    Ok(())
}
