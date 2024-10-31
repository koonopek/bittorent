const axios = require("axios");
const { performance } = require("perf_hooks");

// Configure axios instance with keep-alive
const https = require("https");
const agent = new https.Agent({
  keepAlive: true,
  maxSockets: 50,
});

const axiosInstance = axios.create({
  httpsAgent: agent,
});

// Test configuration
const TEST_URL = "https://api.example.com/endpoint"; // Replace with your test endpoint
const TOTAL_REQUESTS = 1000;
const CONCURRENT_REQUESTS = 50;

async function makeRequest(client, url) {
  const start = performance.now();
  try {
    if (client === "axios") {
      await axiosInstance.get(url);
    } else {
      await fetch(url);
    }
    return performance.now() - start;
  } catch (error) {
    console.error(`Error: ${error.message}`);
    return null;
  }
}

async function runBatch(client, batchSize) {
  const promises = Array(batchSize)
    .fill()
    .map(() => makeRequest(client, TEST_URL));
  const results = await Promise.all(promises);
  return results.filter((r) => r !== null);
}

async function benchmark() {
  console.log(
    `Running benchmark with ${TOTAL_REQUESTS} total requests, ${CONCURRENT_REQUESTS} concurrent`
  );

  // Benchmark Axios
  const axiosLatencies = [];
  const axiosStart = performance.now();

  for (let i = 0; i < TOTAL_REQUESTS; i += CONCURRENT_REQUESTS) {
    const batchResults = await runBatch(
      "axios",
      Math.min(CONCURRENT_REQUESTS, TOTAL_REQUESTS - i)
    );
    axiosLatencies.push(...batchResults);
  }

  const axiosEnd = performance.now();

  // Benchmark Fetch
  const fetchLatencies = [];
  const fetchStart = performance.now();

  for (let i = 0; i < TOTAL_REQUESTS; i += CONCURRENT_REQUESTS) {
    const batchResults = await runBatch(
      "fetch",
      Math.min(CONCURRENT_REQUESTS, TOTAL_REQUESTS - i)
    );
    fetchLatencies.push(...batchResults);
  }

  const fetchEnd = performance.now();

  // Calculate metrics
  const calculateMetrics = (latencies, totalTime) => ({
    avgLatency: (
      latencies.reduce((a, b) => a + b, 0) / latencies.length
    ).toFixed(2),
    p95Latency: latencies
      .sort((a, b) => a - b)
      [Math.floor(latencies.length * 0.95)].toFixed(2),
    requestsPerSecond: (TOTAL_REQUESTS / (totalTime / 1000)).toFixed(2),
    totalTime: totalTime.toFixed(2),
  });

  const axiosMetrics = calculateMetrics(axiosLatencies, axiosEnd - axiosStart);
  const fetchMetrics = calculateMetrics(fetchLatencies, fetchEnd - fetchStart);

  console.log("\nResults:");
  console.log("\nAxios:");
  console.log(`Average Latency: ${axiosMetrics.avgLatency}ms`);
  console.log(`P95 Latency: ${axiosMetrics.p95Latency}ms`);
  console.log(`Requests/second: ${axiosMetrics.requestsPerSecond}`);
  console.log(`Total Time: ${axiosMetrics.totalTime}ms`);

  console.log("\nFetch:");
  console.log(`Average Latency: ${fetchMetrics.avgLatency}ms`);
  console.log(`P95 Latency: ${fetchMetrics.p95Latency}ms`);
  console.log(`Requests/second: ${fetchMetrics.requestsPerSecond}`);
  console.log(`Total Time: ${fetchMetrics.totalTime}ms`);
}

// Run benchmark
benchmark().catch(console.error);
