use crate::clearsky_v1;
use crate::model::{LabeledPostCollection, PostRecord};
use bsky_sdk::BskyAgent;
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use bsky_sdk::api::app::bsky::{actor, feed};
use bsky_sdk::api::types::string::{AtIdentifier, Did};
use bsky_sdk::api::types::{Union, Unknown};
use reqwest::Client;
use std::collections::{HashMap, HashSet};

pub struct NotificationStore {
    pub notifications: Vec<Notification>,
    notification_keys: HashSet<String>,
    reply_texts: HashMap<String, String>,
    bios: HashMap<String, Option<String>>,
    post_collections: HashMap<String, LabeledPostCollection>,
    pinned_post_replies: HashMap<String, Vec<CachedThreadReply>>,
    clearsky_lists: HashMap<String, Vec<clearsky_v1::ModerationListEntry>>,
    did_to_handle: HashMap<String, String>,
    handle_to_did: HashMap<String, String>,
    clearsky_client: Client,
}

pub struct PersistedActorRow {
    pub did: String,
    pub handle: String,
    pub bio: Option<String>,
}

pub struct PersistedClearskyListsRow {
    pub actor_did: String,
    pub lists: Vec<clearsky_v1::ModerationListEntry>,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            notification_keys: HashSet::new(),
            reply_texts: HashMap::new(),
            bios: HashMap::new(),
            post_collections: HashMap::new(),
            pinned_post_replies: HashMap::new(),
            clearsky_lists: HashMap::new(),
            did_to_handle: HashMap::new(),
            handle_to_did: HashMap::new(),
            clearsky_client: Client::new(),
        }
    }

    pub fn insert_notification(&mut self, notif: Notification, reply_text: Option<String>) -> bool {
        let key = notification_key(&notif);
        if !self.notification_keys.insert(key) {
            return false;
        }

        if let Some(text) = reply_text {
            self.reply_texts.insert(notif.data.uri.clone(), text);
        }

        self.cache_actor(
            &notif.author.data.did,
            notif.author.data.handle.as_str(),
            notif.author.data.description.clone(),
        );
        self.notifications.insert(0, notif);
        true
    }

    pub fn cache_actor(&mut self, did: &Did, handle: &str, bio: Option<String>) {
        let did_key = did.as_str().to_owned();
        let handle_key = handle.to_owned();
        self.did_to_handle
            .insert(did_key.clone(), handle_key.clone());
        self.handle_to_did.insert(handle_key, did_key.clone());
        self.bios.entry(did_key).or_insert(bio);
    }

    pub fn cache_bio(&mut self, did: &Did, bio: Option<String>) {
        self.bios.insert(did.as_str().to_owned(), bio);
    }

    pub fn get_bio(&self, did: &Did) -> Option<Option<&str>> {
        self.bios.get(did.as_str()).map(|bio| bio.as_deref())
    }

    pub fn cache_post_collection(&mut self, collection: LabeledPostCollection) {
        self.post_collections
            .insert(collection.id.clone(), collection);
    }

    pub fn get_post_collection(&self, collection_id: &str) -> Option<&LabeledPostCollection> {
        self.post_collections.get(collection_id)
    }

    pub fn get_pinned_posts(&self, did: &Did) -> Option<&[PostRecord]> {
        self.post_collections
            .get(&pinned_posts_collection_id(did))
            .map(|collection| collection.posts.as_slice())
    }

    #[allow(dead_code)]
    pub fn get_recent_posts(&self, did: &Did) -> Option<&[PostRecord]> {
        self.post_collections
            .get(&recent_posts_collection_id(did))
            .map(|collection| collection.posts.as_slice())
    }

    pub fn get_pinned_posts_collection(&self, did: &Did) -> Option<&LabeledPostCollection> {
        self.get_post_collection(&pinned_posts_collection_id(did))
    }

    #[allow(dead_code)]
    pub fn get_recent_posts_collection(&self, did: &Did) -> Option<&LabeledPostCollection> {
        self.get_post_collection(&recent_posts_collection_id(did))
    }

    pub fn cache_pinned_post_replies(&mut self, post_uri: &str, replies: Vec<CachedThreadReply>) {
        self.pinned_post_replies
            .insert(post_uri.to_owned(), replies);
    }

    pub fn get_pinned_post_replies(&self, post_uri: &str) -> Option<&[CachedThreadReply]> {
        self.pinned_post_replies.get(post_uri).map(Vec::as_slice)
    }

    pub fn cache_clearsky_lists(
        &mut self,
        did: &Did,
        lists: Vec<clearsky_v1::ModerationListEntry>,
    ) {
        self.clearsky_lists.insert(did.as_str().to_owned(), lists);
    }

    pub fn get_clearsky_lists(&self, did: &Did) -> Option<&[clearsky_v1::ModerationListEntry]> {
        self.clearsky_lists.get(did.as_str()).map(Vec::as_slice)
    }

    pub fn find_did(&self, actor: &str) -> Option<Did> {
        if actor.starts_with("did:") {
            actor.parse().ok()
        } else {
            self.handle_to_did
                .get(actor)
                .and_then(|did| did.parse().ok())
        }
    }

    pub fn get_handle(&self, did: &Did) -> Option<&str> {
        self.did_to_handle.get(did.as_str()).map(String::as_str)
    }

    pub fn replies_from(&self, did: &Did) -> Vec<&Notification> {
        self.notifications
            .iter()
            .filter(|notif| notif.data.reason == "reply" && notif.author.data.did == *did)
            .collect()
    }

    pub fn persisted_actors(&self) -> Vec<PersistedActorRow> {
        let mut rows = self
            .did_to_handle
            .iter()
            .map(|(did, handle)| PersistedActorRow {
                did: did.clone(),
                handle: handle.clone(),
                bio: self.bios.get(did).cloned().flatten(),
            })
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| left.handle.cmp(&right.handle));
        rows
    }

    pub fn persisted_post_collections(&self) -> Vec<&LabeledPostCollection> {
        let mut collections = self.post_collections.values().collect::<Vec<_>>();
        collections.sort_by(|left, right| left.id.cmp(&right.id));
        collections
    }

    pub fn persisted_clearsky_lists(&self) -> Vec<PersistedClearskyListsRow> {
        let mut rows = self
            .clearsky_lists
            .iter()
            .map(|(actor_did, lists)| PersistedClearskyListsRow {
                actor_did: actor_did.clone(),
                lists: lists.clone(),
            })
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| left.actor_did.cmp(&right.actor_did));
        rows
    }

    pub fn restore_actor_cache(
        &mut self,
        did: &str,
        handle: &str,
        bio: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let did: Did = did.parse()?;
        self.cache_actor(&did, handle, bio.clone());
        self.cache_bio(&did, bio);
        Ok(())
    }

    pub fn restore_clearsky_lists(
        &mut self,
        actor_did: &str,
        lists: Vec<clearsky_v1::ModerationListEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let did: Did = actor_did.parse()?;
        self.cache_clearsky_lists(&did, lists);
        Ok(())
    }
}

#[derive(Clone)]
pub struct ActorProfile {
    pub did: Did,
    pub handle: String,
    pub bio: Option<String>,
}

pub struct ReplyNode {
    pub text: Option<String>,
    pub parent_uri: Option<String>,
    pub root_uri: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostFacet {
    pub feature_type: String,
    pub uri: Option<String>,
    pub did: Option<String>,
    pub tag: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostDetails {
    pub text: Option<String>,
    pub facets: Vec<PostFacet>,
    pub quoted_text: Option<String>,
}

#[derive(Clone)]
pub struct CachedThreadReply {
    pub author_handle: String,
    pub uri: String,
    pub text: String,
    pub children: Vec<CachedThreadReply>,
}

pub async fn poll_notifications(
    agent: &BskyAgent,
    store: &mut NotificationStore,
) -> Result<usize, Box<dyn std::error::Error>> {
    let notifications = agent
        .api
        .app
        .bsky
        .notification
        .list_notifications(
            bsky_sdk::api::app::bsky::notification::list_notifications::ParametersData {
                cursor: None,
                limit: None,
                priority: None,
                seen_at: None,
                reasons: None,
            }
            .into(),
        )
        .await?;

    let mut any_unread = false;
    let mut new_count = 0;
    let mut seen_dids: HashSet<Did> = HashSet::new();

    for notif in notifications.data.notifications {
        if !notif.data.is_read {
            any_unread = true;
        }

        let reply_text = if notif.data.reason == "reply" {
            extract_reply_node(&notif.data.record).text
        } else {
            None
        };

        let did = notif.author.data.did.clone();
        if store.insert_notification(notif, reply_text) {
            new_count += 1;
        }
        seen_dids.insert(did);
    }

    for did in seen_dids {
        ensure_recent_posts_cached(agent, store, &did, 20).await?;
        ensure_pinned_posts_cached(agent, store, &did).await?;
        ensure_clearsky_lists_cached(store, &did).await?;
    }

    if any_unread {
        let _ = agent
            .api
            .app
            .bsky
            .notification
            .update_seen(
                bsky_sdk::api::app::bsky::notification::update_seen::InputData {
                    seen_at: bsky_sdk::api::types::string::Datetime::now(),
                }
                .into(),
            )
            .await;
    }

    Ok(new_count)
}

pub async fn ensure_actor_profile_cached(
    agent: &BskyAgent,
    store: &mut NotificationStore,
    actor_ref: &str,
) -> Result<ActorProfile, Box<dyn std::error::Error>> {
    if let Some(did) = store.find_did(actor_ref) {
        if let (Some(handle), Some(bio)) = (
            store.get_handle(&did),
            store.get_bio(&did).map(|bio| bio.map(str::to_owned)),
        ) {
            return Ok(ActorProfile {
                did,
                handle: handle.to_owned(),
                bio,
            });
        }
    }

    let actor: AtIdentifier = actor_ref
        .parse()
        .map_err(|err| format!("invalid actor identifier {actor_ref}: {err}"))?;
    let profile = agent
        .api
        .app
        .bsky
        .actor
        .get_profile(actor::get_profile::ParametersData { actor }.into())
        .await?;

    let did = profile.data.did.clone();
    let handle = profile.data.handle.to_string();
    let bio = profile.data.description.clone();

    store.cache_actor(&did, &handle, bio.clone());
    store.cache_bio(&did, bio.clone());

    Ok(ActorProfile { did, handle, bio })
}

pub async fn ensure_pinned_posts_cached(
    agent: &BskyAgent,
    store: &mut NotificationStore,
    did: &Did,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.get_pinned_posts_collection(did).is_some() {
        return Ok(());
    }

    let feed = agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(
            feed::get_author_feed::ParametersData {
                actor: AtIdentifier::Did(did.clone()),
                cursor: None,
                filter: None,
                include_pins: Some(true),
                limit: None,
            }
            .into(),
        )
        .await?;

    let pinned_posts: Vec<feed::defs::PostView> = feed
        .data
        .feed
        .into_iter()
        .filter_map(|item| {
            if matches!(
                item.reason,
                Some(Union::Refs(feed::defs::FeedViewPostReasonRefs::ReasonPin(
                    _
                )))
            ) {
                Some(item.post.clone())
            } else {
                None
            }
        })
        .collect();

    for post in &pinned_posts {
        ensure_pinned_post_replies_cached(agent, store, &post.data.uri).await?;
    }

    store.cache_post_collection(build_pinned_posts_collection(did, pinned_posts));
    Ok(())
}

#[allow(dead_code)]
pub async fn ensure_recent_posts_cached(
    agent: &BskyAgent,
    store: &mut NotificationStore,
    did: &Did,
    limit: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.get_recent_posts_collection(did).is_some() {
        return Ok(());
    }

    let feed = agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(
            feed::get_author_feed::ParametersData {
                actor: AtIdentifier::Did(did.clone()),
                cursor: None,
                filter: None,
                include_pins: Some(false),
                limit: Some(limit.try_into()?),
            }
            .into(),
        )
        .await?;

    let recent_posts = feed
        .data
        .feed
        .into_iter()
        .map(|item| item.post.clone())
        .collect();

    store.cache_post_collection(build_recent_posts_collection(did, recent_posts));
    Ok(())
}

pub async fn ensure_pinned_post_replies_cached(
    agent: &BskyAgent,
    store: &mut NotificationStore,
    post_uri: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.get_pinned_post_replies(post_uri).is_some() {
        return Ok(());
    }

    let thread = agent
        .api
        .app
        .bsky
        .feed
        .get_post_thread(
            feed::get_post_thread::ParametersData {
                depth: Some(6u16.try_into()?),
                parent_height: Some(0u16.try_into()?),
                uri: post_uri.to_owned(),
            }
            .into(),
        )
        .await?;

    let replies = match thread.data.thread {
        Union::Refs(feed::get_post_thread::OutputThreadRefs::AppBskyFeedDefsThreadViewPost(
            thread,
        )) => collect_thread_replies(&thread),
        _ => Vec::new(),
    };

    store.cache_pinned_post_replies(post_uri, replies);
    Ok(())
}

pub async fn ensure_clearsky_lists_cached(
    store: &mut NotificationStore,
    did: &Did,
) -> Result<(), Box<dyn std::error::Error>> {
    if store.get_clearsky_lists(did).is_some() {
        return Ok(());
    }

    let lists = clearsky_v1::get_moderation_lists(&store.clearsky_client, did.as_str()).await?;
    store.cache_clearsky_lists(did, lists.clone());
    store.cache_post_collection(build_clearsky_lists_collection(did, lists));
    Ok(())
}

pub fn extract_reply_node(record: &Unknown) -> ReplyNode {
    let value = serde_json::to_value(record).ok();
    let text = value
        .as_ref()
        .and_then(|value| value.get("text"))
        .and_then(|value| value.as_str())
        .map(String::from);
    let parent_uri = value
        .as_ref()
        .and_then(|value| value.get("reply"))
        .and_then(|value| value.get("parent"))
        .and_then(|value| value.get("uri"))
        .and_then(|value| value.as_str())
        .map(String::from);
    let root_uri = value
        .as_ref()
        .and_then(|value| value.get("reply"))
        .and_then(|value| value.get("root"))
        .and_then(|value| value.get("uri"))
        .and_then(|value| value.as_str())
        .map(String::from);

    ReplyNode {
        text,
        parent_uri,
        root_uri,
    }
}

pub fn extract_post_text(record: &Unknown) -> Option<String> {
    let details = extract_post_details(record)?;
    let mut lines = Vec::new();

    if let Some(text) = details.text {
        lines.push(text);
    }

    for facet in details.facets {
        if let Some(uri) = facet.uri {
            lines.push(format!("link: {uri}"));
        } else if let Some(did) = facet.did {
            lines.push(format!("mention: {did}"));
        } else if let Some(tag) = facet.tag {
            lines.push(format!("tag: {tag}"));
        }
    }

    if let Some(quoted_text) = details.quoted_text {
        lines.push("quoted:".to_string());
        lines.extend(quoted_text.lines().map(str::to_owned));
    }

    Some(lines.join("\n"))
}

pub fn extract_post_details(record: &Unknown) -> Option<PostDetails> {
    let value = serde_json::to_value(record).ok()?;
    let text = value
        .get("text")
        .and_then(|value| value.as_str())
        .map(str::to_owned);

    let facets = value
        .get("facets")
        .and_then(|value| value.as_array())
        .map(|facets| {
            facets
                .iter()
                .flat_map(|facet| {
                    facet
                        .get("features")
                        .and_then(|value| value.as_array())
                        .into_iter()
                        .flatten()
                        .map(|feature| PostFacet {
                            feature_type: feature
                                .get("$type")
                                .and_then(|value| value.as_str())
                                .unwrap_or("unknown")
                                .to_owned(),
                            uri: feature
                                .get("uri")
                                .and_then(|value| value.as_str())
                                .map(str::to_owned),
                            did: feature
                                .get("did")
                                .and_then(|value| value.as_str())
                                .map(str::to_owned),
                            tag: feature
                                .get("tag")
                                .and_then(|value| value.as_str())
                                .map(str::to_owned),
                        })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let quoted_text = value
        .get("embed")
        .and_then(|embed| embed.get("record"))
        .and_then(|record| record.get("value"))
        .and_then(|value| value.get("text"))
        .and_then(|value| value.as_str())
        .map(str::to_owned);

    Some(PostDetails {
        text,
        facets,
        quoted_text,
    })
}

fn build_pinned_posts_collection(
    did: &Did,
    posts: Vec<feed::defs::PostView>,
) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert("collection_kind".to_string(), "pinned_posts".to_string());
    metadata.insert("actor_did".to_string(), did.as_str().to_owned());

    LabeledPostCollection::new(
        pinned_posts_collection_id(did),
        format!("Pinned posts by {}", did.as_str()),
        posts.into_iter().map(post_record_from_view).collect(),
    )
    .with_metadata(metadata)
}

fn build_recent_posts_collection(
    did: &Did,
    posts: Vec<feed::defs::PostView>,
) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert("collection_kind".to_string(), "recent_posts".to_string());
    metadata.insert("actor_did".to_string(), did.as_str().to_owned());

    LabeledPostCollection::new(
        recent_posts_collection_id(did),
        format!("Recent posts by {}", did.as_str()),
        posts.into_iter().map(post_record_from_view).collect(),
    )
    .with_related_collections(vec![pinned_posts_collection_id(did)])
    .with_metadata(metadata)
}

fn build_clearsky_lists_collection(
    did: &Did,
    lists: Vec<clearsky_v1::ModerationListEntry>,
) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert("collection_kind".to_string(), "clearsky_lists".to_string());
    metadata.insert("actor_did".to_string(), did.as_str().to_owned());

    LabeledPostCollection::new(
        clearsky_lists_collection_id(did),
        format!("Clearsky moderation lists for {}", did.as_str()),
        lists.into_iter().map(clearsky_list_record).collect(),
    )
    .with_metadata(metadata)
}

fn post_record_from_view(post: feed::defs::PostView) -> PostRecord {
    PostRecord {
        uri: post.uri.clone(),
        author_handle: post.author.data.handle.to_string(),
        body: extract_post_text(&post.record).unwrap_or_default(),
    }
}

fn clearsky_list_record(entry: clearsky_v1::ModerationListEntry) -> PostRecord {
    PostRecord {
        uri: entry.url.clone(),
        author_handle: "clearsky".to_string(),
        body: format!(
            "list_name: {}\nlist_description: {}\nlist_did: {}\ncreated_date: {}\ndate_added: {}\nurl: {}",
            entry.name,
            entry.description,
            entry.did,
            entry.created_date,
            entry.date_added,
            entry.url
        ),
    }
}

fn pinned_posts_collection_id(did: &Did) -> String {
    format!("pinned_posts:{}", did.as_str())
}

fn recent_posts_collection_id(did: &Did) -> String {
    format!("recent_posts:{}", did.as_str())
}

fn clearsky_lists_collection_id(did: &Did) -> String {
    format!("clearsky_lists:{}", did.as_str())
}

fn collect_thread_replies(thread: &feed::defs::ThreadViewPost) -> Vec<CachedThreadReply> {
    let Some(children) = thread.data.replies.as_ref() else {
        return Vec::new();
    };

    let mut replies = Vec::new();
    for child in children {
        if let Union::Refs(feed::defs::ThreadViewPostRepliesItem::ThreadViewPost(post)) = child {
            replies.push(CachedThreadReply {
                author_handle: post.data.post.author.data.handle.to_string(),
                uri: post.data.post.uri.clone(),
                text: extract_post_text(&post.data.post.record).unwrap_or_default(),
                children: collect_thread_replies(post),
            });
        }
    }
    replies
}

fn notification_key(notif: &Notification) -> String {
    format!(
        "{}|{}|{}|{}",
        notif.author.data.did.as_str(),
        notif.data.reason,
        notif.data.uri,
        notif.indexed_at.as_ref()
    )
}
