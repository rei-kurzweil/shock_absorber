use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostRecord {
    pub uri: String,
    pub author_handle: String,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabeledPostCollection {
    pub id: String,
    pub label: String,
    pub posts: Vec<PostRecord>,
    pub related_collection_ids: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl LabeledPostCollection {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        posts: Vec<PostRecord>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            posts,
            related_collection_ids: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_related_collections(
        mut self,
        related_collection_ids: Vec<String>,
    ) -> Self {
        self.related_collection_ids = related_collection_ids;
        self
    }

    pub fn with_metadata(
        mut self,
        metadata: HashMap<String, String>,
    ) -> Self {
        self.metadata = metadata;
        self
    }
}
