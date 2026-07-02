use crate::clearsky_v1::ModerationListEntry;
use crate::model::LabeledPostCollection;
use crate::net_backend::NotificationStore;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA_VERSION: i64 = 1;

pub struct AppDb {
    path: PathBuf,
}

pub struct PersistedState {
    pub actors: Vec<PersistedActor>,
    pub post_collections: Vec<LabeledPostCollection>,
    pub clearsky_lists: Vec<PersistedClearskyLists>,
}

pub struct PersistedActor {
    pub did: String,
    pub handle: String,
    pub bio: Option<String>,
}

pub struct PersistedClearskyLists {
    pub actor_did: String,
    pub lists: Vec<ModerationListEntry>,
}

impl AppDb {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Self { path: path.into() };
        db.init()?;
        Ok(db)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_state(&self) -> Result<PersistedState, Box<dyn std::error::Error>> {
        let connection = self.connection()?;
        let actors = load_actors(&connection)?;
        let post_collections = load_post_collections(&connection)?;
        let clearsky_lists = load_clearsky_lists(&connection)?;

        Ok(PersistedState {
            actors,
            post_collections,
            clearsky_lists,
        })
    }

    pub fn save_store(&self, store: &NotificationStore) -> Result<(), Box<dyn std::error::Error>> {
        let mut connection = self.connection()?;
        let tx = connection.transaction()?;

        tx.execute("DELETE FROM actors", [])?;
        tx.execute("DELETE FROM post_collections", [])?;
        tx.execute("DELETE FROM clearsky_lists", [])?;

        for actor in store.persisted_actors() {
            tx.execute(
                "INSERT INTO actors (did, handle, bio) VALUES (?1, ?2, ?3)",
                params![actor.did, actor.handle, actor.bio],
            )?;
        }

        for collection in store.persisted_post_collections() {
            tx.execute(
                "INSERT INTO post_collections (id, payload_json, updated_at_unix) VALUES (?1, ?2, ?3)",
                params![
                    collection.id,
                    serde_json::to_string(collection)?,
                    now_unix_timestamp()
                ],
            )?;
        }

        for entry in store.persisted_clearsky_lists() {
            tx.execute(
                "INSERT INTO clearsky_lists (actor_did, payload_json, updated_at_unix) VALUES (?1, ?2, ?3)",
                params![
                    entry.actor_did,
                    serde_json::to_string(&entry.lists)?,
                    now_unix_timestamp()
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let connection = self.connection()?;
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS app_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS actors (
                did TEXT PRIMARY KEY,
                handle TEXT NOT NULL,
                bio TEXT
            );

            CREATE TABLE IF NOT EXISTS post_collections (
                id TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                updated_at_unix INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS clearsky_lists (
                actor_did TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                updated_at_unix INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                label TEXT NOT NULL,
                created_at_unix INTEGER NOT NULL,
                last_selected_notification_uri TEXT,
                last_opened_notification_uri TEXT
            );
            ",
        )?;

        let existing_version = connection
            .query_row(
                "SELECT value FROM app_meta WHERE key = 'schema_version'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        if existing_version.as_deref() != Some("1") {
            connection.execute(
                "INSERT OR REPLACE INTO app_meta (key, value) VALUES ('schema_version', ?1)",
                [SCHEMA_VERSION.to_string()],
            )?;
        }

        Ok(())
    }

    fn connection(&self) -> Result<Connection, Box<dyn std::error::Error>> {
        Ok(Connection::open(&self.path)?)
    }
}

fn load_actors(connection: &Connection) -> Result<Vec<PersistedActor>, Box<dyn std::error::Error>> {
    let mut statement =
        connection.prepare("SELECT did, handle, bio FROM actors ORDER BY handle")?;
    let rows = statement.query_map([], |row| {
        Ok(PersistedActor {
            did: row.get(0)?,
            handle: row.get(1)?,
            bio: row.get(2)?,
        })
    })?;

    let mut actors = Vec::new();
    for row in rows {
        actors.push(row?);
    }
    Ok(actors)
}

fn load_post_collections(
    connection: &Connection,
) -> Result<Vec<LabeledPostCollection>, Box<dyn std::error::Error>> {
    let mut statement = connection
        .prepare("SELECT payload_json FROM post_collections ORDER BY updated_at_unix DESC, id")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;

    let mut collections = Vec::new();
    for row in rows {
        collections.push(serde_json::from_str(&row?)?);
    }
    Ok(collections)
}

fn load_clearsky_lists(
    connection: &Connection,
) -> Result<Vec<PersistedClearskyLists>, Box<dyn std::error::Error>> {
    let mut statement = connection.prepare(
        "SELECT actor_did, payload_json FROM clearsky_lists ORDER BY updated_at_unix DESC, actor_did",
    )?;
    let rows = statement.query_map([], |row| {
        let actor_did = row.get::<_, String>(0)?;
        let payload_json = row.get::<_, String>(1)?;
        Ok((actor_did, payload_json))
    })?;

    let mut entries = Vec::new();
    for row in rows {
        let (actor_did, payload_json) = row?;
        entries.push(PersistedClearskyLists {
            actor_did,
            lists: serde_json::from_str(&payload_json)?,
        });
    }
    Ok(entries)
}

fn now_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}
