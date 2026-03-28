# Azure Architecture Styles Reference

## Comparison Table

| Style | Dependency Management | Domain Type |
|---|---|---|
| N-tier | Horizontal tiers divided by subnet | Traditional business, low update frequency |
| Web-Queue-Worker | Front/back-end decoupled by async messaging | Simple domain, resource-intensive tasks |
| Microservices | Vertically decomposed services via APIs | Complex domain, frequent updates |
| Event-driven | Producer/consumer, independent views | IoT, real-time systems |
| Big data | Divide into small chunks, parallel processing | Batch/real-time data analysis, ML |
| Big compute | Data allocation to thousands of cores | Compute-intensive (simulation) |

---

## 1. N-tier

Traditional architecture that divides an application into logical layers and physical tiers. Each layer has a specific responsibility and communicates only with the layer directly below it.

### Logical Diagram

```
┌──────────────────────────────────┐
│        Presentation Tier         │  ← Web / UI
│          (Subnet A)              │
├──────────────────────────────────┤
│       Business Logic Tier        │  ← Rules / Workflows
│          (Subnet B)              │
├──────────────────────────────────┤
│        Data Access Tier          │  ← Database / Storage
│          (Subnet C)              │
└──────────────────────────────────┘
```

### Benefits

- Familiar pattern for most development teams
- Natural mapping for migrating existing layered applications to Azure
- Clear separation of concerns between tiers

### Challenges

- Horizontal layering makes cross-cutting changes difficult — a single feature may touch every tier
- Limits agility and release velocity as tiers are tightly coupled vertically

### Best Practices

- Use VNet subnets to isolate tiers and control traffic flow with NSGs
- Keep each tier stateless where possible to enable horizontal scaling
- Use managed services (App Service, Azure SQL) to reduce operational overhead

### Dependency Management

Horizontal tiers divided by subnet. Each tier depends only on the tier directly below it, enforced through network segmentation.

### Recommended Azure Services

- Azure App Service
- Azure SQL Database
- Azure Virtual Machines
- Azure Virtual Network (subnets)

---

## 2. Web-Queue-Worker

A web front end handles HTTP requests while a worker process performs resource-intensive or long-running tasks. The two components communicate through an asynchronous message queue.

### Logical Diagram

```
                ┌───────────┐
 HTTP ─────────►│    Web    │
 Requests       │ Front End │
                └─────┬─────┘
                      │
                      ▼
               ┌──────────────┐
               │  Message     │
               │  Queue       │
               └──────┬───────┘
                      │
                      ▼
                ┌───────────┐
                │  Worker   │
                │  Process  │
                └─────┬─────┘
                      │
                      ▼
                ┌───────────┐
                │  Database │
                └───────────┘
```

### Benefits

- Easy to understand and deploy, especially with managed compute services
- Clean separation between interactive and background workloads
- Each component can scale independently

### Challenges

- Without careful design, the front end and worker can become monolithic components that are hard to maintain and update
- Hidden dependencies may emerge if front end and worker share data schemas or storage

### Best Practices

- Keep the web front end thin — delegate heavy processing to the worker
- Use durable message queues to ensure work is not lost on failure
- Design idempotent worker operations to handle message retries safely

### Dependency Management

Front-end and back-end jobs are decoupled by asynchronous messaging. The web tier never calls the worker directly; all communication flows through the queue.

### Recommended Azure Services

- Azure App Service
- Azure Functions
- Azure Queue Storage
- Azure Service Bus

---

## 3. Microservices

A collection of small, autonomous services where each service implements a single business capability. Each service owns its bounded context and data, and communicates with other services via well-defined APIs.

### Logical Diagram

```
┌──────────┐   ┌──────────┐   ┌──────────┐
│ Service  │   │ Service  │   │ Service  │
│    A     │   │    B     │   │    C     │
│ ┌──────┐ │   │ ┌──────┐ │   │ ┌──────┐ │
│ │ Data │ │   │ │ Data │ │   │ │ Data │ │
│ └──────┘ │   │ └──────┘ │   │ └──────┘ │
└────┬─────┘   └────┬─────┘   └────┬─────┘
     │              │              │
     └──────┬───────┘──────────────┘
            ▼
     ┌──────────────┐
     │ API Gateway  │
     └──────┬───────┘
            │
         Clients
```

### Benefits

- Autonomous teams can develop, deploy, and scale services independently
- Enables frequent updates and higher release velocity
- Technology diversity — each service can use the stack best suited to its task

### Challenges

- Service discovery and inter-service communication add complexity
- Data consistency across services requires patterns like Saga or eventual consistency
- Distributed system management (monitoring, debugging, tracing) is inherently harder

### Best Practices

- Define clear bounded contexts — avoid sharing databases between services
- Use an API gateway for cross-cutting concerns (auth, rate limiting, routing)
- Implement health checks, circuit breakers, and distributed tracing from day one

### Dependency Management

Vertically decomposed services calling each other via APIs. Each service is independently deployable with its own data store, minimizing coupling.

### Recommended Azure Services

- Azure Kubernetes Service (AKS)
- Azure Container Apps
- Azure API Management
- Azure Service Bus
- Azure Cosmos DB

---

## 4. Event-driven

A publish-subscribe architecture where event producers emit events and event consumers react to them. Producers and consumers are fully decoupled, communicating only through event channels or brokers.

### Logical Diagram

```
┌──────────┐     ┌──────────────────┐     ┌──────────┐
│ Producer │────►│                  │────►│ Consumer │
│    A     │     │   Event Broker   │     │    A     │
└──────────┘     │   / Channel     │     └──────────┘
                 │                  │
┌──────────┐     │  ┌────────────┐ │     ┌──────────┐
│ Producer │────►│  │  Pub/Sub   │ │────►│ Consumer │
│    B     │     │  │  or Stream │ │     │    B     │
└──────────┘     │  └────────────┘ │     └──────────┘
                 └──────────────────┘
```

**Two models:** Pub/Sub (events delivered to subscribers) and Event Streaming (events written to an ordered log for consumers to read).

**Consumer variations:** Simple event processing, basic correlation, complex event processing, event stream processing.

### Benefits

- Producers and consumers are fully decoupled — they can evolve independently
- Highly scalable — add consumers without affecting producers
- Responsive and well-suited to real-time processing pipelines

### Challenges

- Guaranteed delivery requires careful broker configuration and dead-letter handling
- Event ordering can be difficult to maintain across partitions
- Eventual consistency — consumers may see stale data temporarily
- Error handling and poison message management add operational complexity

### Best Practices

- Design events as immutable facts with clear schemas
- Use dead-letter queues for events that fail processing
- Implement idempotent consumers to handle duplicate delivery safely

### Dependency Management

Producer/consumer model with independent views per subsystem. Producers have no knowledge of consumers; each subsystem maintains its own projection of the event stream.

### Recommended Azure Services

- Azure Event Grid
- Azure Event Hubs
- Azure Functions
- Azure Service Bus
- Azure Stream Analytics

---

## 5. Big Data

Architecture designed to handle ingestion, processing, and analysis of data that is too large or complex for traditional database systems.

### Logical Diagram

```
┌─────────────┐    ┌──────────────────────────────────┐
│ Data Sources│───►│          Data Storage             │
│ (logs, IoT, │    │         (Data Lake)               │
│  files)     │    └──┬──────────────┬─────────────────┘
└─────────────┘       │              │
                      ▼              ▼
              ┌──────────────┐ ┌──────────────┐
              │    Batch     │ │  Real-time   │
              │  Processing  │ │  Processing  │
              └──────┬───────┘ └──────┬───────┘
                     │                │
                     ▼                ▼
              ┌───────────────────────────────┐
              │   Analytical Data Store       │
              └──────────────┬────────────────┘
                             │
              ┌──────────────▼────────────────┐
              │   Analysis & Reporting        │
              │   (Dashboards, ML Models)     │
              └───────────────────────────────┘

         Orchestration manages the full pipeline
```

**Components:** Data sources → Data storage (data lake) → Batch processing → Real-time processing → Analytical data store → Analysis and reporting → Orchestration.

### Benefits

- Process massive datasets that exceed traditional database capacity
- Support both batch and real-time analytics in a single architecture
- Enable predictive analytics and machine learning at scale

### Challenges

- Complexity of coordinating batch and real-time processing paths
- Data quality and governance across a data lake require disciplined schema management
- Cost management — large-scale storage and compute can grow unpredictably

### Best Practices

- Use parallelism for both batch and real-time processing
- Partition data to enable parallel reads and writes
- Apply schema-on-read semantics to keep ingestion flexible
- Process data in batches on arrival rather than waiting for scheduled windows
- Balance usage costs against time-to-insight requirements

### Dependency Management

Divide huge datasets into small chunks for parallel processing. Each chunk can be processed independently, with an orchestration layer coordinating the overall pipeline.

### Recommended Azure Services

- Microsoft Fabric
- Azure Data Lake Storage
- Azure Event Hubs
- Azure SQL Database
- Azure Cosmos DB
- Power BI

---

## 6. Big Compute

Architecture for large-scale workloads that require hundreds or thousands of cores running in parallel. Tasks can be independent (embarrassingly parallel) or tightly coupled requiring inter-node communication.

### Logical Diagram

```
┌─────────────────────────────────────────────┐
│              Job Scheduler                  │
│         (submit, monitor, manage)           │
└─────────────────┬───────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌────────┐  ┌────────┐  ┌────────────────┐
│ Core   │  │ Core   │  │ Core           │
│ Pool 1 │  │ Pool 2 │  │ Pool N         │
│(100s)  │  │(100s)  │  │(1000s of cores)│
└───┬────┘  └───┬────┘  └───┬────────────┘
    │           │            │
    └─────────┬─┘────────────┘
              ▼
     ┌──────────────┐
     │   Results    │
     │   Storage    │
     └──────────────┘
```

**Use cases:** Simulations, financial risk modeling, oil exploration, drug design, image rendering.

### Benefits

- High performance through massive parallel processing
- Access to specialized hardware (GPU, FPGA, InfiniBand) for compute-intensive workloads
- Scales to thousands of cores for embarrassingly parallel problems

### Challenges

- Managing VM infrastructure at scale (provisioning, patching, decommissioning)
- Provisioning thousands of cores in a timely manner to meet job deadlines
- Cost control — idle compute resources are expensive

### Best Practices

- Use low-priority or spot VMs to reduce cost for fault-tolerant workloads
- Auto-scale compute pools based on job queue depth
- Partition work into independent tasks when possible to maximize parallelism

### Dependency Management

Data allocation to thousands of cores. The job scheduler distributes work units across the compute pool, with each core processing its assigned data partition independently.

### Recommended Azure Services

- Azure Batch
- Microsoft HPC Pack
- H-series Virtual Machines (HPC-optimized)

---

> Source: [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
