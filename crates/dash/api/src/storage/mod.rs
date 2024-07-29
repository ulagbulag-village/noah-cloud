pub mod db;
pub mod kubernetes;
pub mod object;

use ark_core_k8s::data::Url;
use chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, CustomResource)]
#[kube(
    group = "dash.ulagbulag.io",
    version = "v1alpha1",
    kind = "ModelStorage",
    root = "ModelStorageCrd",
    status = "ModelStorageStatus",
    shortname = "ms",
    namespaced,
    printcolumn = r#"{
        "name": "state",
        "type": "string",
        "description": "state of the model storage",
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
pub struct ModelStorageSpec {
    #[serde(flatten)]
    pub kind: ModelStorageKindSpec,
    #[serde(default)]
    pub default: bool,
}

impl ModelStorageCrd {
    pub const FINALIZER_NAME: &'static str = "dash.ulagbulag.io/finalizer-model-storages";
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ModelStorageKindSpec {
    Database(#[serde(default)] self::db::ModelStorageDatabaseSpec),
    Kubernetes(#[serde(default)] self::kubernetes::ModelStorageKubernetesSpec),
    ObjectStorage(#[serde(default)] self::object::ModelStorageObjectSpec),
}

impl ModelStorageKindSpec {
    #[inline]
    pub fn endpoint(&self, namespace: &str) -> Option<Url> {
        match self {
            Self::Database(spec) => spec.endpoint(),
            Self::Kubernetes(spec) => spec.endpoint(),
            Self::ObjectStorage(spec) => spec.endpoint(namespace),
        }
    }

    pub const fn is_unique(&self) -> bool {
        match self {
            Self::Database(_) => false,
            Self::Kubernetes(_) => true,
            Self::ObjectStorage(spec) => spec.is_unique(),
        }
    }

    pub const fn to_kind(&self) -> ModelStorageKind {
        match self {
            Self::Database(_) => ModelStorageKind::Database,
            Self::Kubernetes(_) => ModelStorageKind::Kubernetes,
            Self::ObjectStorage(_) => ModelStorageKind::ObjectStorage,
        }
    }
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
pub enum ModelStorageKind {
    Database,
    Kubernetes,
    ObjectStorage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelStorageStatus {
    #[serde(default)]
    pub state: ModelStorageState,
    pub kind: Option<ModelStorageKindSpec>,
    pub last_updated: DateTime<Utc>,
    #[serde(default)]
    pub total_quota: Option<u128>,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Display,
    Default,
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
pub enum ModelStorageState {
    #[default]
    Pending,
    Ready,
    Deleting,
}
