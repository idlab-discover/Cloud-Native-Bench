use benchmark_operator::benchmark_controller::Benchmark;
use kube::CustomResourceExt;

/// Generates the CustomResourceDefinition and writes to `crd.yaml`
fn main() -> std::io::Result<()> {
    std::fs::write(
        "crd.yaml",
        serde_yaml::to_string(&Benchmark::crd()).unwrap(),
    )?;

    Ok(())
}
