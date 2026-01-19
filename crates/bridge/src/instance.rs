use std::{collections::HashSet, path::Path, sync::Arc};

use schema::{content::ContentSource, modification::ModrinthModpackFileDownload};

use crate::safe_path::SafePath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceID {
    pub index: usize,
    pub generation: usize,
}

impl InstanceID {
    pub fn dangling() -> Self {
        Self {
            index: usize::MAX,
            generation: usize::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InstanceModID {
    pub index: usize,
    pub generation: usize,
}

impl InstanceModID {
    pub fn dangling() -> Self {
        Self {
            index: usize::MAX,
            generation: usize::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceStatus {
    NotRunning,
    Launching,
    Running,
}

#[derive(Debug, Clone)]
pub struct InstanceWorldSummary {
    pub title: Arc<str>,
    pub subtitle: Arc<str>,
    pub level_path: Arc<Path>,
    pub last_played: i64,
    pub png_icon: Option<Arc<[u8]>>,
}

#[derive(Debug, Clone)]
pub struct InstanceServerSummary {
    pub name: Arc<str>,
    pub ip: Arc<str>,
    pub png_icon: Option<Arc<[u8]>>,
}

#[derive(Debug, Clone)]
pub struct InstanceModSummary {
    pub mod_summary: Arc<ModSummary>,
    pub id: InstanceModID,
    pub filename: Arc<str>,
    pub lowercase_filename: Arc<str>,
    pub filename_hash: u64,
    pub path: Arc<Path>,
    pub enabled: bool,
    pub content_source: ContentSource,
    pub disabled_children: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct ModSummary {
    pub id: Arc<str>,
    pub hash: [u8; 20],
    pub name: Arc<str>,
    pub lowercase_search_key: Arc<str>,
    pub version_str: Arc<str>,
    pub authors: Arc<str>,
    pub png_icon: Option<Arc<[u8]>>,
    pub update_status: Arc<AtomicContentUpdateStatus>,
    pub extra: LoaderSpecificModSummary,
}

#[derive(Debug, Clone)]
pub enum LoaderSpecificModSummary {
    Fabric,
    Forge,
    JavaModule,
    ModrinthModpack {
        downloads: Arc<[ModrinthModpackFileDownload]>,
        summaries: Arc<[Option<Arc<ModSummary>>]>,
        overrides: Arc<[(SafePath, Arc<[u8]>)]>,
    },
}


#[atomic_enum::atomic_enum]
#[derive(PartialEq, Eq)]
pub enum ContentUpdateStatus {
    Unknown,
    ManualInstall,
    ErrorNotFound,
    ErrorInvalidHash,
    AlreadyUpToDate,
    Modrinth,
}

impl ContentUpdateStatus {
    pub fn can_update(&self) -> bool {
        match self {
            ContentUpdateStatus::Modrinth => true,
            _ => false,
        }
    }
}
