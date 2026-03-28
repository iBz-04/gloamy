# Performance Antipatterns

Ten common performance antipatterns in cloud applications and how to resolve them.

---

## 1. Busy Database

**Description:** Offloading too much processing to a data store. Using stored procedures for formatting, string manipulation, or complex calculations that belong in the application tier.

**Why it happens:**
- Database viewed as a service rather than a repository
- Developers write queries that format data for direct display
- Attempts to correct Extraneous Fetching by pushing compute to the database

**Symptoms:**
- Disproportionate decline in throughput and response times for database operations

**How to detect:**
- Performance monitoring of database activity
- Examine work performed during slow periods

**How to fix:**
- Move processing (formatting, string manipulation, calculations) to application tiers
- Limit the database to data access operations (aggregation is acceptable)
- Don't move processing out if it causes the database to transfer far more data (Extraneous Fetching)

**Example scenario:** A SQL query performs XML formatting, string concatenation, and locale-specific formatting in T-SQL instead of returning raw data and letting application code handle presentation.

---

## 2. Busy Front End

**Description:** Moving resource-intensive tasks onto foreground/UI threads instead of background threads.

**Why it happens:**
- Processing done synchronously in request handlers

**Symptoms:**
- High latency on requests
- Poor user responsiveness

**How to detect:**
- Monitor thread utilization and CPU usage on the front-end tier

**How to fix:**
- Move resource-intensive tasks to background threads or services (Azure Functions, WebJobs)

**Example scenario:** A web API controller performs image resizing synchronously within the HTTP request pipeline, blocking the thread and causing timeouts for other users during peak load.

---

## 3. Chatty I/O

**Description:** Continually sending many small network requests instead of fewer larger ones.

**Why it happens:**
- Following object-oriented patterns that make many small calls
- Individual property gets instead of batch reads

**Symptoms:**
- High number of I/O operations
- High latency due to network round trips

**How to detect:**
- Monitor the number of I/O requests and their sizes

**How to fix:**
- Bundle multiple smaller requests into fewer larger ones
- Use caching to avoid repeated calls
- Read data in bulk

**Example scenario:** An application retrieves a user profile by making separate API calls for name, email, address, and preferences instead of a single call that returns the complete profile object.

---

## 4. Extraneous Fetching

**Description:** Retrieving more data than needed, resulting in unnecessary I/O and memory consumption.

**Why it happens:**
- `SELECT *` queries
- Fetching all columns or rows when only a subset is needed

**Symptoms:**
- High memory usage
- Excessive bandwidth consumption

**How to detect:**
- Profile queries and check data transfer sizes

**How to fix:**
- Request only needed data columns and rows
- Use pagination and projections

**Example scenario:** An order history page runs `SELECT * FROM Orders` and filters in application code, transferring millions of rows when the user only sees the 20 most recent orders.

---

## 5. Improper Instantiation

**Description:** Repeatedly creating and destroying objects designed to be shared and reused, such as `HttpClient`, database connections, or service clients.

**Why it happens:**
- Not understanding object lifecycle
- Creating new instances per request

**Symptoms:**
- Port exhaustion
- Socket exhaustion
- Connection pool depletion

**How to detect:**
- Monitor connection counts, socket usage, and object creation rates

**How to fix:**
- Use singleton or static instances for shared clients
- Use connection pooling
- Use `IHttpClientFactory` for HTTP clients

**Example scenario:** A web application creates a new `HttpClient` instance for every incoming request. Under load, available sockets are exhausted and requests fail with `SocketException` errors.

---

## 6. Monolithic Persistence

**Description:** Using one data store for data with very different usage patterns.

**Why it happens:**
- Simplicity of a single database
- Legacy design decisions

**Symptoms:**
- Performance degradation as different workloads compete for the same resources

**How to detect:**
- Monitor data store metrics and identify competing access patterns

**How to fix:**
- Separate data by usage pattern into appropriate stores (hot/warm/cold)
- Apply polyglot persistence — use the right store for each data type

**Example scenario:** A single SQL database handles both high-throughput transactional writes and complex analytical reporting queries, causing lock contention that degrades both workloads.

---

## 7. No Caching

**Description:** Failing to cache frequently accessed data that changes infrequently.

**Why it happens:**
- Not considering a caching strategy
- Assuming the database can handle all read traffic

**Symptoms:**
- Repeated identical queries
- High database load
- Slow response times

**How to detect:**
- Monitor cache hit ratios and look for repeated identical queries

**How to fix:**
- Implement the Cache-Aside pattern
- Use Azure Cache for Redis
- Set appropriate TTLs based on data volatility

**Example scenario:** A product catalog page queries the database on every page load for data that only changes once a day, generating thousands of identical queries per hour.

---

## 8. Noisy Neighbor

**Description:** A single tenant consumes a disproportionate amount of shared resources, starving other tenants.

**Why it happens:**
- Multi-tenant systems without resource isolation or throttling

**Symptoms:**
- Other tenants experience degraded performance
- Resource starvation for well-behaved tenants

**How to detect:**
- Monitor per-tenant resource usage and identify outliers

**How to fix:**
- Implement tenant isolation, throttling, and quotas
- Use the Bulkhead pattern to partition resources
- Separate compute per tenant if needed

**Example scenario:** In a shared SaaS platform, one tenant runs a bulk data import that saturates the database connection pool, causing timeout errors for all other tenants.

---

## 9. Retry Storm

**Description:** Retrying failed requests too aggressively, amplifying failures during outages.

**Why it happens:**
- Aggressive retry policies without backoff
- All clients retry simultaneously (thundering herd)

**Symptoms:**
- Cascading failures
- Overwhelming services that are trying to recover

**How to detect:**
- Monitor retry rates and correlate with service recovery time

**How to fix:**
- Use exponential backoff with jitter
- Implement the Circuit Breaker pattern
- Cap retry attempts

**Example scenario:** A downstream service goes offline for 30 seconds. Hundreds of clients immediately retry every 100ms with no backoff, generating 10x normal traffic and preventing the service from recovering for several minutes.

---

## 10. Synchronous I/O

**Description:** Blocking the calling thread while I/O completes.

**Why it happens:**
- Using synchronous APIs for network or disk operations

**Symptoms:**
- Thread pool exhaustion
- Poor scalability under load
- UI freezing in client applications

**How to detect:**
- Monitor thread pool usage and identify blocking calls

**How to fix:**
- Use `async`/`await` patterns
- Use non-blocking I/O and async APIs

**Example scenario:** A web API makes synchronous HTTP calls to three downstream services sequentially. Under load, all thread pool threads are blocked waiting for responses, and the server can no longer accept new requests.

---

> Source: [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
