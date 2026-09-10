# Purpose

Mélodium is designed for programs that process continuous, unbounded data flows: event-driven servers, signal pipelines, distributed workers, and anything that must keep running even when parts of the system encounter problems.

Most languages treat data as discrete values moved through memory step by step, which works well for many tasks. The difficulty appears when data never stops arriving, does not fit in memory, or must be processed across many machines simultaneously. Managing threads, queues, backpressure, and error handling in those situations requires significant effort and leaves many failure modes up to the developer.

Mélodium takes a different approach. A program is described as a graph of treatments connected by data flows. Execution is driven by data availability, not instruction order. The graph is fully validated before any execution starts, catching type mismatches and structural errors at startup. Parallelism and scalability emerge naturally from the graph structure, without explicit threading code.