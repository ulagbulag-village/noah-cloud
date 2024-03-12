use std::fmt;

use anyhow::Result;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkGraphRow {
    id: String,
    #[serde(flatten)]
    pub key: NetworkEdgeKey,
    pub value: NetworkValue,
}

impl NetworkGraphRow {
    pub fn try_new(key: NetworkEdgeKey, value: NetworkValue) -> Result<Self> {
        Ok(Self {
            id: {
                // read hash digest and consume hasher
                let hash = Sha256::digest(::serde_json::to_vec(&key)?);

                // encode to hex format
                format!("{hash:x}")
            },
            key,
            value,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(
    rename_all = "camelCase",
    bound = "
    NodeKey: Ord + Serialize + DeserializeOwned,
"
)]
pub struct NetworkEdgeKey<NodeKey = NetworkNodeKey>
where
    NodeKey: Ord,
{
    #[serde(rename = "le")]
    pub interval_ms: u64,
    #[serde(
        flatten,
        deserialize_with = "self::prefix::link::deserialize",
        serialize_with = "self::prefix::link::serialize"
    )]
    pub link: NodeKey,
    #[serde(
        flatten,
        deserialize_with = "self::prefix::sink::deserialize",
        serialize_with = "self::prefix::sink::serialize"
    )]
    pub sink: NodeKey,
    #[serde(
        flatten,
        deserialize_with = "self::prefix::src::deserialize",
        serialize_with = "self::prefix::src::serialize"
    )]
    pub src: NodeKey,
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct NetworkNodeKey {
    pub kind: String,
    pub name: String,
    pub namespace: String,
}

impl fmt::Display for NetworkNodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            kind,
            name,
            namespace,
        } = self;

        write!(f, "{kind}/{namespace}/{name}")
    }
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct NetworkValue(pub u64);

mod prefix {
    ::serde_with::with_prefix!(pub(super) link "link_");
    ::serde_with::with_prefix!(pub(super) sink "sink_");
    ::serde_with::with_prefix!(pub(super) src "src_");
}