use std::{
    io::{self, Write},
    path::PathBuf,
};

use anyhow::Result;
use chrono::Utc;
use comfy_table::Table;
use serde::{Deserialize, Serialize};

use crate::{cli::LicenseType, iracing::IRacingService};

mod cli;
mod error;
mod iracing;
mod settings;

#[derive(Debug, Deserialize)]
pub struct CsvUser {
    id: u32,
}

fn prompt(message: &str) -> Result<String> {
    let mut stdout = io::stdout();
    write!(stdout, "{}: ", message)?;
    stdout.flush()?;

    let stdin = io::stdin();

    let mut buffer = String::new();
    stdin.read_line(&mut buffer)?;

    Ok(buffer)
}

#[derive(Clone, Serialize)]
struct Summary {
    id: String,
    name: String,
    #[serde(rename = "iRating")]
    irating: String,
    license: String,
    #[serde(rename = "SR")]
    sr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::parse();

    let settings = settings::load()?;

    let email = settings
        .auth
        .email
        .ok_or(())
        .or_else(|_| prompt("Email"))?
        .trim()
        .to_string();

    let password = settings
        .auth
        .password
        .ok_or(())
        .or_else(|_| prompt("Password"))?;
    let password = password.trim();

    let drivers = PathBuf::from(args.drivers);

    let mut users = vec![];

    let mut rdr = csv::Reader::from_path(&drivers)?;
    for record in rdr.deserialize::<CsvUser>() {
        let user = record?;
        users.push(user);
    }

    let iracing_service = IRacingService::login(password, email).await?;

    let mut details = vec![];
    for chunk in users.chunks(10) {
        let chunk = iracing_service.get_driver_details(chunk).await?;
        details.extend(chunk);
    }

    let mut summarized = vec![];
    for detail in details {
        let license = detail
            .licenses
            .iter()
            .find(|l| {
                matches!(
                    (l.category.as_str(), args.mode),
                    ("oval", LicenseType::Oval)
                        | ("road", LicenseType::Road)
                        | ("dirt_oval", LicenseType::DirtOval)
                        | ("dirt_road", LicenseType::DirtRoad)
                )
            })
            .expect("definitely present");

        summarized.push(Summary {
            id: detail.cust_id.to_string(),
            name: detail.display_name.to_string(),
            irating: license.irating.to_string(),
            license: license.group_name.to_string(),
            sr: license.safety_rating.to_string(),
        });
    }

    let mut table = Table::new();
    table.set_header(vec!["Id", "Name", "iRating", "License", "SR"]);

    for summary in &summarized {
        let summary = summary.clone();
        table.add_row(vec![
            summary.id,
            summary.name,
            summary.irating,
            summary.license,
            summary.sr,
        ]);
    }

    println!("{table}");

    let mut out_users = drivers.clone();

    let drivers_stem = out_users
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("drivers");
    let unix = Utc::now().timestamp();

    out_users.set_file_name(format!("{}-{}.csv", drivers_stem, unix));

    let mut wrt = csv::Writer::from_path(out_users)?;
    for summary in &summarized {
        wrt.serialize(summary)?;
    }
    wrt.flush()?;

    Ok(())
}
