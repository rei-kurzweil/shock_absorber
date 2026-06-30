use crate::clearsky_v1;
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use bsky_sdk::api::app::bsky::{actor, feed};
use bsky_sdk::api::types::string::{AtIdentifier, Did};
use bsky_sdk::api::types::{Union, Unknown};
use bsky_sdk::BskyAgent;
use reqwest::Client;
use std::collections::{HashMap, HashSet};

pub struct NotificationStore {
    pub notifications: Vec<Notification>,
    notification_keys: HashSet<String>,
    reply_texts: HashMap<String, String>,
    bios: HashMap<String, Option<String>>,
    pinned_posts: HashMap<String, Vec<feed::defs::PostView>>,
    pinned_post_replies: HashMap<String, Vec<CachedThreadReply>>,
    clearsky_lists: HashMap<String, Vec<clearsky_v1::ModerationListEntry>>,
    did_to_handle: HashMap<String, String>,
    handle_to_did: HashMap<String, String>,
    clearsky_client: Client,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            notification_keys: HashSet::new(),
            reply_texts: HashMap::new(),
            bios: HashMap::new(),
            pinned_posts: HashMap::new(),
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
        self.did_to_handle.insert(did_key.clone(), handle_key.clone());
        self.handle_to_did.insert(handle_key, did_key.clone());
        self.bios.entry(did_key).or_insert(bio);
    }

    pub fn cache_bio(&mut self, did: &Did, bio: Option<String>) {
        self.bios.insert(did.as_str().to_owned(), bio);
    }

    pub fn get_bio(&self, did: &Did) -> Option<Option<&str>> {
        self.bios.get(did.as_str()).map(|bio| bio.as_deref())
    }

    pub fn cache_pinned_posts(&mut self, did: &Did, posts: Vec<feed::defs::PostView>) {
        self.pinned_posts.insert(did.as_str().to_owned(), posts);
    }

    pub fn get_pinned_posts(&self, did: &Did) -> Option<&[feed::defs::PostView]> {
        self.pinned_posts.get(did.as_str()).map(Vec::as_slice)
    }

    pub fn cache_pinned_post_replies(&mut self, post_uri: &str, replies: Vec<CachedThreadReply>) {
        self.pinned_post_replies.insert(post_uri.to_owned(), replies);
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
            self.handle_to_did.get(actor).and_then(|did| did.parse().ok())
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
    if store.get_pinned_posts(did).is_some() {
        return Ok(());
    }

    let feed = agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(feed::get_author_feed::ParametersData {
            actor: AtIdentifier::Did(did.clone()),
            cursor: None,
            filter: None,
            include_pins: Some(true),
            limit: None,
        }
        .into())
        .await?;

    let pinned_posts: Vec<feed::defs::PostView> = feed
        .data
        .feed
        .into_iter()
        .filter_map(|item| {
            if matches!(
                item.reason,
                Some(Union::Refs(feed::defs::FeedViewPostReasonRefs::ReasonPin(_)))
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

    store.cache_pinned_posts(did, pinned_posts);
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
        .get_post_thread(feed::get_post_thread::ParametersData {
            depth: Some(6u16.try_into()?),
            parent_height: Some(0u16.try_into()?),
            uri: post_uri.to_owned(),
        }
        .into())
        .await?;

    let replies = match thread.data.thread {
        Union::Refs(feed::get_post_thread::OutputThreadRefs::AppBskyFeedDefsThreadViewPost(thread)) => {
            collect_thread_replies(&thread)
        }
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
    store.cache_clearsky_lists(did, lists);
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
    let value = serde_json::to_value(record).ok()?;
    let text = value.get("text")?.as_str()?.to_owned();
    let mut lines = vec![text];

    if let Some(facets) = value.get("facets").and_then(|value| value.as_array()) {
        for facet in facets {
            if let Some(features) = facet.get("features").and_then(|value| value.as_array()) {
                for feature in features {
                    if let Some(uri) = feature.get("uri").and_then(|value| value.as_str()) {
                        lines.push(format!("link: {uri}"));
                    }
                }
            }
        }
    }

    if let Some(embed_text) = value
        .get("embed")
        .and_then(|embed| embed.get("record"))
        .and_then(|record| record.get("value"))
        .and_then(|value| value.get("text"))
        .and_then(|value| value.as_str())
    {
        lines.push("quoted:".to_string());
        lines.extend(embed_text.lines().map(str::to_owned));
    }

    Some(lines.join("\n"))
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
