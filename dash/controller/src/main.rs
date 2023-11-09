#![recursion_limit = "256"]

mod ctx;
mod validator;

use ark_core_k8s::manager::Ctx;
use tokio::join;

pub(crate) mod consts {
    pub const NAME: &str = "dash-controller";
}

#[tokio::main]
async fn main() {
    join!(
        self::ctx::function::Ctx::spawn_crd(),
        self::ctx::injectors::otlp::Ctx::spawn(),
        self::ctx::job::Ctx::spawn_crd(),
        self::ctx::model::Ctx::spawn_crd(),
        self::ctx::model_storage_binding::Ctx::spawn_crd(),
        self::ctx::storage::Ctx::spawn_crd(),
        self::ctx::task::Ctx::spawn_crd(),
    );
}
