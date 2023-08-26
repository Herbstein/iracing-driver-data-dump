use anyhow::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::{Request, Response, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::{error::Error, CsvUser};

const BASE_URL: &str = "https://members-ng.iracing.com/";

#[derive(Serialize)]
struct LoginData {
    email: String,
    password: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Success {
        #[serde(flatten)]
        data: T,
    },
    Failure,
}

#[derive(Deserialize)]
struct Members {
    members: Vec<Member>,
}

#[derive(Debug, Deserialize)]
pub struct Member {
    pub cust_id: u32,
    pub display_name: String,
    pub licenses: Vec<License>,
}

#[derive(Debug, Deserialize)]
pub struct License {
    pub category: String,
    #[serde(default)]
    pub irating: u32,
    pub group_name: String,
    pub safety_rating: f32,
}

#[derive(Deserialize)]
struct LinkResponse {
    link: Url,
}

pub struct IRacingService {
    client: reqwest::Client,
    base_url: Url,
}

impl IRacingService {
    pub async fn login(password: &str, email: String) -> Result<Self> {
        let credential = build_credentials(password, &email);

        let client = reqwest::Client::builder().cookie_store(true).build()?;

        let base_url = Url::parse(BASE_URL)?;

        let auth_url = base_url.join("auth")?;

        let resp = client
            .post(auth_url)
            .json(&LoginData {
                email,
                password: credential,
            })
            .send()
            .await?;
        error_check(&resp)?;

        Ok(Self { client, base_url })
    }

    pub async fn get_driver_details(&self, drivers: &[CsvUser]) -> Result<Vec<Member>> {
        let url = self.base_url.join("data/member/get/")?;

        let drivers = drivers
            .iter()
            .map(|d| d.id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let req = self
            .client
            .get(url)
            .query(&[("cust_ids", &drivers)])
            .query(&[("include_licenses", true)])
            .build()?;

        let api_resp = self.make_link_request::<Members>(req).await?;
        let data = match api_resp {
            ApiResponse::Success { data } => data,
            ApiResponse::Failure => return Err(Error::UnknownApiResponseError.into()),
        };

        Ok(data.members)
    }

    async fn make_link_request<T: DeserializeOwned>(
        &self,
        request: Request,
    ) -> Result<ApiResponse<T>> {
        let link_resp = self.client.execute(request).await?;
        error_check(&link_resp)?;

        let link = link_resp.json::<LinkResponse>().await?;

        let data_resp = self.client.get(link.link).send().await?;
        error_check(&data_resp)?;

        let data = data_resp.json().await?;

        Ok(data)
    }
}

fn error_check(response: &Response) -> Result<()> {
    if response.status() == StatusCode::SERVICE_UNAVAILABLE {
        return Err(Error::DownForMaintenance.into());
    }

    response.error_for_status_ref()?;

    Ok(())
}

fn build_credentials(password: &str, email: &str) -> String {
    let combined = format!("{password}{}", email.to_lowercase());

    let mut sha = Sha256::new();
    sha.update(&combined);

    let hashed = sha.finalize();

    BASE64_STANDARD.encode(hashed)
}
