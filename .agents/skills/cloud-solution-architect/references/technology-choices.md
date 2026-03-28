# Azure Technology Choice Decision Frameworks

Decision frameworks for selecting the right Azure service in each category. Use these tables to compare options based on scale, cost, complexity, and use case fit.

## Decision Approach

1. **Start with requirements** — workload type, scale needs, team expertise
2. **Use the comparison tables** — narrow to 2-3 candidates
3. **Follow the decision trees** — Azure Architecture Center provides flowcharts for compute, data store, load balancing, and messaging
4. **Validate with constraints** — budget, compliance, regional availability, existing infrastructure

---

## 1. Compute

Choose a compute service based on control needs, scaling model, and operational complexity.

| Service | Best For | Scale | Complexity | Cost Model |
|---|---|---|---|---|
| Azure VMs | Full control, lift-and-shift, custom OS | Manual/VMSS | High | Per-hour |
| App Service | Web apps, APIs, mobile backends | Built-in autoscale | Low | Per App Service plan |
| Azure Functions | Event-driven, short-lived processes | Consumption-based auto | Very Low | Per execution |
| AKS | Microservices, complex orchestration | Node/pod autoscaling | High | Per node VM |
| Container Apps | Serverless containers, microservices | KEDA-based autoscale | Medium | Per vCPU/memory/s |
| Container Instances | Simple containers, batch jobs | Per-instance | Very Low | Per second |

**Quick decision:**
- Need full OS control? → **VMs**
- Web app or API with minimal ops? → **App Service**
- Short-lived event-driven code? → **Functions**
- Complex microservices with K8s expertise? → **AKS**
- Microservices without K8s management? → **Container Apps**
- Run a container quickly, no orchestration? → **Container Instances**

---

## 2. Storage

Choose a storage service based on data structure, access patterns, and scale.

| Service | Best For | Access Pattern | Scale | Cost |
|---|---|---|---|---|
| Blob Storage | Unstructured data, media, backups | REST API, SDK | Massive | Per GB + operations |
| Azure Files | SMB/NFS file shares, lift-and-shift | File system mount | TB-scale | Per GB provisioned |
| Queue Storage | Simple message queuing | Pull-based | High throughput | Very low per message |
| Table Storage | NoSQL key-value data | REST API | TB-scale | Per GB + operations |
| Data Lake Storage | Big data analytics, hierarchical namespace | ABFS, REST | Massive | Per GB, tiered |

**Quick decision:**
- Blobs, images, videos, backups? → **Blob Storage**
- Need a mounted file share (SMB/NFS)? → **Azure Files**
- Simple async message queue? → **Queue Storage**
- Key-value NoSQL without Cosmos DB cost? → **Table Storage**
- Big data analytics with hierarchical namespace? → **Data Lake Storage**

---

## 3. Database

Choose a database based on data model, consistency needs, and scale requirements.

| Service | Best For | Consistency | Scale | Cost Model |
|---|---|---|---|---|
| Azure SQL | Relational, OLTP, enterprise apps | Strong (ACID) | Up to Hyperscale | DTU or vCore-based |
| Cosmos DB | Global distribution, multi-model, low latency | Tunable (5 levels) | Unlimited horizontal | RU/s + storage |
| Azure Database for PostgreSQL | Open-source relational, PostGIS, JSON | Strong (ACID) | Flexible Server auto | vCore-based |
| Azure Database for MySQL | Open-source relational, web apps | Strong (ACID) | Flexible Server auto | vCore-based |

**Quick decision:**
- Enterprise SQL Server workloads? → **Azure SQL**
- Global distribution or single-digit-ms latency? → **Cosmos DB**
- Open-source relational with spatial/JSON? → **PostgreSQL**
- Open-source relational for web apps? → **MySQL**

---

## 4. Messaging

Choose a messaging service based on delivery guarantees, throughput, and integration pattern.

| Service | Best For | Delivery | Throughput | Cost |
|---|---|---|---|---|
| Service Bus | Enterprise messaging, ordered delivery, transactions | At-least-once, at-most-once | Moderate-high | Per operation + unit |
| Event Hubs | Event streaming, telemetry, big data ingestion | At-least-once, partitioned | Very high (millions/s) | Per TU/PU + ingress |
| Event Grid | Event-driven reactive programming, webhooks | At-least-once | High | Per operation |
| Queue Storage | Simple async messaging, decoupling | At-least-once | Moderate | Very low per message |

**Quick decision:**
- Enterprise messaging with ordering/transactions? → **Service Bus**
- High-volume event streaming or telemetry? → **Event Hubs**
- Reactive event routing (resource events, webhooks)? → **Event Grid**
- Simple, cheap async decoupling? → **Queue Storage**

---

## 5. Networking

Choose a load balancing service based on traffic scope, protocol layer, and feature needs.

| Service | Best For | Scope | Layer | Features |
|---|---|---|---|---|
| Azure Front Door | Global HTTP(S) load balancing, CDN, WAF | Global | Layer 7 | CDN, WAF, SSL offload, caching |
| Application Gateway | Regional HTTP(S) load balancing, WAF | Regional | Layer 7 | WAF, URL routing, SSL termination |
| Azure Load Balancer | TCP/UDP traffic distribution | Regional | Layer 4 | High perf, zone redundant |
| Traffic Manager | DNS-based global traffic routing | Global | DNS | Failover, performance, geographic routing |

**Quick decision:**
- Global HTTP(S) with CDN and WAF? → **Front Door**
- Regional HTTP(S) with WAF? → **Application Gateway**
- Regional TCP/UDP load balancing? → **Load Balancer**
- DNS-based global failover? → **Traffic Manager**

---

## 6. AI Services

Choose an AI service based on customization needs and model type.

| Service | Best For | Complexity | Scale |
|---|---|---|---|
| Azure OpenAI | LLMs, GPT models, generative AI | Medium | API-based, token pricing |
| Azure AI Services | Pre-built AI (vision, speech, language) | Low | API-based, per transaction |
| Azure Machine Learning | Custom ML models, MLOps, training | High | Compute cluster-based |

**Quick decision:**
- Need GPT/LLM capabilities? → **Azure OpenAI**
- Pre-built vision, speech, or language? → **AI Services**
- Custom model training and MLOps? → **Azure Machine Learning**

---

## 7. Containers

Choose a container service based on orchestration needs and operational complexity.

| Service | Best For | Orchestration | Complexity | Cost |
|---|---|---|---|---|
| AKS | Full Kubernetes, complex workloads | Full K8s control plane | High | Per node VM |
| Container Apps | Serverless containers, microservices, event-driven | Managed (built on K8s) | Medium | Per vCPU/memory/s |
| Container Instances | Simple containers, sidecar groups, batch | None (per-instance) | Very Low | Per second |

**Quick decision:**
- Need full Kubernetes API and control? → **AKS**
- Serverless containers with event-driven scaling? → **Container Apps**
- Run a single container or batch job quickly? → **Container Instances**

---

## Related Decision Trees

The Azure Architecture Center provides detailed flowcharts for these decisions:

- [Choose a compute service](https://learn.microsoft.com/en-us/azure/architecture/guide/technology-choices/compute-decision-tree)
- [Compare Container Apps with other options](https://learn.microsoft.com/en-us/azure/architecture/guide/technology-choices/compute-decision-tree#compare-container-options)
- [Choose a data store](https://learn.microsoft.com/en-us/azure/architecture/guide/technology-choices/data-store-overview)
- [Load balancing decision tree](https://learn.microsoft.com/en-us/azure/architecture/guide/technology-choices/load-balancing-overview)
- [Compare messaging services](https://learn.microsoft.com/en-us/azure/architecture/guide/technology-choices/messaging)

> Source: [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
