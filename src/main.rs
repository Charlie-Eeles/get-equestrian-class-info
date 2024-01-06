use reqwest::header::{HeaderMap, ORIGIN};
use reqwest::{Client, Error as ReqError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use prettytable::{Table, Row, Cell, row};
use std::error::Error as stderr;
use csv::Writer;

#[derive(Debug)]
enum Error {
    ReqError(ReqError),
    Serialization(serde_json::Error),
}

impl From<ReqError> for Error {
    fn from(err: ReqError) -> Error {
        Error::ReqError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Serialization(err)
    }
}

#[derive(Deserialize, Debug)]
struct PrimaryQueryRes {
    trips: Vec<Trips>
}

#[derive(Deserialize, Debug)]
struct ClassQueryRes {
    classes: Vec<Classes>
}

#[derive(Deserialize, Debug)]
struct Trips {
    entry_id: u32,
    entry_number: u32,
    sponsor: String,
    horse: String,
    rider_id: u32,
    rider_name: String,
}

#[derive(Debug, Serialize)]
struct CombinedData {
    entry_number: u32,
    horse: String,
    rider_name: String,
    class_number: u32,
    class_name: String,
    sponsor: String,
    trips_count: u32,
    placing: u32,
    ring: u32,
    scheduled_date: String,
    scheduled_date_mdy: String,
    scheduled_start_time: String,
}

#[derive(Deserialize, Debug)]
struct Classes {
    class_number: u32,
    placing: u32,
    ring: u32,
    name: String,
    scheduled_date: String,
    schedule_starttime: String,
    count: u32
}

const BASE_URL: &str = "https://sglapi.wellingtoninternational.com";
const ORIGIN_HEADER_VALUE: &str = "https://wellingtoninternational.com";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new();
    let headers = build_headers();

    let response = make_request(&client, &headers, "/people/8778?pid=8778&customer_id=15").await?;
    if response.status().is_success() {
        let body = response.json::<PrimaryQueryRes>().await?;
        let mut combined_data_vec = Vec::new();
        let mut seen_riders = HashSet::new();

        for trip in body.trips {
            if !seen_riders.insert(trip.rider_id) {
                continue;
            }

            let path = format!("/entries/{}?eid={}&customer_id=15", trip.entry_id, trip.entry_id);
            let class_response = make_request(&client, &headers, &path).await?.json::<ClassQueryRes>().await?;

            for class in &class_response.classes {
                let scheduled_date = String::from(&class.scheduled_date[0..10]);
                let parts: Vec<&str> = scheduled_date.split('-').collect();
                let scheduled_date_mdy = format!("{}-{}-{}", parts[1], parts[2], parts[0]);


                let combined_data_entry = CombinedData {
                    entry_number: trip.entry_number.clone(),
                    horse: trip.horse.clone(),
                    rider_name: trip.rider_name.clone(),
                    class_number: class.class_number.clone(),
                    class_name: class.name.clone(),
                    sponsor: trip.sponsor.clone(),
                    trips_count: class.count.clone(),
                    placing: class.placing.clone(),
                    ring: class.ring.clone(),
                    scheduled_start_time: class.schedule_starttime.clone(),
                    scheduled_date,
                    scheduled_date_mdy,
                };

                combined_data_vec.push(combined_data_entry)
            }
        }


        print_table_in_console(&combined_data_vec);
        write_to_csv(&combined_data_vec).expect("Uh oh, An error occurred 😟, failed to create CSV.");

    } else {
        println!("Sorry 😟, request failed with status: {}", response.status());
    }

    Ok(())
}

fn build_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ORIGIN, ORIGIN_HEADER_VALUE.parse().unwrap());
    headers
}

async fn make_request(client: &Client, headers: &HeaderMap, path: &str) -> Result<reqwest::Response, ReqError> {
    let url = format!("{}{}", BASE_URL, path);
    client.get(&url).headers(headers.clone()).send().await
}

fn print_table_in_console(combined_data_vec: &Vec<CombinedData>) {
    let mut table = Table::new();

    // Add the headers
    table.add_row(row![
        "Entry", "Horse", "Rider", "Class Number", "Class Name", "Sponsor",
        "Trips", "Placing", "Ring", "Date", "Start Time"
    ]);

    for element in combined_data_vec {
        table.add_row(Row::new(vec![
            Cell::new(&element.entry_number.to_string()),
            Cell::new(&element.horse),
            Cell::new(&element.rider_name),
            Cell::new(&element.class_number.to_string()),
            Cell::new(&element.class_name),
            Cell::new(&element.sponsor),
            Cell::new(&element.trips_count.to_string()),
            Cell::new(&element.placing.to_string()),
            Cell::new(&element.ring.to_string()),
            Cell::new(&element.scheduled_date),
            Cell::new(&element.scheduled_date_mdy),
            Cell::new(&element.scheduled_start_time),
        ]));
    }

    table.printstd();
}

fn write_to_csv(combined_data_vec: &Vec<CombinedData>) -> Result<(), Box<dyn stderr>> {
    let file_path = "wellington_class_data.csv";
    let mut writer = Writer::from_path(file_path)?;

    for element in combined_data_vec {
        writer.serialize(element)?;
    }

    writer.flush()?;
    Ok(())
}