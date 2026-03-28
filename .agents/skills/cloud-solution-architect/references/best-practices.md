# Cloud Application Best Practices

Twelve best practices from the Azure Architecture Center for designing, building, and operating cloud applications.

---

## 1. API Design

Design RESTful web APIs that promote platform independence and loose coupling between clients and services.

### Key Recommendations

- Organize APIs around resources using nouns, not verbs, in URIs
- Use standard HTTP methods (GET, POST, PUT, PATCH, DELETE) with correct semantics
- Use plural nouns for collection endpoints (e.g., `/orders`, `/customers`)
- Support HATEOAS to enable client navigation of the API without prior knowledge
- Design coarse-grained operations to avoid chatty request patterns
- Do not expose internal database structure through the API surface
- Version APIs to manage breaking changes without disrupting existing clients
- Return appropriate HTTP status codes and consistent error response bodies

### WAF Pillar Alignment

Performance Efficiency · Operational Excellence

### Common Mistakes

- Using verbs in URIs (e.g., `/getOrders`) instead of resource-based paths
- Exposing database schema directly through API contracts
- Creating chatty APIs that require multiple round-trips for a single logical operation

---

## 2. API Implementation

Implement web APIs to be efficient, responsive, scalable, and available for consuming clients.

### Key Recommendations

- Make actions idempotent so retries are safe (especially PUT and DELETE)
- Support content negotiation via `Accept` and `Content-Type` headers
- Follow the HTTP specification for status codes, methods, and headers
- Handle exceptions gracefully and return meaningful error responses
- Support resource discovery through links and metadata
- Limit and paginate large result sets to minimize network traffic
- Handle large requests asynchronously using `202 Accepted` with status polling
- Compress responses where appropriate to reduce payload size

### WAF Pillar Alignment

Operational Excellence

### Common Mistakes

- Not handling large requests asynchronously, causing timeouts
- Not minimizing network traffic through pagination, filtering, or compression

---

## 3. Autoscaling

Dynamically allocate and deallocate resources to match performance requirements while optimizing cost.

### Key Recommendations

- Use Azure Monitor autoscale and built-in platform autoscaling features
- Scale based on metrics that directly correlate with load (CPU, queue length, request rate)
- Combine schedule-based and metric-based scaling for predictable traffic patterns
- Set appropriate minimum, maximum, and default instance counts
- Configure scale-in rules as carefully as scale-out rules
- Use cooldown periods to prevent oscillation (flapping)
- Plan for the delay between triggering a scale event and resources becoming available

### WAF Pillar Alignment

Performance Efficiency · Cost Optimization

### Common Mistakes

- Not setting appropriate minimum and maximum limits for scaling
- Not considering scale-in behavior, leading to premature resource removal
- Using metrics that do not accurately reflect application load

---

## 4. Background Jobs

Implement batch processing, long-running tasks, and workflows as background jobs decoupled from the user interface.

### Key Recommendations

- Use Azure platform services such as Functions, WebJobs, and Batch for hosting
- Trigger background jobs with events, schedules, or message queues
- Return results to calling tasks through queues, events, or shared storage
- Design jobs to be independently deployable, scalable, and versioned
- Handle partial failures and support safe restarts with checkpointing
- Monitor job health with logging, metrics, and alerting
- Implement graceful shutdown to allow in-progress work to complete

### WAF Pillar Alignment

Operational Excellence

### Common Mistakes

- Not handling failures or partial completion within long-running jobs
- Not monitoring background job health, missing silent failures

---

## 5. Caching

Copy frequently read, rarely modified data to fast storage close to the application to improve performance.

### Key Recommendations

- Cache data that is read often but changes infrequently
- Use Azure Cache for Redis for distributed, high-throughput caching
- Set appropriate expiration policies (TTL) to balance freshness and hit rates
- Handle cache misses gracefully with a cache-aside pattern
- Address concurrency issues when multiple processes update the same cached data
- Implement cache invalidation strategies aligned with data change patterns
- Pre-populate caches for known hot data during application startup

### WAF Pillar Alignment

Performance Efficiency

### Common Mistakes

- Caching highly volatile data that expires before it can be served
- Not handling cache invalidation, serving stale data to users
- Cache stampede — many concurrent requests regenerating the same expired entry

---

## 6. CDN (Content Delivery Network)

Use CDNs to deliver static and dynamic web content efficiently to users from edge locations worldwide.

### Key Recommendations

- Offload static assets (images, scripts, stylesheets) to the CDN to reduce origin load
- Configure appropriate cache-control headers for each content type
- Version static content via file names or query strings for reliable cache busting
- Use HTTPS and enforce TLS for secure delivery
- Plan for CDN fallback so the application degrades gracefully if the CDN is unavailable
- Handle deployment and versioning so users receive updated content promptly

### WAF Pillar Alignment

Performance Efficiency

### Common Mistakes

- Not versioning content, causing users to receive stale cached assets after deployments
- Setting improper cache headers, resulting in under-caching or over-caching

---

## 7. Data Partitioning

Divide data stores into partitions to improve scalability, availability, and performance while reducing contention and storage costs.

### Key Recommendations

- Choose partition keys that distribute data and load evenly across partitions
- Use horizontal (sharding), vertical, or functional partitioning based on access patterns
- Minimize cross-partition queries to avoid performance degradation
- Design partitions to match the most common query patterns
- Plan for rebalancing as data volume and access patterns evolve
- Consider partition limits and throughput caps of the target data store
- Reduce contention and storage costs by separating hot and cold data

### WAF Pillar Alignment

Performance Efficiency · Cost Optimization

### Common Mistakes

- Creating hotspots by selecting a partition key with skewed distribution
- Not considering the cost and latency of cross-partition queries

---

## 8. Data Partitioning Strategies (by Service)

Apply service-specific partitioning strategies across Azure SQL Database, Azure Table Storage, Azure Blob Storage, and other data services.

### Key Recommendations

- Shard Azure SQL Database to distribute data for horizontal scaling
- Design Azure Table Storage partition keys around query access patterns
- Organize Azure Blob Storage using virtual directories and naming conventions
- Align partition boundaries with the most frequent query predicates
- Reduce latency by co-locating related data within the same partition
- Monitor partition metrics and rebalance when skew is detected

### WAF Pillar Alignment

Performance Efficiency · Cost Optimization

### Common Mistakes

- Not aligning the partition strategy with actual query patterns, causing full scans
- Ignoring service-specific partition limits and throttling thresholds

---

## 9. Host Name Preservation

Preserve the original HTTP host name between reverse proxies and backend web applications to avoid issues with cookies, redirects, and CORS.

### Key Recommendations

- Forward the original `Host` header from the reverse proxy to the backend
- Configure Azure Front Door, Application Gateway, and API Management for host preservation
- Ensure cookies are set with the correct domain matching the original host name
- Verify redirect URLs reference the external host name, not the internal backend address
- Test CORS configurations end-to-end with the preserved host name
- Document host name flow across all network hops in the architecture

### WAF Pillar Alignment

Reliability

### Common Mistakes

- Not preserving host headers, causing redirect loops or incorrect absolute URLs
- Breaking session cookies because the cookie domain does not match the forwarded host

---

## 10. Message Encoding

Choose the right payload structure, encoding format, and serialization library for asynchronous messages exchanged between distributed components.

### Key Recommendations

- Evaluate JSON, Avro, Protobuf, and MessagePack based on performance and interoperability needs
- Use schema registries to enforce and version message contracts
- Validate incoming messages against their schemas before processing
- Prefer compact binary formats (Protobuf, Avro) for high-throughput, latency-sensitive paths
- Use JSON for human-readable messages and broad ecosystem compatibility
- Consider backward and forward compatibility when evolving message schemas

### WAF Pillar Alignment

Security

### Common Mistakes

- Using an inefficient encoding format for high-volume message streams
- Not validating message schemas, allowing malformed data into the processing pipeline

---

## 11. Monitoring and Diagnostics

Track system health, usage, and performance through a comprehensive monitoring pipeline that turns raw data into alerts, reports, and automated triggers.

### Key Recommendations

- Instrument applications with structured logging, metrics, and distributed tracing
- Use Azure Monitor, Application Insights, and Log Analytics as the monitoring backbone
- Define actionable alerts with clear thresholds, severity levels, and response procedures
- Detect and correct issues before they affect users by monitoring leading indicators
- Correlate telemetry across services using distributed trace context (e.g., correlation IDs)
- Establish performance baselines and track deviations over time
- Build dashboards for operational visibility across all tiers of the architecture
- Review and tune alert rules regularly to reduce noise

### WAF Pillar Alignment

Operational Excellence

### Common Mistakes

- Insufficient logging, making incident root-cause analysis slow or impossible
- Not using distributed tracing in microservice or multi-tier architectures
- Alert fatigue from poorly tuned thresholds that generate excessive false positives

---

## 12. Transient Fault Handling

Detect and handle transient faults caused by momentary loss of network connectivity, temporary service unavailability, or resource throttling.

### Key Recommendations

- Implement retry logic with exponential backoff and jitter for transient failures
- Use a circuit breaker pattern to stop retrying when failures are persistent
- Distinguish transient faults (e.g., HTTP 429, 503) from permanent errors (e.g., 400, 404)
- Leverage built-in retry policies in Azure SDKs before adding custom retry logic
- Avoid duplicating retry layers across middleware and application code
- Log every retry attempt for post-incident analysis
- Set a maximum retry count and total timeout to bound retry duration

### WAF Pillar Alignment

Reliability

### Common Mistakes

- Retrying non-transient faults (e.g., authentication failures, bad requests)
- Not using exponential backoff, overwhelming a recovering service with constant retries
- Retry storms caused by multiple layers retrying simultaneously without coordination

---

> Source: [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
