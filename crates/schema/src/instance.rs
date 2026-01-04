use std::sync::Arc;

use serde::{Deserialize, Serialize};
use ustr::Ustr;

use crate::loader::Loader;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceConfiguration {
    pub minecraft_version: Ustr,
    pub loader: Loader,
    #[serde(default, deserialize_with = "crate::try_deserialize", skip_serializing_if = "is_default_memory_configuration")]
    pub memory: Option<InstanceMemoryConfiguration>,
    #[serde(default, deserialize_with = "crate::try_deserialize", skip_serializing_if = "is_default_jvm_flags_configuration")]
    pub jvm_flags: Option<InstanceJvmFlagsConfiguration>
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct InstanceMemoryConfiguration {
    pub enabled: bool,
    pub min: u32,
    pub max: u32,
}

impl InstanceMemoryConfiguration {
    pub const DEFAULT_MIN: u32 = 512;
    pub const DEFAULT_MAX: u32 = 4096;
}

impl Default for InstanceMemoryConfiguration {
    fn default() -> Self {
        Self {
            enabled: false,
            min: Self::DEFAULT_MIN,
            max: Self::DEFAULT_MAX
        }
    }
}

fn is_default_memory_configuration(config: &Option<InstanceMemoryConfiguration>) -> bool {
    if let Some(config) = config {
        !config.enabled &&
            config.min == InstanceMemoryConfiguration::DEFAULT_MIN &&
            config.max == InstanceMemoryConfiguration::DEFAULT_MAX
    } else {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InstanceJvmFlagsConfiguration {
    pub enabled: bool,
    pub flags: Arc<str>,
}

fn is_default_jvm_flags_configuration(config: &Option<InstanceJvmFlagsConfiguration>) -> bool {
    if let Some(config) = config {
        !config.enabled && config.flags.trim_ascii().is_empty()
    } else {
        true
    }
}
