use std::{sync::Arc, time::Duration};

use anyhow::Result;
use ark_core_k8s::manager::Manager;
use async_trait::async_trait;
use chrono::Utc;
use dash_api::{
    model::ModelSpec,
    model_storage_binding::{
        ModelStorageBindingCrd, ModelStorageBindingState, ModelStorageBindingStatus,
        ModelStorageBindingSyncPolicy,
    },
    storage::ModelStorageSpec,
};
use dash_provider::storage::KubernetesStorageClient;
use kube::{
    api::{Patch, PatchParams},
    runtime::controller::Action,
    Api, Client, CustomResourceExt, Error, ResourceExt,
};
use log::{info, warn};
use serde_json::json;

use crate::validator::{
    model::ModelValidator, model_storage_binding::ModelStorageBindingValidator,
    storage::ModelStorageValidator,
};

#[derive(Default)]
pub struct Ctx {}

#[async_trait]
impl ::ark_core_k8s::manager::Ctx for Ctx {
    type Data = ModelStorageBindingCrd;

    const NAME: &'static str = crate::consts::NAME;
    const NAMESPACE: &'static str = ::dash_api::consts::NAMESPACE;
    const FALLBACK: Duration = Duration::from_secs(30); // 30 seconds

    async fn reconcile(
        manager: Arc<Manager<Self>>,
        data: Arc<<Self as ::ark_core_k8s::manager::Ctx>::Data>,
    ) -> Result<Action, Error>
    where
        Self: Sized,
    {
        let name = data.name_any();
        let namespace = data.namespace().unwrap();

        match data
            .status
            .as_ref()
            .map(|status| status.state)
            .unwrap_or_default()
        {
            ModelStorageBindingState::Pending => {
                let kubernetes_storage = KubernetesStorageClient {
                    namespace: &namespace,
                    kube: &manager.kube,
                };
                let validator = ModelStorageBindingValidator {
                    model: ModelValidator { kubernetes_storage },
                    model_storage: ModelStorageValidator { kubernetes_storage },
                    sync_policy: data.spec.sync_policy,
                };
                match validator.validate_model_storage_binding(&data.spec).await {
                    Ok((model, storage, sync_policy)) => {
                        Self::update_state_or_requeue(
                            &namespace,
                            &manager.kube,
                            &name,
                            model,
                            storage,
                            sync_policy,
                        )
                        .await
                    }
                    Err(e) => {
                        warn!("failed to validate model storage binding: {name:?}: {e}");
                        Ok(Action::requeue(
                            <Self as ::ark_core_k8s::manager::Ctx>::FALLBACK,
                        ))
                    }
                }
            }
            ModelStorageBindingState::Ready => {
                // TODO: implement to finding changes
                Ok(Action::await_change())
            }
        }
    }
}

impl Ctx {
    async fn update_state_or_requeue(
        namespace: &str,
        kube: &Client,
        name: &str,
        model: ModelSpec,
        storage: ModelStorageSpec,
        sync_policy: ModelStorageBindingSyncPolicy,
    ) -> Result<Action, Error> {
        match Self::update_state(namespace, kube, name, model, storage, sync_policy).await {
            Ok(()) => {
                info!("model storage binding is ready: {namespace}/{name}");
                Ok(Action::requeue(
                    <Self as ::ark_core_k8s::manager::Ctx>::FALLBACK,
                ))
            }
            Err(e) => {
                warn!("failed to validate model storage binding ({namespace}/{name}): {e}");
                Ok(Action::requeue(
                    <Self as ::ark_core_k8s::manager::Ctx>::FALLBACK,
                ))
            }
        }
    }

    async fn update_state(
        namespace: &str,
        kube: &Client,
        name: &str,
        model: ModelSpec,
        storage: ModelStorageSpec,
        sync_policy: ModelStorageBindingSyncPolicy,
    ) -> Result<()> {
        let api = Api::<<Self as ::ark_core_k8s::manager::Ctx>::Data>::namespaced(
            kube.clone(),
            namespace,
        );
        let crd = <Self as ::ark_core_k8s::manager::Ctx>::Data::api_resource();

        let patch = Patch::Merge(json!({
            "apiVersion": crd.api_version,
            "kind": crd.kind,
            "status": ModelStorageBindingStatus {
                state: ModelStorageBindingState::Ready,
                model: Some(model),
                storage: Some(storage),
                sync_policy,
                last_updated: Utc::now(),
            },
        }));
        let pp = PatchParams::apply(<Self as ::ark_core_k8s::manager::Ctx>::NAME);
        api.patch_status(name, &pp, &patch).await?;
        Ok(())
    }
}
