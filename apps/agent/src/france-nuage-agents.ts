/**
 * France Nuage Monitoring Agent
 *
 * This script collects system metrics and server information and sends them to a centralized API.
 * It includes retry mechanisms, timeout handling, and comprehensive error logging.
 */

// Required Node.js built-in modules
import * as os from "os";
import * as child_process from "child_process";
import axios from "axios";
import dotenv from "dotenv";

// Initialize environment variables from Docker Compose
dotenv.config();

// Validate required environment variables
if (!process.env.API_URL || !process.env.CF_ID || !process.env.CF_SECRET) {
  throw new Error("Missing required environment variables.");
}

// Global configuration constants
const BASE_API_URL = process.env.API_URL;
const API_URL = BASE_API_URL + "/api/v1/infrastructure/metrics";
const INTERVAL = process.env.INTERVAL;
const MAX_RETRIES = 3;
const RETRY_DELAY = 5000;
const REQUEST_TIMEOUT = 10000;

// API request headers configuration
const API_HEADERS = {
  "CF-Access-Client-Id": process.env.CF_ID,
  "CF-Access-Client-Secret": process.env.CF_SECRET,
  "Content-Type": "application/json",
};

/**
 * Logger module with formatted log functions
 */
function formatMessage(level: string, message: any) {
  return `[${new Date().toISOString()}] ${level}: ${message}`;
}

function logInfo(message: any) {
  console.info(
    formatMessage(
      "INFO",
      typeof message === "object" ? JSON.stringify(message) : message,
    ),
  );
}

function logError(message: any) {
  console.error(formatMessage("ERROR", message));
}

function logWarn(message: any) {
  console.warn(formatMessage("WARN", message));
}

export { logInfo, logError, logWarn };

/**
 * Utility function to create a promise-based delay
 */
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Generic retry wrapper for async operations
 */
async function withRetry<T>(
  operation: () => Promise<T>,
  retries = MAX_RETRIES,
  delay = RETRY_DELAY,
  operationName = "Operation",
) {
  let lastError;
  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      if (attempt === retries) {
        logError(
          `${operationName} failed after ${retries} attempts. Last error: ${lastError.message}`,
        );
        throw lastError;
      }
      logWarn(
        `${operationName} failed (attempt ${attempt}/${retries}). Retrying in ${delay / 1000}s...`,
      );
      await sleep(delay);
    }
  }
  throw lastError;
}

/**
 * Collects system metrics with retry mechanism
 */
async function collectMetrics() {
  return withRetry(
    async () => {
      const totalMemory = os.totalmem();
      const freeMemory = os.freemem();
      const usedMemory = totalMemory - freeMemory;
      const memoryUsagePercent = (usedMemory / totalMemory) * 100;
      const cpuUsage = (os.loadavg()[0] * 100) / os.cpus().length;
      const diskUsage = child_process
        .execSync("df -h /")
        .toString()
        .split("\n")[1]
        .split(/\s+/);
      const diskUsagePercent = parseFloat(diskUsage[4]);
      return {
        cpuUsage,
        memoryUsagePercent,
        diskUsagePercent,
        usedMemory,
        totalMemory,
        diskSpace: parseFloat(diskUsage[1]),
      };
    },
    MAX_RETRIES,
    RETRY_DELAY,
    "Metric collection",
  );
}

/**
 * Sends collected metrics to the API with retry and timeout
 */
async function sendMetrics(data: any) {
  await withRetry(
    async () => {
      const response = await axios.post(API_URL, data, {
        headers: API_HEADERS,
        timeout: REQUEST_TIMEOUT,
      });
      if (response.status !== 200) {
        throw new Error(`Failed to send data. Status code: ${response.status}`);
      }
      logInfo("Data sent successfully.");
    },
    MAX_RETRIES,
    RETRY_DELAY,
    "Metrics sending",
  );
}

/**
 * Main function that orchestrates the monitoring agent
 */
async function main() {
  try {
    logInfo("France Nuage Agent is starting...");
    setInterval(async () => {
      try {
        const realMetrics = await collectMetrics();
        await sendMetrics(realMetrics);
      } catch (error) {
        logError(
          `Error in metrics collection cycle: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }, Number(INTERVAL));
  } catch (error) {
    logError(
      `Error in main function: ${error instanceof Error ? error.message : String(error)}`,
    );
    process.exit(1);
  }
}

// Start the application with error handling
main().catch((error) => {
  logError(
    `Unhandled error: ${error instanceof Error ? error.message : String(error)}`,
  );
  process.exit(1);
});
