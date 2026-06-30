use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://public.api.clearsky.services";

#[derive(Clone, Debug, Deserialize)]
pub struct ModerationListEntry {
    pub url: String,
    pub did: String,
    pub name: String,
    pub description: String,
    pub created_date: String,
    pub date_added: String,
}

#[derive(Deserialize)]
struct GetListData {
    lists: Vec<ModerationListEntry>,
}

#[derive(Deserialize)]
struct GetListResponse {
    data: GetListData,
}

pub async fn get_moderation_lists(
    client: &Client,
    actor: &str,
) -> Result<Vec<ModerationListEntry>, Box<dyn std::error::Error>> {
    let actor = urlencoding::encode(actor);
    let response = client
        .get(format!("{BASE_URL}/api/v1/anon/get-list/{actor}"))
        .send()
        .await?
        .error_for_status()?
        .json::<GetListResponse>()
        .await?;

    Ok(response.data.lists)
}
