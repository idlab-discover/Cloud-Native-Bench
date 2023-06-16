use benchmark_proto::protos::benchmark::{
    benchmark_service_client::BenchmarkServiceClient, BenchmarkDoneRequest, BenchmarkStartRequest,
};
use std::error::Error;
use tonic::{transport::Channel, Status};

pub struct GrpcCall {
    pub grpc_client: BenchmarkServiceClient<Channel>,
}

impl GrpcCall {
    /// Connect to the operator gRPC server.
    pub async fn connect(endpoint: String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            grpc_client: BenchmarkServiceClient::connect(endpoint).await?,
        })
    }

    /// Lets the operator know the Benchmark has started.
    /// Returns the database connection URL string that the Benchmark should use to save its results.
    pub async fn benchmark_started(&mut self) -> Result<String, Status> {
        let request = tonic::Request::new(BenchmarkStartRequest { running: true });
        let response = self.grpc_client.benchmark_started(request).await?;

        Ok(response.into_inner().database_connection_string)
    }

    /// Lets the operator know the Benchmark has finished and all the results are saved to the database.
    /// Returns a boolean where the operator acknowledges that the Benchmark is done.
    pub async fn benchmark_done(&mut self) -> Result<bool, Status> {
        let request = tonic::Request::new(BenchmarkDoneRequest { done: true });
        let response = self.grpc_client.benchmark_done(request).await?;

        Ok(response.into_inner().acknowledge)
    }
}
