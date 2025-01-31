use ahash::AHashMap;
use anyhow::{anyhow, Context, Result};
use arc_swap::ArcSwap;
use itertools::Itertools;
use kartoffels_utils::{ArcSwapExt, Id};
use kartoffels_world::prelude::{Config as WorldConfig, Handle as WorldHandle};
use std::collections::hash_map;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, info};

#[derive(Debug)]
pub struct Worlds {
    entries: Arc<ArcSwap<AHashMap<Id, WorldEntry>>>,
    public_idx: ArcSwap<Vec<WorldHandle>>,
    test_next_id: AtomicU64,
}

impl Worlds {
    pub const MAX_WORLDS: usize = 128;

    pub async fn new(dir: Option<&Path>) -> Result<Self> {
        let entries = if let Some(dir) = dir {
            Self::load(dir).await?
        } else {
            Default::default()
        };

        let public_idx = build_public_idx(&entries);

        Ok(Self {
            entries: Arc::new(ArcSwap::from_pointee(entries)),
            public_idx: ArcSwap::from_pointee(public_idx),
            test_next_id: AtomicU64::new(1),
        })
    }

    async fn load(dir: &Path) -> Result<AHashMap<Id, WorldEntry>> {
        let mut entries = AHashMap::new();
        let mut files = fs::read_dir(dir).await?;

        while let Some(file) = files.next_entry().await? {
            let path = file.path();

            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            else {
                continue;
            };

            let Some("world") = path.extension().and_then(|ext| ext.to_str())
            else {
                continue;
            };

            info!("loading: {}", path.display());

            let result: Result<()> = try {
                let id = stem
                    .parse()
                    .context("couldn't extract world id from path")?;

                let handle = kartoffels_world::resume(id, &path)?;

                entries.insert(
                    id,
                    WorldEntry {
                        ty: WorldType::Public,
                        handle: Some(handle),
                    },
                );
            };

            result.with_context(|| {
                format!("couldn't load world: {}", path.display())
            })?;
        }

        Ok(entries)
    }

    pub fn set(&self, handles: impl IntoIterator<Item = WorldHandle>) {
        let entries = handles
            .into_iter()
            .map(|handle| {
                let key = handle.id();

                let val = WorldEntry {
                    ty: WorldType::Public,
                    handle: Some(handle),
                };

                (key, val)
            })
            .collect();

        let public_idx = build_public_idx(&entries);

        self.entries.swap(Arc::new(entries));
        self.public_idx.swap(Arc::new(public_idx));
    }

    pub fn create(
        &self,
        testing: bool,
        dir: Option<&Path>,
        ty: WorldType,
        config: WorldConfig,
    ) -> Result<WorldHandle> {
        debug!(?dir, ?ty, ?config, "creating world");

        assert!(config.id.is_none());
        assert!(config.path.is_none());

        let id = self.create_alloc(testing, ty, &config)?;
        let config = self.create_config(dir, ty, config, id);
        let handle = self.create_spawn(ty, config);

        self.create_reindex(ty, &handle);

        info!(?id, ?ty, "world created");

        Ok(handle)
    }

    fn create_alloc(
        &self,
        testing: bool,
        ty: WorldType,
        config: &WorldConfig,
    ) -> Result<Id> {
        let mut id = None;

        self.entries.try_rcu(|entries| {
            if let WorldType::Public = ty {
                if entries
                    .values()
                    .filter_map(|entry| entry.handle.as_ref())
                    .any(|entry| entry.name() == config.name)
                {
                    return Err(anyhow!(
                        "world named `{}` already exists",
                        config.name
                    ));
                }
            }

            if entries.len() >= Self::MAX_WORLDS {
                return Err(anyhow!(
                    "ouch, the server is currently overloaded"
                ));
            }

            let mut entries = (**entries).clone();

            id = Some(loop {
                let id = if testing {
                    Id::new(self.test_next_id.fetch_add(1, Ordering::Relaxed))
                } else {
                    rand::random()
                };

                if let hash_map::Entry::Vacant(entry) = entries.entry(id) {
                    entry.insert(WorldEntry { ty, handle: None });

                    break id;
                }
            });

            Ok(entries)
        })?;

        Ok(id.unwrap())
    }

    fn create_config(
        &self,
        dir: Option<&Path>,
        ty: WorldType,
        mut config: WorldConfig,
        id: Id,
    ) -> WorldConfig {
        config.id = Some(id);

        if let WorldType::Public = ty {
            config.path = dir.map(|dir| path(dir, id));
        }

        config
    }

    fn create_spawn(&self, ty: WorldType, config: WorldConfig) -> WorldHandle {
        let id = config.id.unwrap();
        let handle = kartoffels_world::create(config);

        self.entries.rcu(|entries| {
            let mut entries = (**entries).clone();

            entries.get_mut(&id).unwrap().handle = Some(handle.clone());
            entries
        });

        match ty {
            WorldType::Public => handle,

            WorldType::Private => handle.on_last_drop({
                let entries = self.entries.clone();

                move || {
                    info!(?id, "world abandoned");

                    entries.rcu(|entries| {
                        let mut entries = (**entries).clone();

                        entries.remove(&id);
                        entries
                    });
                }
            }),
        }
    }

    fn create_reindex(&self, ty: WorldType, handle: &WorldHandle) {
        if let WorldType::Public = ty {
            self.public_idx.rcu(|handles| {
                let mut handles = (**handles).clone();

                handles.push(handle.clone());
                handles.sort_by(|a, b| a.name().cmp(b.name()));
                handles
            });
        }
    }

    pub async fn delete(&self, dir: Option<&Path>, id: Id) -> Result<()> {
        debug!(?dir, ?id, "deleting world");

        let entry = self.delete_remove(id)?;

        self.delete_cleanup(dir, id, entry).await?;

        debug!(?id, "world deleted");

        Ok(())
    }

    fn delete_remove(&self, id: Id) -> Result<WorldEntry> {
        let mut entry = None;

        _ = self.public_idx.try_rcu(|entries| {
            if let Some((idx, _)) =
                entries.iter().find_position(|entry| entry.id() == id)
            {
                let mut entries = (**entries).clone();

                entries.remove(idx);

                Ok(entries)
            } else {
                // No need to update the index (not really an error, it's just
                // that we don't have a better-named ArcSwap combinator at hand)
                Err(())
            }
        });

        self.entries.try_rcu(|entries| -> Result<_> {
            let mut entries = (**entries).clone();

            entry = Some(
                entries
                    .remove(&id)
                    .with_context(|| format!("couldn't find world `{id}`"))?,
            );

            Ok(entries)
        })?;

        Ok(entry.unwrap())
    }

    async fn delete_cleanup(
        &self,
        dir: Option<&Path>,
        id: Id,
        entry: WorldEntry,
    ) -> Result<()> {
        if let Some(handle) = entry.handle {
            handle.shutdown().await?;
        }

        if let WorldType::Public = entry.ty
            && let Some(dir) = dir
        {
            let path = path(dir, id);

            fs::remove_file(&path).await.with_context(|| {
                format!("couldn't remove world's file `{}`", path.display())
            })?;
        }

        Ok(())
    }

    pub fn all(&self) -> Vec<(WorldType, WorldHandle)> {
        self.entries
            .load()
            .values()
            .filter_map(|entry| Some((entry.ty, entry.handle.clone()?)))
            .collect()
    }

    pub fn public(&self) -> Arc<Vec<WorldHandle>> {
        self.public_idx.load_full()
    }

    pub fn first_private(&self) -> Option<WorldHandle> {
        self.entries
            .load()
            .values()
            .filter(|entry| matches!(entry.ty, WorldType::Private))
            .filter_map(|entry| entry.handle.clone())
            .next()
    }

    pub async fn shutdown(&self) -> Result<()> {
        for entry in self.entries.load().values() {
            if let WorldType::Public = entry.ty
                && let Some(handle) = &entry.handle
            {
                handle.shutdown().await?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct WorldEntry {
    ty: WorldType,
    handle: Option<WorldHandle>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldType {
    Public,
    Private,
}

fn path(dir: &Path, id: Id) -> PathBuf {
    dir.join(id.to_string()).with_extension("world")
}

fn build_public_idx(entries: &AHashMap<Id, WorldEntry>) -> Vec<WorldHandle> {
    entries
        .values()
        .filter(|entry| matches!(entry.ty, WorldType::Public))
        .filter_map(|entry| entry.handle.clone())
        .sorted_by(|a, b| a.name().cmp(b.name()))
        .collect()
}
