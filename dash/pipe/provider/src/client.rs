use anyhow::Result;
use ark_core_k8s::data::Name;
use clap::Parser;
use derivative::Derivative;
use serde_json::Value;

use crate::{
    message::{Codec, PipeMessage},
    messengers::{init_messenger, Messenger, MessengerArgs},
    storage::{MetadataStorageArgs, MetadataStorageType, StorageArgs, StorageSet},
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PipeClient {
    encoder: Codec,

    #[derivative(Debug = "ignore")]
    messenger: Box<dyn Messenger<Value>>,

    #[derivative(Debug = "ignore")]
    storage: StorageSet,
}

impl PipeClient {
    pub async fn try_default() -> ::anyhow::Result<Self> {
        let args = PipeClientArgs::try_parse()?;
        Self::try_new(&args).await
    }

    pub async fn try_new(args: &PipeClientArgs) -> ::anyhow::Result<Self> {
        let default_metadata_type = MetadataStorageType::default();
        let encoder = Codec::default();

        Ok(Self {
            encoder,
            messenger: init_messenger(&args.messenger).await?,
            storage: StorageSet::try_new(
                &args.storage,
                None,
                None,
                MetadataStorageArgs::<Value>::new(default_metadata_type),
            )
            .await?,
        })
    }

    pub async fn call(&self, topic: Name, data: PipeMessage) -> Result<PipeMessage<Value, ()>> {
        let data = data
            .dump_payloads(&self.storage, None)
            .await?
            .to_bytes(self.encoder)?;

        self.messenger
            .publish(topic)
            .await?
            .request_one(data)
            .await?
            .try_into()
    }

    pub async fn read(&self, topic: Name) -> Result<Option<PipeMessage<Value, ()>>> {
        self.messenger.subscribe(topic).await?.read_one().await
    }
}

#[derive(Debug, Parser)]
pub struct PipeClientArgs {
    #[command(flatten)]
    pub messenger: MessengerArgs,

    #[command(flatten)]
    pub storage: StorageArgs,
}