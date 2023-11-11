use std::{sync::Arc, time::Duration};

use anyhow::Result;
use ark_core_k8s::manager::{Manager, TryDefault};
use async_trait::async_trait;
use chrono::Utc;
use dash_api::model_claim::{ModelClaimCrd, ModelClaimSpec, ModelClaimState, ModelClaimStatus};
use dash_optimizer_client::OptimizerClient;
use dash_provider::storage::KubernetesStorageClient;
use kube::{
    api::{Patch, PatchParams},
    runtime::controller::Action,
    Api, Client, CustomResourceExt, Error, ResourceExt,
};
use serde_json::json;
use tracing::{info, instrument, warn, Level};

use crate::validator::model_claim::ModelClaimValidator;

pub struct Ctx {
    optimizer: OptimizerClient,
}

#[async_trait]
impl TryDefault for Ctx {
    async fn try_default() -> Result<Self> {
        Ok(Self {
            optimizer: OptimizerClient::try_default().await?,
        })
    }
}

#[async_trait]
impl ::ark_core_k8s::manager::Ctx for Ctx {
    type Data = ModelClaimCrd;

    const NAME: &'static str = crate::consts::NAME;
    const NAMESPACE: &'static str = ::dash_api::consts::NAMESPACE;
    const FALLBACK: Duration = Duration::from_secs(30); // 30 seconds
    const FINALIZER_NAME: &'static str =
        <Self as ::ark_core_k8s::manager::Ctx>::Data::FINALIZER_NAME;

    #[instrument(level = Level::INFO, skip_all, fields(name = data.name_any(), namespace = data.namespace()), err(Display))]
    async fn reconcile(
        manager: Arc<Manager<Self>>,
        data: Arc<<Self as ::ark_core_k8s::manager::Ctx>::Data>,
    ) -> Result<Action, Error>
    where
        Self: Sized,
    {
        let name = data.name_any();
        let namespace = data.namespace().unwrap();

        if data.metadata.deletion_timestamp.is_some()
            && data
                .status
                .as_ref()
                .map(|status| status.state != ModelClaimState::Deleting)
                .unwrap_or(true)
        {
            let status = data.status.as_ref();
            return Self::update_fields_or_requeue(
                &namespace,
                &manager.kube,
                &name,
                status.and_then(|status| status.spec.clone()),
                ModelClaimState::Deleting,
            )
            .await;
        } else if !data
            .finalizers()
            .iter()
            .any(|finalizer| finalizer == <Self as ::ark_core_k8s::manager::Ctx>::FINALIZER_NAME)
        {
            return <Self as ::ark_core_k8s::manager::Ctx>::add_finalizer_or_requeue_namespaced(
                manager.kube.clone(),
                &namespace,
                &name,
            )
            .await;
        }

        let validator = ModelClaimValidator {
            kubernetes_storage: KubernetesStorageClient {
                namespace: &namespace,
                kube: &manager.kube,
            },
            optimizer: &manager.ctx.optimizer,
        };

        match data
            .status
            .as_ref()
            .map(|status| status.state)
            .unwrap_or_default()
        {
            ModelClaimState::Pending | ModelClaimState::Replacing => match validator
                .validate_model_claim(<Self as ::ark_core_k8s::manager::Ctx>::NAME, &data)
                .await
            {
                Ok(()) => {
                    Self::update_fields_or_requeue(
                        &namespace,
                        &manager.kube,
                        &name,
                        Some(data.spec.clone()),
                        ModelClaimState::Ready,
                    )
                    .await
                }
                Err(e) => {
                    warn!("failed to validate model claim: {name:?}: {e}");
                    Ok(Action::requeue(
                        <Self as ::ark_core_k8s::manager::Ctx>::FALLBACK,
                    ))
                }
            },
            ModelClaimState::Ready => {
                // TODO: implement to finding changes
                Ok(Action::await_change())
            }
            ModelClaimState::Deleting => match validator.delete(&data).await {
                Ok(()) => {
                    <Self as ::ark_core_k8s::manager::Ctx>::remove_finalizer_or_requeue_namespaced(
                        manager.kube.clone(),
                        &namespace,
                        &name,
                    )
                    .await
                }
                Err(e) => {
                    warn!("failed to delete model claim ({namespace}/{name}): {e}");
                    Ok(Action::requeue(
                        <Self as ::ark_core_k8s::manager::Ctx>::FALLBACK,
                    ))
                }
            },
        }
    }
}

impl Ctx {
    #[instrument(level = Level::INFO, skip_all, err(Display))]
    async fn update_fields_or_requeue(
        namespace: &str,
        kube: &Client,
        name: &str,
        spec: Option<ModelClaimSpec>,
        state: ModelClaimState,
    ) -> Result<Action, Error> {
        match Self::update_fields(namespace, kube, name, spec, state).await {
            Ok(()) => {
                info!("model claim is ready: {namespace}/{name}");
                Ok(Action::requeue(
                    <Self as ::ark_core_k8s::manager::Ctx>::FALLBACK,
                ))
            }
            Err(e) => {
                warn!("failed to validate model claim ({namespace}/{name}): {e}");
                Ok(Action::requeue(
                    <Self as ::ark_core_k8s::manager::Ctx>::FALLBACK,
                ))
            }
        }
    }

    #[instrument(level = Level::INFO, skip(kube, spec), err(Display))]
    async fn update_fields(
        namespace: &str,
        kube: &Client,
        name: &str,
        spec: Option<ModelClaimSpec>,
        state: ModelClaimState,
    ) -> Result<()> {
        let api = Api::<<Self as ::ark_core_k8s::manager::Ctx>::Data>::namespaced(
            kube.clone(),
            namespace,
        );
        let crd = <Self as ::ark_core_k8s::manager::Ctx>::Data::api_resource();

        let patch = Patch::Merge(json!({
            "apiVersion": crd.api_version,
            "kind": crd.kind,
            "status": ModelClaimStatus {
                state,
                spec,
                last_updated: Utc::now(),
            },
        }));
        let pp = PatchParams::apply(<Self as ::ark_core_k8s::manager::Ctx>::NAME);
        api.patch_status(name, &pp, &patch).await?;
        Ok(())
    }
}
