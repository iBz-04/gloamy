# Azure Design Principles

Ten principles for building reliable, scalable, and manageable applications on Azure.

| # | Principle | Focus |
|---|-----------|-------|
| 1 | [Design for self-healing](#1-design-for-self-healing) | Resilience & automatic recovery |
| 2 | [Make all things redundant](#2-make-all-things-redundant) | Eliminate single points of failure |
| 3 | [Minimize coordination](#3-minimize-coordination) | Scalability through decoupling |
| 4 | [Design to scale out](#4-design-to-scale-out) | Horizontal scaling |
| 5 | [Partition around limits](#5-partition-around-limits) | Overcome service boundaries |
| 6 | [Design for operations](#6-design-for-operations) | Observability & automation |
| 7 | [Use managed services](#7-use-managed-services) | Reduce operational burden |
| 8 | [Use an identity service](#8-use-an-identity-service) | Centralized identity & access |
| 9 | [Design for evolution](#9-design-for-evolution) | Change-friendly architecture |
| 10 | [Build for the needs of business](#10-build-for-the-needs-of-business) | Align tech to business goals |

---

## 1. Design for self-healing

Design the application to detect failures, respond gracefully, and recover automatically without manual intervention.

### Recommendations

- **Implement retry logic with backoff** for transient failures in network calls, database connections, and external service interactions.
- **Use health endpoint monitoring** to expose liveness and readiness probes so orchestrators and load balancers can route traffic away from unhealthy instances.
- **Apply circuit breaker patterns** to prevent cascading failures — stop calling a failing dependency and allow it time to recover.
- **Degrade gracefully** by serving reduced functionality (cached data, default responses) rather than failing entirely when a dependency is unavailable.
- **Adopt chaos engineering** with Azure Chaos Studio to proactively inject faults and validate recovery paths before real incidents occur.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Retry | Handle transient faults by transparently retrying failed operations |
| Circuit Breaker | Prevent repeated calls to a failing service |
| Bulkhead | Isolate failures so one component doesn't take down others |
| Health Endpoint Monitoring | Expose health checks for load balancers and orchestrators |
| Leader Election | Coordinate distributed instances by electing a leader |
| Throttling | Control resource consumption by limiting request rates |

### Azure services

- **Azure Chaos Studio** — fault injection and chaos experiments
- **Azure Monitor / Application Insights** — health monitoring, alerting, diagnostics
- **Azure Traffic Manager / Front Door** — DNS and global failover
- **Availability Zones** — zonal redundancy within a region

---

## 2. Make all things redundant

Build redundancy into the application at every layer to avoid single points of failure. Composite availability formula: `1 - (1 - A)^N` where A is the availability of a single instance and N is the number of instances.

### Recommendations

- **Place VMs behind a load balancer** and deploy multiple instances to ensure requests can be served even if one instance fails.
- **Replicate databases** using read replicas, active geo-replication, or multi-region write to protect data and maintain read performance during outages.
- **Use multi-zone and multi-region deployments** to survive datacenter and regional failures — define clear RTO (Recovery Time Objective) and RPO (Recovery Point Objective) targets.
- **Partition workloads for availability** so that a failure in one partition doesn't affect others.
- **Design for automatic failover** with health probes and traffic routing that redirects users without manual intervention.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Deployment Stamps | Deploy independent, identical copies of infrastructure |
| Geode | Deploy backend services across geographies |
| Health Endpoint Monitoring | Detect unhealthy instances for failover |
| Queue-Based Load Leveling | Buffer requests to smooth demand spikes |

### Azure services

- **Azure Load Balancer / Application Gateway** — distribute traffic across instances
- **Azure SQL geo-replication / Cosmos DB multi-region** — database redundancy
- **Availability Zones** — zonal redundancy within a region
- **Azure Site Recovery** — disaster recovery orchestration
- **Azure Front Door** — global load balancing with automatic failover

---

## 3. Minimize coordination

Minimize coordination between application services to achieve scalability. Tightly coupled services that require synchronous calls create bottlenecks and reduce availability.

### Recommendations

- **Embrace eventual consistency** instead of requiring strong consistency across services — accept that data may be temporarily out of sync.
- **Use domain events and asynchronous messaging** to decouple producers and consumers so they can operate independently.
- **Consider CQRS** (Command Query Responsibility Segregation) to separate read and write workloads with independently optimized stores.
- **Design idempotent operations** so messages can be safely retried or delivered more than once without unintended side effects.
- **Use optimistic concurrency** with version tokens or ETags instead of pessimistic locks that create coordination bottlenecks.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| CQRS | Separate reads from writes for independent scaling |
| Event Sourcing | Capture all changes as an immutable sequence of events |
| Saga | Manage distributed transactions without two-phase commit |
| Asynchronous Request-Reply | Decouple request and response across services |
| Competing Consumers | Process messages in parallel across multiple consumers |

### Azure services

- **Azure Service Bus** — reliable enterprise messaging with queues and topics
- **Azure Event Grid** — event-driven routing at scale
- **Azure Event Hubs** — high-throughput event streaming
- **Azure Cosmos DB** — tunable consistency levels (eventual to strong)

---

## 4. Design to scale out

Design the application so it can scale horizontally by adding or removing instances, rather than scaling up to larger hardware.

### Recommendations

- **Avoid instance stickiness and session affinity** — store session state externally (Redis, database) so any instance can handle any request.
- **Identify and resolve bottlenecks** that prevent horizontal scaling, such as shared databases, monolithic components, or stateful in-memory caches.
- **Decompose workloads** into discrete services that can be scaled independently based on their specific demand profiles.
- **Use autoscaling based on live metrics** (CPU, queue depth, request latency) rather than fixed schedules to match capacity to real demand.
- **Design for scale-in** — handle instance removal gracefully with connection draining and proper shutdown hooks.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Competing Consumers | Distribute work across multiple consumers |
| Sharding | Distribute data across partitions for parallel processing |
| Deployment Stamps | Scale by deploying additional independent stamps |
| Static Content Hosting | Offload static assets to reduce compute load |
| Throttling | Protect the system from overload during scale events |

### Azure services

- **Azure Virtual Machine Scale Sets** — autoscale VM pools
- **Azure App Service / Azure Functions** — built-in autoscale
- **Azure Kubernetes Service (AKS)** — horizontal pod autoscaler and cluster autoscaler
- **Azure Cache for Redis** — externalize session state
- **Azure CDN / Front Door** — offload static content delivery

---

## 5. Partition around limits

Use partitioning to work around database, network, and compute limits. Every Azure service has limits — partitioning allows you to scale beyond them.

### Recommendations

- **Partition databases** horizontally (sharding), vertically (splitting columns), or functionally (by bounded context) to distribute load and storage.
- **Design partition keys to avoid hotspots** — choose keys that distribute data and traffic evenly across partitions.
- **Partition at different levels** — database, queue, network, and compute — to address bottlenecks wherever they occur.
- **Understand service-specific limits** for throughput, connections, storage, and request rates and design partitioning strategies accordingly.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Sharding | Distribute data across multiple databases or partitions |
| Priority Queue | Process high-priority work before lower-priority work |
| Queue-Based Load Leveling | Buffer writes to smooth spikes |
| Valet Key | Grant limited direct access to resources |

### Azure services

- **Azure Cosmos DB** — automatic partitioning with configurable partition keys
- **Azure SQL Elastic Pools** — manage and scale multiple databases
- **Azure Storage** — table, blob, and queue partitioning
- **Azure Service Bus** — partitioned queues and topics

---

## 6. Design for operations

Design the application so that the operations team has the tools they need to monitor, diagnose, and manage it in production.

### Recommendations

- **Instrument everything** with structured logging, distributed tracing, and metrics to make the system observable from day one.
- **Use distributed tracing** with correlation IDs that flow across service boundaries to diagnose issues in microservices architectures.
- **Automate operational tasks** — deployments, scaling, failover, and routine maintenance should require no manual steps.
- **Treat configuration as code** — store all environment configuration in version control and deploy it through the same CI/CD pipelines as application code.
- **Implement dashboards and alerts** that surface actionable information, not just raw data, so operators can respond quickly.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Health Endpoint Monitoring | Expose operational health for monitoring tools |
| Ambassador | Offload cross-cutting concerns like logging and monitoring |
| Sidecar | Deploy monitoring agents alongside application containers |
| External Configuration Store | Centralize configuration management |

### Azure services

- **Azure Monitor** — metrics, logs, and alerts across all Azure resources
- **Application Insights** — application performance monitoring and distributed tracing
- **Azure Log Analytics** — centralized log querying with KQL
- **Azure Resource Manager (ARM) / Bicep** — infrastructure as code
- **Azure DevOps / GitHub Actions** — CI/CD pipelines

---

## 7. Use managed services

Prefer platform as a service (PaaS) over infrastructure as a service (IaaS) wherever possible to reduce operational overhead.

### Recommendations

- **Default to PaaS** for compute, databases, messaging, and storage — let Azure handle OS patching, scaling, and high availability.
- **Use IaaS only when you need fine-grained control** over the operating system, runtime, or network configuration that PaaS cannot provide.
- **Leverage built-in scaling and redundancy** features of managed services instead of building and maintaining them yourself.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Backends for Frontends | Use managed API gateways per client type |
| Gateway Aggregation | Aggregate calls through a managed gateway |
| Static Content Hosting | Use managed storage for static assets |

### Azure services

| IaaS | PaaS Alternative |
|------|-------------------|
| VMs with IIS/Nginx | Azure App Service |
| VMs with SQL Server | Azure SQL Database |
| VMs with RabbitMQ | Azure Service Bus |
| VMs with Kubernetes | Azure Kubernetes Service (AKS) |
| VMs with custom functions | Azure Functions |
| VMs with Redis | Azure Cache for Redis |
| VMs with Elasticsearch | Azure AI Search |

---

## 8. Use an identity service

Use a centralized identity platform instead of building or managing your own authentication and authorization system.

### Recommendations

- **Use Microsoft Entra ID** (formerly Azure AD) as the single identity provider for users, applications, and service-to-service authentication.
- **Never store credentials in application code or configuration** — use managed identities, certificate-based auth, or federated credentials.
- **Implement federation protocols** (SAML, OIDC, OAuth 2.0) to integrate with external identity providers and enable single sign-on.
- **Adopt modern security features** — passwordless authentication (FIDO2, Windows Hello), conditional access policies, multi-factor authentication (MFA), and single sign-on (SSO).
- **Use managed identities for Azure resources** to eliminate credential management for service-to-service communication entirely.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Federated Identity | Delegate authentication to an external identity provider |
| Gatekeeper | Protect backends by validating identity at the edge |
| Valet Key | Grant scoped, time-limited access to resources |

### Azure services

- **Microsoft Entra ID** — cloud identity and access management
- **Azure Managed Identities** — credential-free service-to-service auth
- **Azure Key Vault** — secrets, certificates, and key management
- **Microsoft Entra External ID** — customer and partner identity (B2C/B2B)

---

## 9. Design for evolution

Design the architecture so it can evolve over time as requirements, technologies, and team understanding change.

### Recommendations

- **Enforce loose coupling and high cohesion** — services should expose well-defined interfaces and encapsulate their internal implementation details.
- **Encapsulate domain knowledge** within service boundaries so changes to business logic don't ripple across the system.
- **Use asynchronous messaging** between services to reduce temporal coupling — services don't need to be available at the same time.
- **Version APIs** from day one so clients can migrate at their own pace and you can evolve without breaking existing consumers.
- **Deploy services independently** with their own release cadence — avoid coordinated "big bang" deployments.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Anti-Corruption Layer | Isolate new services from legacy systems |
| Strangler Fig | Incrementally migrate a monolith to microservices |
| Backends for Frontends | Evolve APIs independently per client type |
| Gateway Routing | Route requests to different service versions |

### Azure services

- **Azure API Management** — API versioning, routing, and lifecycle management
- **Azure Kubernetes Service (AKS)** — independent service deployments with rolling updates
- **Azure Service Bus** — asynchronous inter-service messaging
- **Azure Container Apps** — revision-based deployments with traffic splitting

---

## 10. Build for the needs of business

Every design decision must be justified by a business requirement. Align technical choices with business goals, constraints, and growth plans.

### Recommendations

- **Define RTO, RPO, and MTO** (Recovery Time Objective, Recovery Point Objective, Maximum Tolerable Outage) for each workload based on business impact analysis.
- **Document SLAs and SLOs** — understand the composite SLA of your architecture and set internal SLOs that provide an error budget for engineering work.
- **Model the system around the business domain** using domain-driven design to ensure the architecture reflects how the business operates.
- **Define functional and nonfunctional requirements explicitly** — capture performance targets, compliance needs, data residency constraints, and user experience expectations.
- **Plan for growth** — design capacity models that account for business projections, seasonal peaks, and market expansion.

### Related design patterns

| Pattern | Purpose |
|---------|---------|
| Priority Queue | Process business-critical work first |
| Throttling | Protect SLOs under heavy load |
| Deployment Stamps | Scale to new markets and regions |
| Bulkhead | Isolate critical workloads from non-critical ones |

### Azure services

- **Azure Advisor** — cost, performance, reliability, and security recommendations
- **Azure Cost Management** — budget tracking and cost optimization
- **Azure Service Health** — SLA tracking and incident awareness
- **Azure Well-Architected Framework Review** — assess architecture against best practices
- **Azure Monitor SLO/SLI dashboards** — measure and track service level objectives

---

> Source: [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
