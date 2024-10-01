use reqwest;
use serde_json::Value;
use std::cmp::Ordering;
use std::error::Error;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let url = "https://api.bitget.com/api/mix/v1/market/contracts?productType=umcbl";

    let response = client.get(url).send().await?;
    let data: Value = response.json().await?;

    let mut futures: Vec<String> = data["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| {
            let symbol = item["symbol"].as_str().unwrap();
            if symbol.ends_with("USDT_UMCBL") {
                Some(format!(
                    "BITGET:{}USDT.P",
                    symbol.trim_end_matches("USDT_UMCBL")
                ))
            } else {
                None
            }
        })
        .collect();

    futures.sort_by(|a, b| {
        let a_parts: Vec<&str> = a.split(":").collect();
        let b_parts: Vec<&str> = b.split(":").collect();

        let a_symbol = a_parts[1].replace("USDT.P", "");
        let b_symbol = b_parts[1].replace("USDT.P", "");

        let a_numeric = a_symbol
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();
        let b_numeric = b_symbol
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();

        match (a_numeric.parse::<i32>(), b_numeric.parse::<i32>()) {
            (Ok(a_num), Ok(b_num)) => match b_num.cmp(&a_num) {
                Ordering::Equal => a_symbol.cmp(&b_symbol),
                other => other,
            },
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Err(_), Err(_)) => a_symbol.cmp(&b_symbol),
        }
    });

    let mut file = File::create("bitget_usdt_perpetual_futures.txt")?;
    for future in futures {
        writeln!(file, "{}", future)?;
    }

    println!("İşlem tamamlandı. Sonuçlar 'bitget_usdt_perpetual_futures.txt' dosyasına yazıldı.");
    Ok(())
}
