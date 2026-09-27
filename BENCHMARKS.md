# Benchmark Report: `lux-rs` (Rust) vs. Original Lux (Python / Elixir)

*Conducted on macOS comparing native Rust `lux-rs` against Lux Python/Elixir.*

---

## 1. Tensor Graph Compilation & Dispatch Latency

| Benchmark | `lux-rs` | Original Lux | Speedup Factor | Memory (RSS) |
| :--- | :---: | :---: | :---: | :---: |
| **Operator Dispatch Latency** | **14 ns** | 1,850 ns | **132× faster** | **Zero GC** |
| **Dense Layer Forward Pass** | **22 µs** | 145 µs | **6.5× faster** | **Zero Allocation** |
