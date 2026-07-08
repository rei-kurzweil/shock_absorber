use crate::clearsky_v1;
use crate::model::{LabeledPostCollection, PostRecord};
use bsky_sdk::BskyAgent;
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use bsky_sdk::api::app::bsky::{actor, feed};
use bsky_sdk::api::types::string::{AtIdentifier, Did};
use bsky_sdk::api::types::{Union, Unknown};
use reqwest::Client;
use std::collections::{HashMap, HashSet};

const DEFAULT_COLLECTION_REFRESH_TTL_SECONDS: u64 = 15 * 60;
const AUTHOR_FEED_PAGE_SIZE: usize = 25;

#[derive(Clone)]
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
        let collection = normalize_collection(collection);
        self.post_collections
            .insert(collection.id.clone(), collection);
    }

    pub fn remove_post_collection(&mut self, collection_id: &str) {
        self.post_collections.remove(collection_id);
    }

    pub fn get_post_collection(&self, collection_id: &str) -> Option<&LabeledPostCollection> {
        self.post_collections.get(collection_id)
    }

    pub fn post_collections(&self) -> Vec<&LabeledPostCollection> {
        let mut collections = self.post_collections.values().collect::<Vec<_>>();
        collections.sort_by(|left, right| left.id.cmp(&right.id));
        collections
    }

    pub fn actor_post_collections(&self, actor_did: &Did) -> Vec<&LabeledPostCollection> {
        let actor = actor_did.as_str();
        self.post_collections()
            .into_iter()
            .filter(|collection| collection.actor_did.as_deref() == Some(actor))
            .collect()
    }

    pub fn cached_actor_dids(&self) -> Vec<Did> {
        let mut dids = self
            .post_collections
            .values()
            .filter_map(|collection| collection.actor_did.as_deref())
            .filter_map(|did| did.parse().ok())
            .collect::<Vec<Did>>();
        dids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        dids.dedup_by(|left, right| left.as_str() == right.as_str());
        dids
    }

    pub fn get_pinned_posts(&self, did: &Did) -> Option<&[PostRecord]> {
        self.post_collections
            .get(&pinned_posts_collection_id(did))
            .map(|collection| collection.posts.as_slice())
    }

    #[allow(dead_code)]
    pub fn get_recent_posts_unaddressed(&self, did: &Did) -> Option<&[PostRecord]> {
        self.post_collections
            .get(&recent_posts_unaddressed_collection_id(did))
            .map(|collection| collection.posts.as_slice())
    }

    pub fn get_pinned_posts_collection(&self, did: &Did) -> Option<&LabeledPostCollection> {
        self.get_post_collection(&pinned_posts_collection_id(did))
    }

    #[allow(dead_code)]
    pub fn get_recent_posts_unaddressed_collection(
        &self,
        did: &Did,
    ) -> Option<&LabeledPostCollection> {
        self.get_post_collection(&recent_posts_unaddressed_collection_id(did))
    }

    #[allow(dead_code)]
    pub fn get_recent_replies_sent_collection(&self, did: &Did) -> Option<&LabeledPostCollection> {
        self.get_post_collection(&recent_replies_sent_collection_id(did))
    }

    #[allow(dead_code)]
    pub fn get_recent_replies_received_collection(
        &self,
        did: &Did,
    ) -> Option<&LabeledPostCollection> {
        self.get_post_collection(&recent_replies_received_collection_id(did))
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
        ensure_recent_posts_cached(agent, store, &did, 20, 10).await?;
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
            let profile = ActorProfile {
                did,
                handle: handle.to_owned(),
                bio,
            };
            store.cache_post_collection(build_actor_profile_collection(&profile));
            return Ok(profile);
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
    store.cache_post_collection(build_actor_profile_collection(&ActorProfile {
        did: did.clone(),
        handle: handle.clone(),
        bio: bio.clone(),
    }));

    Ok(ActorProfile { did, handle, bio })
}

pub async fn cache_global_search_posts(
    agent: &BskyAgent,
    store: &mut NotificationStore,
    query: &str,
    limit: u8,
) -> Result<LabeledPostCollection, Box<dyn std::error::Error>> {
    let result = agent
        .api
        .app
        .bsky
        .feed
        .search_posts(
            feed::search_posts::ParametersData {
                author: None,
                cursor: None,
                domain: None,
                lang: None,
                limit: Some(limit.try_into()?),
                mentions: None,
                q: query.to_string(),
                since: None,
                sort: None,
                tag: None,
                until: None,
                url: None,
            }
            .into(),
        )
        .await?;

    let collection = build_global_search_posts_collection(query, result.data.posts);
    store.cache_post_collection(collection.clone());
    Ok(collection)
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
    feed_fetch_limit: usize,
    min_top_level_posts: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if let (Some(posts), Some(replies)) = (
        store.get_recent_posts_unaddressed_collection(did),
        store.get_recent_replies_sent_collection(did),
    ) {
        let cached_total = posts.posts.len().saturating_add(replies.posts.len());
        if posts.posts.len() >= min_top_level_posts || cached_total >= feed_fetch_limit {
            return Ok(());
        }
    }

    let mut cursor = None;
    let mut recent_posts = Vec::new();
    let mut remaining_fetch_budget = feed_fetch_limit;

    while remaining_fetch_budget > 0 {
        let page_size = remaining_fetch_budget.min(AUTHOR_FEED_PAGE_SIZE);
        let feed = agent
            .api
            .app
            .bsky
            .feed
            .get_author_feed(
                feed::get_author_feed::ParametersData {
                    actor: AtIdentifier::Did(did.clone()),
                    cursor: cursor.clone(),
                    filter: None,
                    include_pins: Some(false),
                    limit: Some(u8::try_from(page_size)?.try_into()?),
                }
                .into(),
            )
            .await?;

        let next_cursor = feed.data.cursor.clone();
        let page_posts = feed
            .data
            .feed
            .into_iter()
            .map(|item| item.post.clone())
            .collect::<Vec<_>>();
        if page_posts.is_empty() {
            break;
        }

        remaining_fetch_budget = remaining_fetch_budget.saturating_sub(page_posts.len());
        recent_posts.extend(page_posts);

        let top_level_count = recent_posts
            .iter()
            .filter(|post| post_is_authored_by(post, did))
            .filter(|post| !is_reply_record(&post.record))
            .count();
        if top_level_count >= min_top_level_posts {
            break;
        }

        let Some(next_cursor) = next_cursor else {
            break;
        };
        cursor = Some(next_cursor);
    }

    let (recent_posts_unaddressed, recent_replies_sent) =
        partition_recent_posts_for_actor(recent_posts, did);

    store.remove_post_collection(&recent_posts_collection_id(did));
    store.cache_post_collection(build_recent_posts_unaddressed_collection(
        did,
        recent_posts_unaddressed,
    ));
    store.cache_post_collection(build_recent_replies_sent_collection(
        did,
        recent_replies_sent,
    ));
    Ok(())
}

pub async fn ensure_recent_replies_received_cached(
    agent: &BskyAgent,
    store: &mut NotificationStore,
    did: &Did,
    post_limit: usize,
    reply_limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(existing) = store.get_recent_replies_received_collection(did) {
        if existing.posts.len() >= reply_limit.min(25) {
            return Ok(());
        }
    }

    ensure_recent_posts_cached(agent, store, did, post_limit, 25).await?;
    ensure_pinned_posts_cached(agent, store, did).await?;

    let mut source_posts = Vec::new();
    if let Some(collection) = store.get_recent_posts_unaddressed_collection(did) {
        source_posts.extend(collection.posts.iter().map(|post| post.uri.clone()));
    }
    if let Some(collection) = store.get_pinned_posts_collection(did) {
        source_posts.extend(collection.posts.iter().map(|post| post.uri.clone()));
    }
    source_posts.sort();
    source_posts.dedup();

    for post_uri in &source_posts {
        ensure_post_replies_cached(agent, store, post_uri).await?;
    }

    let actor_handle = store.get_handle(did).map(str::to_owned);
    let replies = recent_replies_received_posts(store, &source_posts)
        .into_iter()
        .filter(|post| {
            actor_handle
                .as_deref()
                .map_or(true, |handle| post.author_handle != handle)
        })
        .take(reply_limit)
        .collect::<Vec<_>>();

    store.cache_post_collection(build_recent_replies_received_collection(
        did,
        replies,
        source_posts,
    ));
    Ok(())
}

fn recent_replies_received_posts(
    store: &NotificationStore,
    source_posts: &[String],
) -> Vec<PostRecord> {
    source_posts
        .iter()
        .flat_map(|post_uri| {
            let collection_id = post_replies_collection_id(post_uri);
            if let Some(collection) = store.get_post_collection(&collection_id) {
                return collection.posts.clone();
            }

            store
                .get_pinned_post_replies(post_uri)
                .unwrap_or(&[])
                .iter()
                .flat_map(move |reply| flatten_thread_replies(reply, post_uri))
                .collect()
        })
        .collect()
}

pub async fn ensure_post_replies_cached(
    agent: &BskyAgent,
    store: &mut NotificationStore,
    post_uri: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection_id = post_replies_collection_id(post_uri);
    if store.get_post_collection(&collection_id).is_some() {
        return Ok(());
    }

    let replies = if let Some(cached) = store.get_pinned_post_replies(post_uri) {
        cached.to_vec()
    } else {
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

        match thread.data.thread {
            Union::Refs(
                feed::get_post_thread::OutputThreadRefs::AppBskyFeedDefsThreadViewPost(thread),
            ) => collect_thread_replies(&thread),
            _ => Vec::new(),
        }
    };

    store.cache_pinned_post_replies(post_uri, replies.clone());
    store.cache_post_collection(build_post_replies_collection(post_uri, replies));
    Ok(())
}

pub async fn ensure_pinned_post_replies_cached(
    agent: &BskyAgent,
    store: &mut NotificationStore,
    post_uri: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_post_replies_cached(agent, store, post_uri).await
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
    let metadata = HashMap::new();
    LabeledPostCollection::new(
        pinned_posts_collection_id(did),
        format!("Pinned posts by {}", did.as_str()),
        posts.into_iter().map(post_record_from_view).collect(),
    )
    .with_collection_kind("pinned_posts")
    .with_actor_did(did.as_str())
    .with_refresh_state(
        now_unix_timestamp(),
        Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS),
    )
    .with_metadata(metadata)
}

fn build_actor_profile_collection(profile: &ActorProfile) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "actor_profile".to_string());

    let mut body = vec![
        format!("handle: {}", profile.handle),
        format!("did: {}", profile.did.as_str()),
    ];
    body.push("bio:".to_string());
    match profile.bio.as_deref() {
        Some(bio) if !bio.is_empty() => body.extend(bio.lines().map(str::to_owned)),
        _ => body.push("<none>".to_string()),
    }

    LabeledPostCollection::new(
        actor_profile_collection_id(&profile.did),
        format!("Profile for {}", profile.handle),
        vec![PostRecord {
            uri: format!("at://{}/app.bsky.actor.profile/self", profile.did.as_str()),
            author_handle: profile.handle.clone(),
            body: body.join("\n"),
        }],
    )
    .with_collection_kind("actor_profile")
    .with_actor_did(profile.did.as_str())
    .with_refresh_state(
        now_unix_timestamp(),
        Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS),
    )
    .with_metadata(metadata)
}

fn build_recent_posts_unaddressed_collection(
    did: &Did,
    posts: Vec<feed::defs::PostView>,
) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert("source_feed".to_string(), "author_feed".to_string());

    LabeledPostCollection::new(
        recent_posts_unaddressed_collection_id(did),
        format!("Recent top-level posts by {}", did.as_str()),
        posts.into_iter().map(post_record_from_view).collect(),
    )
    .with_collection_kind("recent_posts_unaddressed")
    .with_actor_did(did.as_str())
    .with_related_collections(vec![pinned_posts_collection_id(did)])
    .with_refresh_state(
        now_unix_timestamp(),
        Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS),
    )
    .with_metadata(metadata)
}

fn build_recent_replies_sent_collection(
    did: &Did,
    posts: Vec<feed::defs::PostView>,
) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert("source_feed".to_string(), "author_feed".to_string());

    LabeledPostCollection::new(
        recent_replies_sent_collection_id(did),
        format!("Recent replies sent by {}", did.as_str()),
        posts.into_iter().map(post_record_from_view).collect(),
    )
    .with_collection_kind("recent_replies_sent")
    .with_actor_did(did.as_str())
    .with_related_collections(vec![recent_posts_unaddressed_collection_id(did)])
    .with_refresh_state(
        now_unix_timestamp(),
        Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS),
    )
    .with_metadata(metadata)
}

fn build_recent_replies_received_collection(
    did: &Did,
    replies: Vec<PostRecord>,
    source_post_uris: Vec<String>,
) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "post_reply_threads".to_string());
    metadata.insert(
        "source_post_count".to_string(),
        source_post_uris.len().to_string(),
    );

    LabeledPostCollection::new(
        recent_replies_received_collection_id(did),
        format!("Recent replies received by {}", did.as_str()),
        replies,
    )
    .with_collection_kind("recent_replies_received")
    .with_actor_did(did.as_str())
    .with_related_collections(source_post_uris)
    .with_refresh_state(
        now_unix_timestamp(),
        Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS),
    )
    .with_metadata(metadata)
}

fn build_post_replies_collection(
    post_uri: &str,
    replies: Vec<CachedThreadReply>,
) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert(
        "source".to_string(),
        "app.bsky.feed.getPostThread".to_string(),
    );
    metadata.insert("source_post_uri".to_string(), post_uri.to_string());

    LabeledPostCollection::new(
        post_replies_collection_id(post_uri),
        format!("Replies to {}", post_uri),
        replies
            .iter()
            .flat_map(|reply| flatten_thread_replies(reply, post_uri))
            .collect(),
    )
    .with_collection_kind("post_replies")
    .with_refresh_state(
        now_unix_timestamp(),
        Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS),
    )
    .with_metadata(metadata)
}

fn build_clearsky_lists_collection(
    did: &Did,
    lists: Vec<clearsky_v1::ModerationListEntry>,
) -> LabeledPostCollection {
    let metadata = HashMap::new();
    LabeledPostCollection::new(
        clearsky_lists_collection_id(did),
        format!("Clearsky moderation lists for {}", did.as_str()),
        lists.into_iter().map(clearsky_list_record).collect(),
    )
    .with_collection_kind("clearsky_lists")
    .with_actor_did(did.as_str())
    .with_refresh_state(
        now_unix_timestamp(),
        Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS),
    )
    .with_metadata(metadata)
}

fn build_global_search_posts_collection(
    query: &str,
    posts: Vec<feed::defs::PostView>,
) -> LabeledPostCollection {
    let mut metadata = HashMap::new();
    metadata.insert(
        "source".to_string(),
        "app.bsky.feed.searchPosts".to_string(),
    );
    metadata.insert("query".to_string(), query.to_string());

    LabeledPostCollection::new(
        global_search_posts_collection_id(query),
        format!("Global Bluesky search results for \"{}\"", query),
        posts.into_iter().map(post_record_from_view).collect(),
    )
    .with_collection_kind("global_search_posts")
    .with_refresh_state(
        now_unix_timestamp(),
        Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS),
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

fn recent_posts_unaddressed_collection_id(did: &Did) -> String {
    format!("recent_posts_unaddressed:{}", did.as_str())
}

fn recent_replies_sent_collection_id(did: &Did) -> String {
    format!("recent_replies_sent:{}", did.as_str())
}

fn recent_replies_received_collection_id(did: &Did) -> String {
    format!("recent_replies_received:{}", did.as_str())
}

fn clearsky_lists_collection_id(did: &Did) -> String {
    format!("clearsky_lists:{}", did.as_str())
}

fn actor_profile_collection_id(did: &Did) -> String {
    format!("actor_profile:{}", did.as_str())
}

fn global_search_posts_collection_id(query: &str) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    query.hash(&mut hasher);
    format!("global_search_posts:{:x}", hasher.finish())
}

fn post_replies_collection_id(post_uri: &str) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    post_uri.hash(&mut hasher);
    format!("post_replies:{:x}", hasher.finish())
}

fn is_reply_record(record: &Unknown) -> bool {
    serde_json::to_value(record)
        .ok()
        .and_then(|value| value.get("reply").cloned())
        .is_some()
}

fn post_is_authored_by(post: &feed::defs::PostView, did: &Did) -> bool {
    post.author.data.did == *did
}

fn partition_recent_posts_for_actor(
    posts: Vec<feed::defs::PostView>,
    did: &Did,
) -> (Vec<feed::defs::PostView>, Vec<feed::defs::PostView>) {
    posts.into_iter()
        .filter(|post| post_is_authored_by(post, did))
        .partition(|post| !is_reply_record(&post.record))
}

fn normalize_collection(mut collection: LabeledPostCollection) -> LabeledPostCollection {
    if collection.collection_kind.is_empty() {
        if let Some(kind) = collection.metadata.get("collection_kind") {
            collection.collection_kind = kind.clone();
        }
    }
    if collection.actor_did.is_none() {
        if let Some(actor_did) = collection.metadata.get("actor_did") {
            collection.actor_did = Some(actor_did.clone());
        }
    }
    if collection.refresh_ttl_seconds.is_none() {
        collection.refresh_ttl_seconds = Some(DEFAULT_COLLECTION_REFRESH_TTL_SECONDS);
    }
    collection
}

fn now_unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
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

fn flatten_thread_replies(reply: &CachedThreadReply, source_post_uri: &str) -> Vec<PostRecord> {
    let mut posts = vec![PostRecord {
        uri: reply.uri.clone(),
        author_handle: reply.author_handle.clone(),
        body: format!(
            "source_post_uri: {source_post_uri}\nreply_text: {}",
            reply.text
        ),
    }];
    for child in &reply.children {
        posts.extend(flatten_thread_replies(child, source_post_uri));
    }
    posts
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

#[cfg(test)]
mod tests {
    use super::*;
    use bsky_sdk::api::app::bsky::actor::defs::{ProfileViewBasic, ProfileViewBasicData};
    use bsky_sdk::api::app::bsky::feed::defs::{PostView, PostViewData};
    use bsky_sdk::api::app::bsky::feed::post::{Record, RecordData, ReplyRef, ReplyRefData};
    use bsky_sdk::api::com::atproto::repo::strong_ref::{Main as StrongRef, MainData};
    use bsky_sdk::api::types::TryIntoUnknown;
    use bsky_sdk::api::types::string::{Cid, Datetime};

    #[test]
    fn recent_replies_received_uses_cached_post_replies_collections() {
        let mut store = NotificationStore::new();
        let source_post_uri = "at://did:plc:testactor/app.bsky.feed.post/root".to_string();
        store.cache_post_collection(
            LabeledPostCollection::new(
                post_replies_collection_id(&source_post_uri),
                format!("Replies to {source_post_uri}"),
                vec![
                    PostRecord {
                        uri: "at://did:plc:reply-author/app.bsky.feed.post/reply-one".to_string(),
                        author_handle: "reply.author".to_string(),
                        body: format!(
                            "source_post_uri: {source_post_uri}\nreply_text: This is grounded inbound feedback."
                        ),
                    },
                    PostRecord {
                        uri: "at://did:plc:testactor/app.bsky.feed.post/reply-two".to_string(),
                        author_handle: "rei-cast.xyz".to_string(),
                        body: format!(
                            "source_post_uri: {source_post_uri}\nreply_text: This is the actor's follow-up."
                        ),
                    },
                ],
            )
            .with_collection_kind("post_replies"),
        );

        let replies = recent_replies_received_posts(&store, std::slice::from_ref(&source_post_uri));

        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0].author_handle, "reply.author");
        assert!(replies[0].body.contains("grounded inbound feedback"));
    }

    #[test]
    fn partition_recent_posts_for_actor_excludes_foreign_authors() {
        let actor_did: Did = "did:plc:testactor".parse().expect("invalid actor did");
        let actor = profile_view_basic("actor.test", &actor_did);
        let other_did: Did = "did:plc:otheractor".parse().expect("invalid other did");
        let other = profile_view_basic("other.test", &other_did);

        let top_level_actor = post_view(
            &actor,
            record("top-level by actor", false),
            "at://did:plc:testactor/app.bsky.feed.post/top-level",
        );
        let reply_actor = post_view(
            &actor,
            record("reply by actor", true),
            "at://did:plc:testactor/app.bsky.feed.post/reply",
        );
        let foreign_top_level = post_view(
            &other,
            record("top-level by other actor", false),
            "at://did:plc:otheractor/app.bsky.feed.post/top-level",
        );

        let (top_level, replies) = partition_recent_posts_for_actor(
            vec![top_level_actor, reply_actor, foreign_top_level],
            &actor_did,
        );

        assert_eq!(top_level.len(), 1);
        assert_eq!(replies.len(), 1);
        assert_eq!(top_level[0].author.data.did, actor_did);
        assert_eq!(replies[0].author.data.did, actor_did);
        assert_eq!(
            extract_post_text(&top_level[0].record).as_deref(),
            Some("top-level by actor")
        );
        assert_eq!(
            extract_post_text(&replies[0].record).as_deref(),
            Some("reply by actor")
        );
    }

    fn profile_view_basic(handle: &str, did: &Did) -> ProfileViewBasic {
        ProfileViewBasicData {
            associated: None,
            avatar: None,
            created_at: None,
            did: did.clone(),
            display_name: None,
            handle: handle.parse().expect("invalid handle"),
            labels: None,
            pronouns: None,
            status: None,
            verification: None,
            viewer: None,
        }
        .into()
    }

    fn post_view(author: &ProfileViewBasic, record: Record, uri: &str) -> PostView {
        PostViewData {
            author: author.clone(),
            bookmark_count: None,
            cid: fake_cid(),
            embed: None,
            indexed_at: Datetime::now(),
            labels: None,
            like_count: None,
            quote_count: None,
            record: record
                .try_into_unknown()
                .expect("failed to convert record to unknown"),
            reply_count: None,
            repost_count: None,
            threadgate: None,
            uri: uri.to_string(),
            viewer: None,
        }
        .into()
    }

    fn record(text: &str, is_reply: bool) -> Record {
        RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets: None,
            labels: None,
            langs: None,
            reply: is_reply.then(fake_reply_ref),
            tags: None,
            text: text.to_string(),
        }
        .into()
    }

    fn fake_reply_ref() -> ReplyRef {
        ReplyRefData {
            parent: fake_strong_ref("at://did:plc:root/app.bsky.feed.post/parent"),
            root: fake_strong_ref("at://did:plc:root/app.bsky.feed.post/root"),
        }
        .into()
    }

    fn fake_strong_ref(uri: &str) -> StrongRef {
        MainData {
            cid: fake_cid(),
            uri: uri.to_string(),
        }
        .into()
    }

    fn fake_cid() -> Cid {
        "bafyreid6jmj4onjsdxp66m7uhs5vvh2n7xk6kqv7sfr5w2umh4m4z3xv4e"
            .parse()
            .expect("invalid cid")
    }
}
