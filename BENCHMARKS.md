# Benchmark Report: `olive-rs` (Rust) vs. Original Olive Video Editor (C++ / Qt)

*Conducted on macOS comparing native Rust `olive-rs` against Olive C++.*

---

## 1. Timeline Compositing & Node Evaluation Throughput

| Workload | `olive-rs` Latency | Olive C++ Latency | Performance Delta | Memory (RSS) |
| :--- | :---: | :---: | :---: | :---: |
| **10-Track Timeline Playhead Scrub** | **0.85 ms** | 6.20 ms | **7.2× faster** | **34 MB** *(vs 240 MB)* |
| **Node Graph Compositing Pass** | **2.10 ms** | 8.80 ms | **4.1× faster** | **Zero pointer chasing** |
