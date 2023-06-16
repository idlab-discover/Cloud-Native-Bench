use crate::{
    benchmark_controller::{Benchmark, BenchmarkState},
    state::State,
};
use benchmark_proto::protos::benchmark::{
    benchmark_service_server::{BenchmarkService, BenchmarkServiceServer},
    BenchmarkDoneRequest, BenchmarkDoneResponse, BenchmarkStartRequest, BenchmarkStartedResponse,
};
use chrono::Utc;
use futures::FutureExt;
use kube::Client;
use std::env;
use tonic::{transport::Server, Code, Request, Response, Status};
use tracing::info;

pub struct BenchmarkGrpcService {
    state: State,
    k8s_client: Client,
}

impl BenchmarkGrpcService {
    pub async fn new(state: &State) -> BenchmarkGrpcService {
        let k8s_client = Client::try_default()
            .await
            .expect("gRPC: Failed to create kube client.");

        BenchmarkGrpcService {
            state: state.clone(),
            k8s_client,
        }
    }
}

#[tonic::async_trait]
impl BenchmarkService for BenchmarkGrpcService {
    async fn benchmark_started(
        &self,
        request: Request<BenchmarkStartRequest>,
    ) -> Result<Response<BenchmarkStartedResponse>, Status> {
        info!("Benchmark start request from {:?}.", request.remote_addr());

        let mut state_data = self.state.state_data.write().await;

        state_data.last_event_time = Utc::now();

        // Set Running state.
        Benchmark::set_benchmark_state(
            self.k8s_client.clone(),
            &state_data.benchmark_name,
            &state_data.namespace,
            BenchmarkState::Running,
        )
        .await
        .map_err(|e| Status::new(Code::FailedPrecondition, e.to_string()))?;

        Ok(Response::new(BenchmarkStartedResponse {
            database_connection_string: env::var("DATABASE_URL")
                .expect("DATABASE_URL environment variable was not set."),
        }))
    }

    async fn benchmark_done(
        &self,
        request: Request<BenchmarkDoneRequest>,
    ) -> Result<Response<BenchmarkDoneResponse>, Status> {
        info!("Benchmark done request from {:?}.", request.remote_addr());

        let mut state_data = self.state.state_data.write().await;

        state_data.last_event_time = Utc::now();

        // Set Done state.
        Benchmark::set_benchmark_state(
            self.k8s_client.clone(),
            &state_data.benchmark_name,
            &state_data.namespace,
            BenchmarkState::Done,
        )
        .await
        .map_err(|e| Status::new(Code::FailedPrecondition, e.to_string()))?;

        Ok(Response::new(BenchmarkDoneResponse { acknowledge: true }))
    }
}

pub async fn spawn(state: &State) {
    let addr = env::var("GRPC_SOCKET_ADDRESS")
        .expect("GRPC_SOCKET_ADDRESS environment variable was not set.")
        .parse()
        .unwrap();
    let benchmark_grpc_service = BenchmarkGrpcService::new(state).await;

    info!("Benchmark gRPC server listening on {}", addr);

    Server::builder()
        .add_service(BenchmarkServiceServer::new(benchmark_grpc_service))
        .serve_with_shutdown(addr, tokio::signal::ctrl_c().map(|_| ()))
        .await
        .unwrap()
}
