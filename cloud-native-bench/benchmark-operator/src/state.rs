use std::sync::Arc;

use chrono::{DateTime, Utc};
use kube::Client;
use serde::Serialize;
use tokio::sync::RwLock;

/// Shared state data between web server, reconciler and gRPC server.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateData {
    pub last_event_time: DateTime<Utc>,
    pub benchmark_name: String,
    pub namespace: String,
    pub is_benchmark_running: bool,
    pub is_benchmark_done: bool,
}

impl Default for StateData {
    fn default() -> Self {
        Self {
            last_event_time: Utc::now(),
            benchmark_name: Default::default(),
            namespace: Default::default(),
            is_benchmark_running: false,
            is_benchmark_done: false,
        }
    }
}

impl StateData {
    pub fn new_benchmark(&mut self, benchmark_name: String, namespace: String) {
        self.last_event_time = Utc::now();
        self.benchmark_name = benchmark_name;
        self.namespace = namespace;
        self.is_benchmark_running = false;
        self.is_benchmark_done = false;
    }

    pub fn clear_state(&mut self) {
        self.new_benchmark(Default::default(), Default::default())
    }
}

/// Context for the reconciler.
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client.
    pub k8s_client: Client,

    /// Shared state between all the components of the operator.
    pub state_data: Arc<RwLock<StateData>>,
}

impl Context {
    pub async fn set_is_benchmark_running(&self, is_running: bool) {
        let mut state_data = self.state_data.write().await;
        state_data.last_event_time = Utc::now();
        state_data.is_benchmark_running = is_running;
    }

    pub async fn set_is_benchmark_done(&self, is_done: bool) {
        let mut state_data = self.state_data.write().await;
        state_data.last_event_time = Utc::now();
        state_data.is_benchmark_done = is_done;
    }
}

/// Shared state wrapper for the webserver and gRPC server.
#[derive(Clone, Default)]
pub struct State {
    /// Shared state between all the components of the operator.
    pub state_data: Arc<RwLock<StateData>>,
}

impl State {
    pub fn create_context(&self, k8s_client: Client) -> Arc<Context> {
        Arc::new(Context {
            k8s_client,
            state_data: self.state_data.clone(),
        })
    }
}
