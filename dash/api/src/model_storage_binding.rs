use chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{model::ModelSpec, storage::ModelStorageSpec};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, CustomResource)]
#[kube(
    group = "dash.ulagbulag.io",
    version = "v1alpha1",
    kind = "ModelStorageBinding",
    struct = "ModelStorageBindingCrd",
    status = "ModelStorageBindingStatus",
    shortname = "msb",
    namespaced,
    printcolumn = r#"{
        "name": "state",
        "type": "string",
        "description": "state of the binding",
        "jsonPath": ".status.state"
    }"#,
    printcolumn = r#"{
        "name": "created-at",
        "type": "date",
        "description": "created time",
        "jsonPath": ".metadata.creationTimestamp"
    }"#,
    printcolumn = r#"{
        "name": "updated-at",
        "type": "date",
        "description": "updated time",
        "jsonPath": ".status.lastUpdated"
    }"#
)]
#[serde(rename_all = "camelCase")]
pub struct ModelStorageBindingSpec {
    pub model: String,
    pub storage: ModelStorageBindingStorageKind<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ModelStorageBindingStorageKind<Storage> {
    Cloned(ModelStorageBindingStorageKindClonedSpec<Storage>),
    Owned(ModelStorageBindingStorageKindOwnedSpec<Storage>),
}

impl<Storage> ModelStorageBindingStorageKind<Storage> {
    pub fn source(&self) -> Option<(&Storage, ModelStorageBindingSyncPolicy)> {
        match self {
            Self::Cloned(spec) => Some((&spec.source, spec.sync_policy)),
            Self::Owned(_) => None,
        }
    }

    pub fn target(&self) -> &Storage {
        match self {
            Self::Cloned(spec) => &spec.target,
            Self::Owned(spec) => &spec.target,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelStorageBindingStorageKindClonedSpec<Storage> {
    pub source: Storage,
    pub target: Storage,
    #[serde(default)]
    pub sync_policy: ModelStorageBindingSyncPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelStorageBindingStorageSpec<'name, Storage> {
    pub source: Option<(&'name str, Storage, ModelStorageBindingSyncPolicy)>,
    pub target: Storage,
    pub target_name: &'name str,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelStorageBindingStorageKindOwnedSpec<Storage> {
    pub target: Storage,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub enum ModelStorageBindingSyncPolicy {
    #[serde(alias = "ModelPeering", alias = "DatasetPeering")]
    Always,
    Never,
    #[serde(alias = "ModelTiering", alias = "DatasetTiering")]
    OnDelete,
}

impl Default for ModelStorageBindingSyncPolicy {
    fn default() -> Self {
        Self::Never
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelStorageBindingStatus {
    #[serde(default)]
    pub state: ModelStorageBindingState,
    pub model: Option<ModelSpec>,
    pub storage: Option<ModelStorageBindingStorageKind<ModelStorageSpec>>,
    pub last_updated: DateTime<Utc>,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub enum ModelStorageBindingState {
    Pending,
    Ready,
}

impl Default for ModelStorageBindingState {
    fn default() -> Self {
        Self::Pending
    }
}
