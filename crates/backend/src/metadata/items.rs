use std::{
    borrow::Cow, path::{Path, PathBuf}, sync::Arc
};

use reqwest::RequestBuilder;
use schema::{
    assets_index::AssetsIndex, fabric_launch::FabricLaunch, fabric_loader_manifest::{FabricLoaderManifest, FABRIC_LOADER_MANIFEST_URL}, java_runtime_component::JavaRuntimeComponentManifest, java_runtimes::{JavaRuntimes, JAVA_RUNTIMES_URL}, maven::MavenMetadataXml, modrinth::{ModrinthLoader, ModrinthProjectVersion, ModrinthProjectVersionsRequest, ModrinthProjectVersionsResult, ModrinthSearchRequest, ModrinthSearchResult, ModrinthVersionFileUpdateResult, MODRINTH_SEARCH_URL}, version::MinecraftVersion, version_manifest::{MinecraftVersionLink, MinecraftVersionManifest, MOJANG_VERSION_MANIFEST_URL}
};
use serde::Serialize;
use ustr::Ustr;

use crate::metadata::manager::{MetaLoadError, MetaLoadStateWrapper, MetadataManager, MetadataManagerStates};

pub trait MetadataItem {
    type T: Send + Sync + 'static;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder;
    fn expires(&self) -> bool;
    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T>;
    fn post_process_download(bytes: &[u8]) -> Result<Cow<'_, [u8]>, MetaLoadError> {
        Ok(Cow::Borrowed(bytes))
    }
    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError>;
    fn cache_file(&self, _metadata_manager: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        None::<PathBuf>
    }
    fn data_hash(&self) -> Option<Ustr> {
        None
    }
}

pub struct MinecraftVersionManifestMetadataItem;

impl MetadataItem for MinecraftVersionManifestMetadataItem {
    type T = MinecraftVersionManifest;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get(MOJANG_VERSION_MANIFEST_URL)
    }

    fn expires(&self) -> bool {
        true
    }

    fn cache_file(&self, metadata_manager: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        Some(Arc::clone(&metadata_manager.version_manifest_cache))
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.minecraft_version_manifest.clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct MojangJavaRuntimesMetadataItem;

impl MetadataItem for MojangJavaRuntimesMetadataItem {
    type T = JavaRuntimes;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get(JAVA_RUNTIMES_URL)
    }

    fn expires(&self) -> bool {
        true
    }

    fn cache_file(&self, metadata_manager: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        Some(Arc::clone(&metadata_manager.mojang_java_runtimes_cache))
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.mojang_java_runtimes.clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct MinecraftVersionMetadataItem<'v>(pub &'v MinecraftVersionLink);

impl<'v> MetadataItem for MinecraftVersionMetadataItem<'v> {
    type T = MinecraftVersion;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get(self.0.url.as_str())
    }

    fn expires(&self) -> bool {
        false
    }

    fn data_hash(&self) -> Option<Ustr> {
        Some(self.0.sha1)
    }

    fn cache_file(&self, metadata_manager: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        if !crate::is_single_component_path(&self.0.sha1) {
            panic!("Invalid sha1 {}, possible directory traversal attack?", self.0.sha1);
        }
        let mut path = metadata_manager.metadata_cache.join("version_info");
        path.push(self.0.sha1.as_str());
        Some(path)
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.version_info.entry(self.0.url).or_default().clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct AssetsIndexMetadataItem {
    pub url: Ustr,
    pub cache: Arc<Path>,
    pub hash: Ustr,
}

impl MetadataItem for AssetsIndexMetadataItem {
    type T = AssetsIndex;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get(self.url.as_str())
    }

    fn expires(&self) -> bool {
        false
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.assets_index.entry(self.url).or_default().clone()
    }

    fn cache_file(&self, _: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        Some(Arc::clone(&self.cache))
    }

    fn data_hash(&self) -> Option<Ustr> {
        Some(self.hash)
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct MojangJavaRuntimeComponentMetadataItem {
    pub url: Ustr,
    pub cache: Arc<Path>,
    pub hash: Ustr,
}

impl MetadataItem for MojangJavaRuntimeComponentMetadataItem {
    type T = JavaRuntimeComponentManifest;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get(self.url.as_str())
    }

    fn expires(&self) -> bool {
        false
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.java_runtime_manifests.entry(self.url).or_default().clone()
    }

    fn cache_file(&self, _: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        Some(Arc::clone(&self.cache))
    }

    fn data_hash(&self) -> Option<Ustr> {
        Some(self.hash)
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct FabricLoaderManifestMetadataItem;

impl MetadataItem for FabricLoaderManifestMetadataItem {
    type T = FabricLoaderManifest;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get(FABRIC_LOADER_MANIFEST_URL)
    }

    fn expires(&self) -> bool {
        true
    }

    fn cache_file(&self, metadata_manager: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        Some(Arc::clone(&metadata_manager.fabric_loader_manifest_cache))
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.fabric_loader_manifest.clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct FabricLaunchMetadataItem {
    pub minecraft_version: Ustr,
    pub loader_version: Ustr,
}

impl MetadataItem for FabricLaunchMetadataItem {
    type T = FabricLaunch;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get(format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}", self.minecraft_version, self.loader_version))
    }

    fn expires(&self) -> bool {
        false
    }

    fn cache_file(&self, metadata_manager: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        let mut path = metadata_manager.metadata_cache.join("fabric_launch");
        path.push(self.minecraft_version.as_str());
        path.push(self.loader_version.as_str());
        Some(path)
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        let key = (self.minecraft_version, self.loader_version);
        states.fabric_launch.entry(key).or_default().clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct ModrinthSearchMetadataItem<'a>(pub &'a ModrinthSearchRequest);

impl<'a> MetadataItem for ModrinthSearchMetadataItem<'a> {
    type T = ModrinthSearchResult;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get(MODRINTH_SEARCH_URL).query(self.0)
    }

    fn expires(&self) -> bool {
        true
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.modrinth_search.entry(self.0.clone()).or_default().clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct ModrinthProjectVersionsMetadataItem<'a>(pub &'a ModrinthProjectVersionsRequest);

impl<'a> MetadataItem for ModrinthProjectVersionsMetadataItem<'a> {
    type T = ModrinthProjectVersionsResult;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        let url = format!("https://api.modrinth.com/v2/project/{}/version", self.0.project_id);
        let mut request = client.get(url);
        if let Some(loaders) = &self.0.loaders && let Ok(str) = serde_json::to_string(loaders) {
            request = request.query(&[("loaders", str)]);
        }
        if let Some(game_versions) = &self.0.game_versions && let Ok(str) = serde_json::to_string(game_versions) {
            request = request.query(&[("game_versions", str)]);
        }
        request
    }

    fn expires(&self) -> bool {
        true
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.modrinth_project_versions.entry(self.0.project_id.clone()).or_default().clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct ModrinthVersionMetadataItem(pub Arc<str>);

impl MetadataItem for ModrinthVersionMetadataItem {
    type T = ModrinthProjectVersion;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        let url = format!("https://api.modrinth.com/v2/version/{}", self.0);
        client.get(url)
    }

    fn expires(&self) -> bool {
        true
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.modrinth_versions.entry(self.0.clone()).or_default().clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct VersionUpdateParameters {
    pub loaders: Arc<[ModrinthLoader]>,
    pub game_versions: Arc<[Ustr]>,
}

pub struct ModrinthVersionUpdateMetadataItem {
    pub sha1: Arc<str>,
    pub params: VersionUpdateParameters,
}

impl MetadataItem for ModrinthVersionUpdateMetadataItem {
    type T = ModrinthVersionFileUpdateResult;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        let url = format!("https://api.modrinth.com/v2/version_file/{}/update", self.sha1);
        client.post(url).json(&self.params)
    }

    fn expires(&self) -> bool {
        true
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.modrinth_version_updates.entry(self.sha1.clone()).or_default().clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct VersionV3UpdateParameters {
    pub loaders: Arc<[Arc<str>]>,
    pub loader_fields: VersionV3LoaderFields,
}

#[derive(Clone, Debug, Serialize)]
pub struct VersionV3LoaderFields {
    pub mrpack_loaders: Arc<[ModrinthLoader]>,
    pub game_versions: Arc<[Ustr]>,
}

pub struct ModrinthV3VersionUpdateMetadataItem {
    pub sha1: Arc<str>,
    pub params: VersionV3UpdateParameters,
}

impl MetadataItem for ModrinthV3VersionUpdateMetadataItem {
    type T = ModrinthVersionFileUpdateResult;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        let url = format!("https://api.modrinth.com/v3/version_file/{}/update", self.sha1);
        client.post(url).json(&self.params)
    }

    fn expires(&self) -> bool {
        true
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.modrinth_version_updates.entry(self.sha1.clone()).or_default().clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

pub struct NeoforgeInstallerMavenMetadataItem;

impl MetadataItem for NeoforgeInstallerMavenMetadataItem {
    type T = MavenMetadataXml;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get("https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml")
    }

    fn expires(&self) -> bool {
        true
    }

    fn cache_file(&self, metadata_manager: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        Some(Arc::clone(&metadata_manager.neoforge_installer_maven_cache))
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.neoforge_installer_maven_manifest.clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_xml_rs::from_reader(bytes)?)
    }
}

pub struct ForgeInstallerMavenMetadataItem;

impl MetadataItem for ForgeInstallerMavenMetadataItem {
    type T = MavenMetadataXml;

    fn request(&self, client: &reqwest::Client) -> RequestBuilder {
        client.get("https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml")
    }

    fn expires(&self) -> bool {
        true
    }

    fn cache_file(&self, metadata_manager: &MetadataManager) -> Option<impl AsRef<Path> + Send + Sync + 'static> {
        Some(Arc::clone(&metadata_manager.forge_installer_maven_cache))
    }

    fn state(&self, states: &mut MetadataManagerStates) -> MetaLoadStateWrapper<Self::T> {
        states.forge_installer_maven_manifest.clone()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::T, MetaLoadError> {
        Ok(serde_xml_rs::from_reader(bytes)?)
    }
}
