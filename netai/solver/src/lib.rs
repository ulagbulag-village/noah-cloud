mod io;
mod models;
mod primitive;
mod role;
pub mod session;
mod tensor;

type BoxSolver = Box<dyn Solver + Send + Sync>;

#[::async_trait::async_trait(?Send)]
trait Solver {
    async fn solve(
        &self,
        session: &crate::session::Session,
        tensors: crate::tensor::BatchedTensor,
    ) -> ::anyhow::Result<crate::tensor::BatchedTensor>;

    async fn solve_web(
        &self,
        session: &crate::session::Session,
        request: crate::io::Request,
    ) -> ::anyhow::Result<crate::io::Response>;
}
