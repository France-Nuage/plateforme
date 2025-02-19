/**
 * France Nuage Monitoring Agent
 *
 * This script collects system metrics and server information and sends them to a centralized API.
 * It includes retry mechanisms, timeout handling, and comprehensive error logging.
 */

// Required Node.js built-in modules
import * as os from "os"; // Operating system related operations
import * as fs from "fs"; // File system operations
import * as child_process from "child_process"; // For executing system commands
import axios from "axios"; // HTTP client for API requests
import dotenv from "dotenv"; // Environment variables management

// Initialize environment variables from Docker Compose
dotenv.config();

// Validate required environment variables
if (!process.env.API_URL || !process.env.CF_ID || !process.env.CF_SECRET) {
  throw new Error("Missing required environment variables.");
}

// Global configuration constants
const BASE_API_URL = process.env.API_URL;
const API_URL = BASE_API_URL + "/api/v1/infrastructure/metrics";
const INTERVAL = process.env.INTERVAL; // Metric collection interval
const METRICS_RETENTION_DAYS = 30; // Historical data retention period
const MAX_RETRIES = 3; // Maximum retry attempts for operations
const RETRY_DELAY = 5000; // Delay between retries (in milliseconds)
const REQUEST_TIMEOUT = 10000; // API request timeout (in milliseconds)

// API request headers configuration
const API_HEADERS = {
  "CF-Access-Client-Id": process.env.CF_ID,
  "CF-Access-Client-Secret": process.env.CF_SECRET,
  "Content-Type": "application/json",
};

/**
 * Interface defining server information structure
 */
interface ServerInfo {
  ip_address: string; // Server IP address
  hostname: string; // Server hostname
  total_memory: number; // Total system memory
  cpu_count: number; // Number of CPU cores
  disk_space: number; // Total disk space
  os: string; // Operating system type
  os_version: string; // OS version
  installed_packages: string[]; // List of installed packages
  quotas: QuotasInfo; // Resource quotas and usage
}

/**
 * Interface for metric data structure
 */
interface MetricData {
  [key: string]: string | number | string[] | QuotasInfo;
}

/**
 * Interface defining resource quota information
 */
interface QuotasInfo {
  cpuUsage: number; // CPU usage percentage
  usedMemory: number; // Used memory in bytes
  totalMemory: number; // Total memory in bytes
  diskSpace: number; // Total disk space
  memoryUsagePercent: number; // Memory usage percentage
  diskUsagePercent: number; // Disk usage percentage
}

/**
 * Logger class for consistent log formatting and output
 */
class Logger {
  /**
   * Formats log messages with timestamp and level
   */
  private static formatMessage(level: string, message: string): string {
    return `[${new Date().toISOString()}] ${level}: ${message}`;
  }

  /**
   * Logs info level messages
   * Handles both string and object messages
   */
  static info(message: string | object): void {
    if (typeof message === "object") {
      console.info(this.formatMessage("INFO", JSON.stringify(message)));
    } else {
      console.info(this.formatMessage("INFO", message));
    }
  }

  /**
   * Logs error level messages
   */
  static error(message: string): void {
    console.error(this.formatMessage("ERROR", message));
  }

  /**
   * Logs warning level messages
   */
  static warn(message: string): void {
    console.warn(this.formatMessage("WARN", message));
  }
}

/**
 * Utility function to create a promise-based delay
 * @param ms - Milliseconds to sleep
 */
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Generic retry wrapper for async operations
 * @param operation - Async function to retry
 * @param retries - Maximum number of retry attempts
 * @param delay - Delay between retries in milliseconds
 * @param operationName - Name of the operation for logging
 */
async function withRetry<T>(
  operation: () => Promise<T>,
  retries = MAX_RETRIES,
  delay = RETRY_DELAY,
  operationName = "Operation",
): Promise<T> {
  let lastError: Error;

  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      if (attempt === retries) {
        Logger.error(
          `${operationName} failed after ${retries} attempts. Last error: ${lastError.message}`,
        );
        throw lastError;
      }

      Logger.warn(
        `${operationName} failed (attempt ${attempt}/${retries}). Retrying in ${delay / 1000}s...`,
      );
      await sleep(delay);
    }
  }

  throw lastError!;
}

/**
 * Collects system metrics with retry mechanism
 * Returns CPU, memory, and disk usage statistics
 */
async function collectMetrics(): Promise<MetricData> {
  return withRetry(
    async () => {
      // Calculate memory metrics
      const totalMemory = os.totalmem();
      const freeMemory = os.freemem();
      const usedMemory = totalMemory - freeMemory;
      const memoryUsagePercent = (usedMemory / totalMemory) * 100;

      // Calculate CPU usage
      const cpuUsage = (os.loadavg()[0] * 100) / os.cpus().length;

      // Get disk usage using df command
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
 * Collects detailed server information with retry mechanism
 * Returns comprehensive system information including hardware and OS details
 */
async function getServerInfo(): Promise<ServerInfo> {
  return withRetry(
    async () => {
      // Get network interface information
      const networkInterfaces = os.networkInterfaces();
      const eth0 = networkInterfaces["eth0"];

      if (!eth0) {
        throw new Error("Network interface eth0 not found");
      }

      // Collect basic system information
      const ip_address = eth0[0].address;
      const hostname = os.hostname();
      const total_memory = os.totalmem();
      const cpu_count = os.cpus().length;
      const disk_space = fs.statSync("/").size;
      const os_type = os.type();
      const os_version = os.release();

      // Get installed packages using npm
      const installed_packages = child_process
        .execSync("npm list -g --depth=0")
        .toString()
        .split("\n");

      // Calculate resource quotas
      const quotas: QuotasInfo = {
        cpuUsage: (os.loadavg()[0] * 100) / cpu_count,
        usedMemory: total_memory - os.freemem(),
        totalMemory: total_memory,
        diskSpace: disk_space,
        memoryUsagePercent:
          ((total_memory - os.freemem()) / total_memory) * 100,
        diskUsagePercent: parseFloat(
          child_process
            .execSync("df -h /")
            .toString()
            .split("\n")[1]
            .split(/\s+/)[4],
        ),
      };

      return {
        ip_address,
        hostname,
        total_memory,
        cpu_count,
        disk_space,
        os: os_type,
        os_version,
        installed_packages,
        quotas,
      };
    },
    MAX_RETRIES,
    RETRY_DELAY,
    "Server info collection",
  );
}

/**
 * Sends collected metrics to the API with retry and timeout
 * @param data - Metric data to send
 */
async function sendMetrics(data: MetricData): Promise<void> {
  await withRetry(
    async () => {
      const response = await axios.post(API_URL, data, {
        headers: API_HEADERS,
        timeout: REQUEST_TIMEOUT,
      });

      if (response.status !== 200) {
        throw new Error(`Failed to send data. Status code: ${response.status}`);
      }

      Logger.info("Data sent successfully.");
    },
    MAX_RETRIES,
    RETRY_DELAY,
    "Metrics sending",
  );
}

/**
 * Verifies API availability with retry and timeout
 * Exits process if API is not available after all retries
 */
async function checkApiAvailability(): Promise<boolean> {
  try {
    await withRetry(
      async () => {
        const response = await axios.get(API_URL, {
          headers: API_HEADERS,
          timeout: REQUEST_TIMEOUT,
        });

        if (response.status !== 200) {
          throw new Error(
            `API is not reachable. Status code: ${response.status}`,
          );
        }

        Logger.info("API is reachable.");
      },
      MAX_RETRIES,
      RETRY_DELAY,
      "API availability check",
    );
    return true;
  } catch (error) {
    Logger.error(
      `API availability check failed: ${error instanceof Error ? error.message : String(error)}`,
    );
    return false;
  }
}

/**
 * Main function that orchestrates the monitoring agent
 * - Checks API availability
 * - Collects initial server information
 * - Starts periodic metric collection
 */
async function main(): Promise<void> {
  try {
    Logger.info("France Nuage Agent is starting...");
    await checkApiAvailability();

    // Collect and log initial server information
    const serverInfo = await getServerInfo();
    Logger.info({ message: "Server information collected", data: serverInfo });

    // Start periodic metrics collection loop
    setInterval(async () => {
      try {
        const realMetrics: MetricData = await collectMetrics();
        await sendMetrics(realMetrics);
      } catch (error) {
        Logger.error(
          `Error in metrics collection cycle: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }, Number(INTERVAL));
  } catch (error) {
    Logger.error(
      `Error in main function: ${error instanceof Error ? error.message : String(error)}`,
    );
    process.exit(1);
  }
}

// Start the application with error handling
main().catch((error) => {
  Logger.error(
    `Unhandled error: ${error instanceof Error ? error.message : String(error)}`,
  );
  process.exit(1);
});
