# Cloud-Native-Bench

Cloud-Native-Bench is a highly extensible system, that enables users to run standardized and custom benchmarks that are fine-tuned to their specific business needs and that fully integrate with the system without the need to adapt the system in any way. An automated data gathering and analysis system greatly reduces the time spent to gather and parse different benchmark outputs, and enables the benchmarker to gain useful insights more swiftly.

The components of Cloud-Native-Bench are shown in the table below, with its corresponding technology or programming language it is developed in.

|                 | Technology     |
|-----------------|----------------|
| Operator        | Rust (kube-rs) |
| Result DB       | PostgreSQL     |
| Web server      | Rust (axum)    |
| Analysis runner | Python         |
