use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::from_str;

const BASE_URL: &str = "https://public.api.clearsky.services";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModerationListEntry {
    #[serde(deserialize_with = "null_string_as_empty")]
    pub url: String,
    #[serde(deserialize_with = "null_string_as_empty")]
    pub did: String,
    #[serde(deserialize_with = "null_string_as_empty")]
    pub name: String,
    #[serde(deserialize_with = "null_string_as_empty")]
    pub description: String,
    #[serde(deserialize_with = "null_string_as_empty")]
    pub created_date: String,
    #[serde(deserialize_with = "null_string_as_empty")]
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

fn null_string_as_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

fn parse_moderation_lists_response(
    body: &str,
) -> Result<Vec<ModerationListEntry>, Box<dyn std::error::Error>> {
    let response = from_str::<GetListResponse>(body).map_err(|err| {
        let snippet = body.chars().take(240).collect::<String>();
        format!("failed to decode Clearsky response: {err}; body starts with: {snippet:?}")
    })?;
    Ok(response.data.lists)
}

pub async fn get_moderation_lists(
    client: &Client,
    actor: &str,
) -> Result<Vec<ModerationListEntry>, Box<dyn std::error::Error>> {
    let actor = urlencoding::encode(actor);
    let body = client
        .get(format!("{BASE_URL}/api/v1/anon/get-list/{actor}"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    parse_moderation_lists_response(&body)
}

#[cfg(test)]
mod tests {
    use super::parse_moderation_lists_response;

    #[test]
    fn parses_expected_clearsky_list_shape() {
        let body = r#"{
            "data": {
                "lists": [
                    {
                        "url": "https://example.com/list-a",
                        "did": "did:plc:abc",
                        "name": "accused troll",
                        "description": "actors accused of trolling",
                        "created_date": "2026-05-07T14:12:26.507000+00:00",
                        "date_added": "2026-05-08T00:00:00+00:00"
                    }
                ]
            }
        }"#;

        let parsed = parse_moderation_lists_response(body).expect("valid Clearsky payload");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "accused troll");
    }

    #[test]
    fn includes_body_snippet_on_decode_failure() {
        let err = parse_moderation_lists_response("<html>bad gateway</html>")
            .expect_err("invalid payload should fail");
        let message = err.to_string();
        assert!(message.contains("failed to decode Clearsky response"));
        assert!(message.contains("bad gateway"));
    }

    #[test]
    fn accepts_null_string_fields_in_clearsky_payload() {
        let body = r#"{
            "data": {
                "lists": [
                    {
                        "url": "https://example.com/list-a",
                        "did": "did:plc:abc",
                        "name": "accused troll",
                        "description": null,
                        "created_date": "2026-05-07T14:12:26.507000+00:00",
                        "date_added": null
                    }
                ]
            }
        }"#;

        let parsed = parse_moderation_lists_response(body).expect("valid Clearsky payload");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].description, "");
        assert_eq!(parsed[0].date_added, "");
    }
}
