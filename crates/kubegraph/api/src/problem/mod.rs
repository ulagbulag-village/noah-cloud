pub mod r#virtual;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::graph::GraphMetadataRaw;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
    CustomResource,
)]
#[kube(
    group = "kubegraph.ulagbulag.io",
    version = "v1alpha1",
    kind = "NetworkProblem",
    root = "NetworkProblemCrd",
    shortname = "np",
    namespaced,
    printcolumn = r#"{
        "name": "created-at",
        "type": "date",
        "description": "created time",
        "jsonPath": ".metadata.creationTimestamp"
    }"#,
    printcolumn = r#"{
        "name": "version",
        "type": "integer",
        "description": "problem version",
        "jsonPath": ".metadata.generation"
    }"#
)]
#[schemars(bound = "M: Default + JsonSchema")]
#[serde(
    rename_all = "camelCase",
    bound = "M: Default + Serialize + DeserializeOwned"
)]
pub struct ProblemSpec<M = GraphMetadataRaw> {
    #[serde(default)]
    pub metadata: M,

    #[serde(default = "ProblemSpec::<M>::default_verbose")]
    pub verbose: bool,
}

impl<M> Default for ProblemSpec<M>
where
    M: Default,
{
    fn default() -> Self {
        Self {
            metadata: M::default(),
            verbose: Self::default_verbose(),
        }
    }
}

impl<M> ProblemSpec<M> {
    pub const MAX_CAPACITY: u64 = u64::MAX >> 32;

    const fn default_verbose() -> bool {
        false
    }
}
