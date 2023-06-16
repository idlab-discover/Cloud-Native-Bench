use crate::state::{Context, State};
use futures::StreamExt;
use k8s_openapi::{
    api::core::v1::{Pod, PodTemplateSpec},
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
};
use kube::{
    api::{DeleteParams, ListParams, Patch, PatchParams, PostParams},
    core::ObjectMeta,
    error::ErrorResponse,
    runtime::{
        controller::Action,
        finalizer,
        wait::{await_condition, conditions},
        watcher, Controller,
    },
    Api, Client, CustomResource, CustomResourceExt, Error, Resource, ResourceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    process::{exit, Command, Stdio},
    sync::Arc,
    time::Duration,
};
use tracing::{error, info, warn};

/// Metadata that indicates what type of benchmark this is.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
pub enum BenchmarkType {
    #[default]
    System, // Testing system resources (e.g., CPU, RAM, disk, etc.).
    Network, // Testing network performance (e.g., load testing, latency, etc.).
}

/// Possible states that the benchmark can be in.
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, Default, PartialEq, Eq)]
pub enum BenchmarkState {
    #[default]
    Pending, // Benchmark CR created, but not running yet.
    Running,   // Benchmark pods started & the benchmark is running.
    Done, // Benchmark pods are done with benchmarking and transmitted the results to the controller.
    Completed, // Controller saved the results and marks this benchmark completed.
}

/// The spec for running a Helm chart.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct HelmSpec {
    pub repository_url: String,  // URL to the helm repository.
    pub chart_reference: String, // Reference to the helm chart in the repository.
}

/// A workload containing a Pod or a Helm chart that needs to be deployed.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkWorkload {
    pub pod_template: Option<PodTemplateSpec>,
    pub helm_chart: Option<HelmSpec>,
}

/// Kubernetes CR status object.
#[derive(Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkStatus {
    pub state: BenchmarkState,
    pub queue_position: u32,
}

/// Benchmark CRD spec.
#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "michiel.van.kenhove.ugent.be",
    version = "v1",
    kind = "Benchmark",
    status = "BenchmarkStatus",
    shortname = "bench",
    category = "all",
    printcolumn = r#"{"name": "State", "type": "string", "jsonPath": ".status.state"}"#,
    printcolumn = r#"{"name": "Queue Position", "type": "integer", "jsonPath": ".status.queuePosition"}"#,
    printcolumn = r#"{"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp"}"#,
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSpec {
    pub title: String,
    pub benchmark_type: BenchmarkType,
    pub workloads: Vec<BenchmarkWorkload>,
}

/// Custom implementation of the Benchmark CR auto-generated derived type for BenchmarkSpec.
impl Benchmark {
    pub async fn reconcile(
        &self,
        ctx: Arc<Context>,
        benchmark_api: &Api<Benchmark>,
        pods_api: &Api<Pod>,
    ) -> Result<Action, Error> {
        let namespace = self.namespace().unwrap_or("default".to_string());
        let name = self.name_any();

        if let Some(status_current) = &self.status {
            match status_current.state {
                BenchmarkState::Pending => {
                    // Only start new Benchmark if no other Benchmark is running.
                    if !ctx.state_data.read().await.is_benchmark_running {
                        // This Benchmark should only be started when it is first in queue.
                        if status_current.queue_position != 0 {
                            return Ok(Action::await_change());
                        }

                        // Set the state that a new Benchmark is about to start.
                        ctx.state_data
                            .write()
                            .await
                            .new_benchmark(name.clone(), namespace.clone());

                        // Start the workloads of this Benchmark.
                        for workload in self.spec.workloads.iter() {
                            if let Some(pod_template) = &workload.pod_template {
                                // Workload is a regular container image.
                                let pod = Pod {
                                    metadata: ObjectMeta {
                                        namespace: Some(namespace.clone()),
                                        generate_name: Some(format!("{}-", name.clone())),
                                        owner_references: Some(vec![self
                                            .controller_owner_ref(&())
                                            .unwrap()]),
                                        ..pod_template.metadata.clone().unwrap_or_default()
                                    },
                                    spec: pod_template.spec.clone(),
                                    ..Default::default()
                                };

                                pods_api.create(&PostParams::default(), &pod).await?;
                            } else if let Some(helm_spec) = &workload.helm_chart {
                                // Workload is a helm chart.
                                Command::new("helm")
                                    .args([
                                        "install",
                                        "-n",
                                        &namespace,
                                        "--repo",
                                        &helm_spec.repository_url,
                                        &helm_spec.chart_reference,
                                        "--generate-name",
                                    ])
                                    .spawn()
                                    .map_err(|_| {
                                        Error::Api(ErrorResponse {
                                            status: "Error".into(),
                                            message: "Helm install failed.".into(),
                                            reason: "Helm install command failed.".into(),
                                            code: 500,
                                        })
                                    })?;
                            }
                        }
                    }
                }
                BenchmarkState::Running => {
                    // Set the state to running.
                    ctx.set_is_benchmark_running(true).await;
                }
                BenchmarkState::Done => {
                    // Remove the workload Pods of the finished Benchmark (i.e., all the Pods in the namespace).
                    pods_api
                        .delete_collection(&DeleteParams::default(), &ListParams::default())
                        .await?;

                    // Remove Helm installs (i.e., all the Helm installs in the namespace).
                    Benchmark::uninstall_helm_charts(&namespace)?;

                    // Change the status of this CR to Completed.
                    Benchmark::set_benchmark_state(
                        ctx.k8s_client.clone(),
                        &name,
                        &namespace,
                        BenchmarkState::Completed,
                    )
                    .await?;

                    // Set the status of the Benchmark.
                    ctx.set_is_benchmark_running(false).await;
                    ctx.set_is_benchmark_done(true).await;

                    // Reorder Pending Benchmarks.
                    Benchmark::reorder_pending_benchmarks(
                        benchmark_api,
                        status_current.queue_position,
                    )
                    .await?;
                }
                BenchmarkState::Completed => {
                    info!("Benchmark {} completed.", name);
                }
            }
        } else {
            // Benchmark was just created and doesn't have a Status object yet.

            // Set pending status and queue position.
            let mut current_queue_pos: u32 = 0;

            // Loop over Pending and Running Benchmarks in current namespace to determine queue position.
            for bench in benchmark_api.list(&ListParams::default()).await?.iter() {
                if let Some(status) = &bench.status {
                    if (status.state == BenchmarkState::Pending
                        || status.state == BenchmarkState::Running)
                        && status.queue_position + 1 > current_queue_pos
                    {
                        current_queue_pos = status.queue_position + 1;
                    }
                }
            }

            // Set new status of this Benchmark.
            let status = json!({
                "status": BenchmarkStatus{queue_position: current_queue_pos, state: BenchmarkState::Pending}
            });
            benchmark_api
                .patch_status(&name, &PatchParams::default(), &Patch::Merge(&status))
                .await?;
        }

        Ok(Action::await_change())
    }

    /// Cleanup is called when a Benchmark CR get removed.
    /// Because we make use of the Owner principle, any Pods that are created by a Benchmark will be automatically removed by Kubernetes.
    /// See <https://kubernetes.io/docs/concepts/overview/working-with-objects/owners-dependents/> for more information.
    pub async fn cleanup(
        &self,
        ctx: Arc<Context>,
        benchmark_api: &Api<Benchmark>,
    ) -> Result<Action, Error> {
        let namespace = self.namespace().unwrap_or("default".to_string());

        // Remove Helm installs (i.e., all the Helm installs in the namespace).
        Benchmark::uninstall_helm_charts(&namespace)?;

        if let Some(status_current) = &self.status {
            // If a running Benchmark CR is being deleted, clear the shared state so a new Benchmark can start when this one is cleaned up.
            if status_current.state == BenchmarkState::Running {
                ctx.state_data.write().await.clear_state();
            }

            // Reorder the queue position of the pending Benchmarks.
            Benchmark::reorder_pending_benchmarks(benchmark_api, status_current.queue_position)
                .await?;
        }

        Ok(Action::await_change())
    }

    /// Set a new Benchmark state keeping other status properties intact.
    pub async fn set_benchmark_state(
        k8s_client: Client,
        benchmark_name: &str,
        namespace: &str,
        state: BenchmarkState,
    ) -> Result<(), Error> {
        let benchmark_api = Api::<Benchmark>::namespaced(k8s_client, namespace);

        let current_status = benchmark_api.get_status(benchmark_name).await?.status;

        if let Some(mut current_status) = current_status {
            current_status.state = state;

            let status = json!({ "status": current_status });

            benchmark_api
                .patch_status(
                    benchmark_name,
                    &PatchParams::default(),
                    &Patch::Merge(&status),
                )
                .await
                .map(|_| ())
        } else {
            Err(Error::Api(ErrorResponse {
                status: "Error".into(),
                message: "Could not set Benchmark state".into(),
                reason: "Current status is None".into(),
                code: 404,
            }))
        }
    }

    /// Decrement the queue position of all the pending benchmarks which have a queue position > `current_queue_pos`.
    pub async fn reorder_pending_benchmarks(
        benchmark_api: &Api<Benchmark>,
        current_queue_pos: u32,
    ) -> Result<(), Error> {
        let mut pending_benchmarks = Vec::<Benchmark>::new();

        // Store pending benchmarks if their queue position is larger than the benchmark that is being deleted or has state `done`.
        for bench_iter in benchmark_api.list(&ListParams::default()).await?.iter() {
            if let Some(status_iter) = &bench_iter.status {
                if status_iter.state == BenchmarkState::Pending
                    && status_iter.queue_position > current_queue_pos
                {
                    let mut pending_bench = bench_iter.clone();

                    // Decrement the queue position.
                    // Unwrap is safe, because the optional status of the benchmark was already checked.
                    pending_bench.status.as_mut().unwrap().queue_position -= 1;

                    // Store the pending benchmark.
                    pending_benchmarks.push(pending_bench);
                }
            }
        }

        // Update the benchmarks CRs with a new status.
        // rev() to make sure we update the largest queue position first and the lowest last,
        // to prevent that a benchmark gets started while the queue was not completely updated.
        for bench_iter in pending_benchmarks.iter().rev() {
            let new_status = json!({ "status": bench_iter.status.clone().unwrap() });

            benchmark_api
                .patch_status(
                    &bench_iter.name_any(),
                    &PatchParams::default(),
                    &Patch::Merge(&new_status),
                )
                .await?;
        }

        Ok(())
    }

    /// Uninstalls all the Helm charts from the provided namespace.
    pub fn uninstall_helm_charts(namespace: &str) -> Result<(), Error> {
        // helm ls -n benchmarking --all --short | xargs -r helm delete -n benchmarking
        let helm_list_stdout = Command::new("helm")
            .args(["ls", "-n", namespace, "--all", "--short"])
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| {
                Error::Api(ErrorResponse {
                    status: "Error".into(),
                    message: "Helm list failed.".into(),
                    reason: "Helm list command failed.".into(),
                    code: 500,
                })
            })?
            .stdout
            .ok_or(Error::Api(ErrorResponse {
                status: "Error".into(),
                message: "Failed to open Helm list stdout.".into(),
                reason: "Failed to open Helm list stdout.".into(),
                code: 500,
            }))?;

        Command::new("xargs")
            .args(["-r", "helm", "delete", "-n", namespace])
            .stdin(Stdio::from(helm_list_stdout))
            .spawn()
            .map_err(|_| {
                Error::Api(ErrorResponse {
                    status: "Error".into(),
                    message: "Helm delete failed.".into(),
                    reason: "Helm delete command failed.".into(),
                    code: 500,
                })
            })?;

        Ok(())
    }
}

/// Entry-point to start the controller.
pub async fn run(state: &State) {
    let k8s_client = Client::try_default()
        .await
        .expect("Failed to create kube client.");

    // Cluster level access to `CustomResourceDefinition` resources.
    let crd_api = Api::<CustomResourceDefinition>::all(k8s_client.clone());

    // Check if the CRD is installed.
    match crd_api.get_opt(Benchmark::crd_name()).await {
        Ok(None) => {
            // The Benchmark CRD does not exist.
            warn!("Benchmark CRD is not queryable. Is the CRD installed?");

            /*
            It is not advisable to install the CRD from inside the controller when developing an operator that runs on a production cluster.

            The intended use of this project is running benchmarks and tests in an isolated testing environment (isolated test cluster).
            Therefore, the decision is made to create the CRD from inside the controller, to improve the convenience usability of this tool.
            Also see: https://kube.rs/controllers/object/#installation
            */
            info!("Trying to install the CRD.");

            // Try to create the CRD, exit on fail.
            if let Err(err) = crd_api
                .create(&PostParams::default(), &Benchmark::crd())
                .await
            {
                error!("Error creating the CRD: {err:#?}");
                info!("Install the CRD manually: cargo run --bin generate_crd && kubectl apply -f crd.yaml");
                exit(1);
            }

            info!("CRD resource created, waiting until it is available...");

            // Wait until CRD is available.
            if let Err(err) = await_condition(
                crd_api,
                Benchmark::crd_name(),
                conditions::is_crd_established(),
            )
            .await
            {
                error!("Error waiting on CRD establishment: {err:#?}");
                info!("Manually check if the CRD was installed. If not, install the CRD manually: cargo run --bin generate_crd && kubectl apply -f crd.yaml");
                exit(1);
            }

            info!("CRD successfully installed.");
        }
        Ok(Some(_)) => info!("CRD is present."),
        Err(err) => {
            error!("Could not query CRD resources: {err:#?}.");
            exit(1);
        }
    }

    // Cluster level access to `Benchmark` resources.
    let benchmark_api = Api::<Benchmark>::all(k8s_client.clone());

    // Initializing and running the controller.
    Controller::new(benchmark_api, watcher::Config::default())
        .shutdown_on_signal()
        .run(reconcile, error_policy, state.create_context(k8s_client))
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("reconciled {o:?}"),
                Err(e) => error!("reconcile failed: {e:?}"),
            }
        })
        .await;
}

async fn reconcile(
    benchmark: Arc<Benchmark>,
    ctx: Arc<Context>,
) -> Result<Action, finalizer::Error<Error>> {
    let namespace = benchmark.namespace().unwrap_or("default".to_string());

    // Apis.
    let benchmark_api = Api::<Benchmark>::namespaced(ctx.k8s_client.clone(), &namespace);
    let pods_api = Api::<Pod>::namespaced(ctx.k8s_client.clone(), &namespace);

    finalizer(
        &benchmark_api,
        Benchmark::crd_name(),
        benchmark,
        |event| async {
            match event {
                finalizer::Event::Apply(benchmark) => {
                    benchmark.reconcile(ctx, &benchmark_api, &pods_api).await
                }
                finalizer::Event::Cleanup(benchmark) => {
                    benchmark.cleanup(ctx, &benchmark_api).await
                }
            }
        },
    )
    .await
}

fn error_policy(
    _benchmark: Arc<Benchmark>,
    _error: &finalizer::Error<Error>,
    _ctx: Arc<Context>,
) -> Action {
    Action::requeue(Duration::from_secs(60))
}
