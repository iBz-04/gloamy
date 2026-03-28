# Mission-Critical Architecture on Azure

Guidance for designing mission-critical workloads on Azure that prioritize cloud-native capabilities to maximize reliability and operational effectiveness.

**Target SLO:** **99.99%** or higher — permitted annual downtime: **52 minutes 35 seconds**.

All encompassed design decisions are intended to accomplish this target SLO.

| SLO Target | Permitted Annual Downtime | Typical Use Case |
|---|---|---|
| 99.9% | 8 hours 45 minutes | Standard business apps |
| 99.95% | 4 hours 22 minutes | Important business apps |
| 99.99% | 52 minutes 35 seconds | Mission-critical workloads |
| 99.999% | 5 minutes 15 seconds | Safety-critical systems |

---

## Key Design Strategies

### 1. Redundancy in Layers

Deploy redundancy at every layer of the architecture to eliminate single points of failure.

- Deploy to multiple regions in an **active-active** model — application distributed across 2+ Azure regions handling active user traffic simultaneously
- Utilize **availability zones** for all considered services — distributing components across physically separate datacenters inside a region
- Choose resources that support **global distribution** natively
- Apply zone-redundant configurations for all stateful services
- Ensure data replication meets RPO requirements across regions

**Azure services:** Azure Front Door (global routing), Azure Traffic Manager (DNS failover), Azure Cosmos DB (multi-region writes), Azure SQL (geo-replication)

### 2. Deployment Stamps

Deploy regional stamps as scale units — a logical set of resources that can be independently provisioned to keep up with demand changes.

- Each stamp is a **self-contained scale unit** with its own compute, caching, and local state
- Multiple nested scale units within a stamp (e.g., Frontend APIs and Background processors scale independently)
- **No dependencies between scale units** — they only communicate with shared services outside the stamp
- Scale units are **temporary/ephemeral** — store persistent system-of-record data only in the replicated database
- Use stamps for blue-green deployments by rolling out new units, validating, and gradually shifting traffic

**Key benefit:** Compartmentalization enables independent scaling and fault isolation per region.

### 3. Reliable and Repeatable Deployments

Apply the principle of Infrastructure as Code (IaC) for version control and standardized operations.

- Use **Terraform** or **Bicep** for infrastructure definition with version control
- Implement **zero-downtime blue/green deployment** pipelines — build and release pipelines fully automated
- Apply **environment consistency** — use the same deployment pipeline code across production and pre-production environments
- Integrate **continuous validation** — automated testing as part of DevOps processes
- Include synchronized **load and chaos testing** to validate both application code and underlying infrastructure
- Deploy stamps as a **single operational unit** — never partially deploy a stamp

### 4. Operational Insights

Build comprehensive observability without introducing single points of failure.

- Use **federated workspaces** for observability data — monitoring data for global and regional resources stored independently
- A centralized observability store is **NOT recommended** (it becomes a single point of failure)
- Use **cross-workspace querying** to achieve a unified data sink and single pane of glass for operations
- Construct a **layered health model** mapping application health to a traffic light model for contextualizing
- Health scores calculated for each **individual component**, then **aggregated at user flow level**
- Combine with key non-functional requirements (performance) as coefficients to quantify application health

---

## Design Areas

Each design area must be addressed for a mission-critical architecture.

| Design Area | Description | Key Concerns |
|---|---|---|
| **Application platform** | Infrastructure choices and mitigations for potential failure cases | AKS vs App Service, availability zones, containerization |
| **Application design** | Design patterns that allow for scaling and error handling | Stateless services, async messaging, queue-based decoupling |
| **Networking and connectivity** | Network considerations for routing incoming traffic to stamps | Global load balancing, WAF, DDoS protection, private endpoints |
| **Data platform** | Choices in data store technologies | Volume, velocity, variety, veracity; active-active vs active-passive |
| **Deployment and testing** | Strategies for CI/CD pipelines and automation | Blue/green deployments, load testing, chaos testing |
| **Health modeling** | Observability through customer impact analysis | Correlated monitoring, traffic light model, health scores |
| **Security** | Mitigation of attack vectors | Microsoft Zero Trust model, identity-based access, encryption |
| **Operational procedures** | Processes related to runtime operations | Deployment SOPs, key management, patching, incident response |

---

## Active-Active Multi-Region Architecture

The core topology for mission-critical workloads distributes the application across multiple Azure regions.

### Architecture Characteristics

- Application distributed across **2+ Azure regions** handling active user traffic simultaneously
- Each region contains independent **deployment stamps** (scale units)
- **Azure Front Door** provides global routing, SSL termination, and WAF at the edge
- Scale units have **no cross-dependencies** — they communicate only with shared services (e.g., global database, DNS)
- Persistent data resides only in the **replicated database** — stamps store no durable local state
- When scale units are replaced or retired, applications reconnect transparently

### Data Replication Strategies

| Strategy | Writes | Reads | Consistency | Best For |
|---|---|---|---|---|
| Active-passive (Azure SQL) | Single primary region | All regions via read replicas | Strong | Relational data, ACID transactions |
| Active-active (Cosmos DB) | All regions | All regions | Tunable (5 levels) | Document/key-value data, global apps |
| Write-behind (Redis → SQL) | Redis first, async to SQL | Redis or SQL | Eventual | High-throughput writes, rate limiting |

### Regional Stamp Composition

Each stamp typically includes:

- **Compute tier** — App Service or AKS with multiple instances across availability zones
- **Caching tier** — Azure Managed Redis for session state, rate limiting, feature flags
- **Configuration** — Azure App Configuration for settings (capacity correlates with requests/second)
- **Secrets** — Azure Key Vault for certificates and secrets
- **Networking** — Virtual network with private endpoints, NSGs, and service endpoints

---

## Health Modeling and Traffic Light Approach

Health modeling provides the foundation for automated operational decisions.

### Building the Health Model

1. **Identify user flows** — map critical paths through the application (e.g., "user login", "checkout", "search")
2. **Decompose into components** — each flow depends on specific compute, data, and network components
3. **Assign health scores** — each component reports a health score based on metrics (latency, error rate, saturation)
4. **Aggregate per flow** — combine component scores weighted by criticality to produce a flow-level health score
5. **Apply traffic light** — map aggregate scores to **Green** (healthy), **Yellow** (degraded), **Red** (unhealthy)

### Health Score Coefficients

| Factor | Metric Examples | Weight Guidance |
|---|---|---|
| Availability | Error rate, HTTP 5xx ratio | High — directly impacts users |
| Performance | P95 latency, request duration | Medium — affects user experience |
| Saturation | CPU %, memory %, queue depth | Medium — indicates future problems |
| Freshness | Data replication lag, cache age | Lower — depends on consistency needs |

### Operational Actions by Health State

| State | Meaning | Automated Action |
|---|---|---|
| 🟢 Green | All components healthy | Normal operations |
| 🟡 Yellow | Degraded but functional | Alert on-call, increase monitoring frequency |
| 🔴 Red | Critical failure detected | Trigger failover, page on-call, block deployments |

---

## Zero-Downtime Deployment (Blue/Green)

Deployment must never cause downtime in a mission-critical system.

### Blue/Green Process

1. **Provision new stamp** — deploy a complete new scale unit ("green") alongside the existing one ("blue")
2. **Run validation** — execute automated smoke tests, integration tests, and synthetic transactions against the green stamp
3. **Canary traffic** — route a small percentage of production traffic (e.g., 5%) to the green stamp
4. **Monitor health** — compare health scores between blue and green stamps over a defined observation period
5. **Gradual shift** — increase traffic to green stamp in increments (5% → 25% → 50% → 100%)
6. **Decommission blue** — once green is fully validated, tear down the blue stamp

### Key Requirements

- Build and release pipelines must be **fully automated** — no manual deployment steps
- Use the **same pipeline code** for all environments (dev, staging, production)
- Each stamp deployed as a **single operational unit** — never partial
- Rollback is achieved by **shifting traffic back** to the previous stamp (still running during validation)
- **Continuous validation** runs throughout the deployment, not just at the end

---

## Chaos Engineering and Continuous Validation

Proactive failure testing ensures recovery mechanisms work before real incidents occur.

### Chaos Engineering Practices

- Use **Azure Chaos Studio** to run controlled experiments against production or pre-production environments
- Test failure modes: availability zone outage, network partition, dependency failure, CPU/memory pressure
- Run chaos experiments as part of the **CI/CD pipeline** — every deployment is validated under fault conditions
- **Synchronized load and chaos testing** — inject faults while the system is under realistic load

### Validation Checklist

- [ ] Health model detects injected faults within SLO-defined time windows
- [ ] Automated failover completes within target RTO
- [ ] No data loss exceeding target RPO during regional failover
- [ ] Application degrades gracefully (reduced functionality, not total failure)
- [ ] Alerts fire correctly and reach the on-call team
- [ ] Runbooks and automated remediation execute successfully

---

## Application Platform Considerations

### Platform Options

| Platform | Best For | Availability Zone Support | Complexity |
|---|---|---|---|
| **Azure App Service** | Web apps, APIs, PaaS-first approach | Yes (zone-redundant) | Low-Medium |
| **AKS** | Complex microservices, full K8s control | Yes (zone-redundant node pools) | High |
| **Container Apps** | Serverless containers, event-driven | Yes | Medium |

### Recommendations

- **Prioritize availability zones** for all production workloads — spread across physically separate datacenters
- **Containerize workloads** for reliability and portability between platforms
- Ensure all services in a scale unit support availability zones — don't mix zonal and non-zonal services
- For latency-sensitive or chatty workloads, consider tradeoffs of cross-zone traffic cost and latency

---

## Data Platform Considerations

### Choosing a Primary Database

| Scenario | Recommended Service | Deployment Model |
|---|---|---|
| Relational data, ACID transactions | **Azure SQL** | Active-passive with geo-replication |
| Global distribution, multi-model | **Azure Cosmos DB** | Active-active with multi-region writes |
| Multiple microservice databases | **Mixed (polyglot)** | Per-service database with appropriate model |

### Azure SQL in Mission-Critical

- Azure SQL does **not** natively support active-active concurrent writes in multiple regions
- Use **active-passive** strategy: single primary region for writes, read replicas in secondary regions
- **Partial active-active** possible at the application tier — route reads to local replicas, writes to primary
- Configure **auto-failover groups** for automated regional failover

### Azure Managed Redis in Mission-Critical

- Use within or alongside each scale unit for:
  - **Cache data** — rebuildable, repopulated on demand
  - **Session state** — user sessions during scale unit lifetime
  - **Rate limit counters** — per-user and per-tenant throttling
  - **Feature flags** — dynamic configuration without redeployment
  - **Coordination metadata** — distributed locks, leader election
- **Active geo-replication** enables Redis data to replicate asynchronously across regions
- Design cached data as either **rebuildable** (repopulate without availability impact) or **durable auxiliary state** (protected by persistence and geo-replication)

---

## Security in Mission-Critical

### Zero Trust Principles

- **Verify explicitly** — authenticate and authorize based on all available data points (identity, location, device, service)
- **Use least privilege access** — limit user access with Just-In-Time and Just-Enough-Access (JIT/JEA)
- **Assume breach** — minimize blast radius and segment access, verify end-to-end encryption, use analytics for threat detection

### Security Controls

| Layer | Control | Azure Service |
|---|---|---|
| Edge | DDoS protection, WAF | Azure Front Door, Azure DDoS Protection |
| Identity | Managed identities, RBAC | Microsoft Entra ID, Azure RBAC |
| Network | Private endpoints, NSGs | Azure Private Link, Virtual Network |
| Data | Encryption at rest and in transit | Azure Key Vault, TDE, TLS 1.2+ |
| Operations | Privileged access management | Microsoft Entra PIM, Azure Bastion |

---

## Operational Procedures

### Key Operational Processes

| Process | Description | Automation Level |
|---|---|---|
| **Deployment** | Blue/green with automated validation | Fully automated |
| **Scaling** | Stamp provisioning and decommissioning | Automated with manual approval gates |
| **Key rotation** | Certificate and secret rotation | Automated via Key Vault policies |
| **Patching** | OS and runtime updates | Automated via platform (PaaS) or pipeline (IaaS) |
| **Incident response** | Detection, triage, mitigation, resolution | Semi-automated (alert → runbook → human) |
| **Capacity planning** | Forecast demand, pre-provision stamps | Manual with data-driven analysis |

### Runbook Requirements

- All operational runbooks must be **tested in pre-production** with the same chaos/load scenarios as production
- **Automated remediation** preferred over manual intervention for known failure modes
- Runbooks must include **rollback procedures** for every change type
- **Post-incident reviews** (blameless) must feed back into health model and chaos experiment improvements

---

> Source: [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
