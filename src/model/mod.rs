use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostRecord {
    pub uri: String,
    pub author_handle: String,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabeledPostCollection {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub collection_kind: String,
    #[serde(default)]
    pub actor_did: Option<String>,
    pub posts: Vec<PostRecord>,
    pub related_collection_ids: Vec<String>,
    #[serde(default)]
    pub last_refreshed_at: i64,
    #[serde(default)]
    pub refresh_ttl_seconds: Option<u64>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl LabeledPostCollection {
    pub fn new(id: impl Into<String>, label: impl Into<String>, posts: Vec<PostRecord>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            collection_kind: String::new(),
            actor_did: None,
            posts,
            related_collection_ids: Vec::new(),
            last_refreshed_at: 0,
            refresh_ttl_seconds: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_collection_kind(mut self, collection_kind: impl Into<String>) -> Self {
        self.collection_kind = collection_kind.into();
        self
    }

    pub fn with_actor_did(mut self, actor_did: impl Into<String>) -> Self {
        self.actor_did = Some(actor_did.into());
        self
    }

    pub fn with_related_collections(mut self, related_collection_ids: Vec<String>) -> Self {
        self.related_collection_ids = related_collection_ids;
        self
    }

    pub fn with_refresh_state(
        mut self,
        last_refreshed_at: i64,
        refresh_ttl_seconds: Option<u64>,
    ) -> Self {
        self.last_refreshed_at = last_refreshed_at;
        self.refresh_ttl_seconds = refresh_ttl_seconds;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}
