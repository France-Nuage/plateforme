import * as os from 'os';
import * as fs from 'fs';
import * as child_process from 'child_process';
import axios from 'axios';

// Types
interface ServerInfo {
    ip_address: string;
    hostname: string;
    total_memory: number;
    cpu_count: number;
    disk_space: number;
    os: string;
    os_version: string;
    installed_packages: string[];
    quotas: QuotasInfo;
}

interface MetricData {
    [key: string]: string | number | string[] | QuotasInfo;
}

interface QuotasInfo {
    cpuUsage: number;      // in percentage
    usedMemory: number;    // in MB
    totalMemory: number;   // in MB
    diskSpace: number;     // in GB
    memoryUsagePercent: number; // in percentage
    diskUsagePercent: number;   // in percentage
}

// Configuration
const API_URL = 'http://localhost:3333/api/v1/infrastructure/metrics';
const INTERVAL = 5000; // Interval in milliseconds
const METRICS_RETENTION_DAYS = 30; // Number of days to keep historical metrics

class Logger {
    private static formatMessage(level: string, message: string): string {
        return `[${new Date().toISOString()}] ${level}: ${message}`;
    }

    static info(message: string | object): void {
        if (typeof message === 'object') {
            console.info(this.formatMessage('INFO', JSON.stringify(message)));
        } else {
            console.info(this.formatMessage('INFO', message));
        }
    }

    static error(message: string): void {
        console.error(this.formatMessage('ERROR', message));
    }

    static warn(message: string): void {
        console.warn(this.formatMessage('WARN', message));
    }
}

class QuotasManager {
    private lastCpuInfo: { idle: number; total: number } | null = null;

    public collectQuotas(): QuotasInfo {
        const totalMemoryMB = Math.round(os.totalmem() / 1024 / 1024);
        const usedMemoryMB = Math.round((os.totalmem() - os.freemem()) / 1024 / 1024);
        const cpuUsage = this.getCpuUsage();
        const diskSpace = this.getDiskSpace();
        const memoryUsagePercent = Math.round((usedMemoryMB / totalMemoryMB) * 100);
        const diskUsagePercent = this.getDiskUsagePercent();

        return {
            cpuUsage,
            usedMemory: usedMemoryMB,
            totalMemory: totalMemoryMB,
            diskSpace,
            memoryUsagePercent,
            diskUsagePercent
        };
    }

    private getCpuUsage(): number {
        const cpus = os.cpus();
        let totalIdle = 0;
        let totalTick = 0;

        cpus.forEach((cpu) => {
            for (const type in cpu.times) {
                totalTick += cpu.times[type as keyof typeof cpu.times];
            }
            totalIdle += cpu.times.idle;
        });

        const currentInfo = {
            idle: totalIdle / cpus.length,
            total: totalTick / cpus.length
        };

        if (this.lastCpuInfo) {
            const idleDiff = currentInfo.idle - this.lastCpuInfo.idle;
            const totalDiff = currentInfo.total - this.lastCpuInfo.total;
            const usage = Math.round((1 - idleDiff / totalDiff) * 100);
            this.lastCpuInfo = currentInfo;
            return usage;
        }

        this.lastCpuInfo = currentInfo;
        return 0;
    }

    private getDiskSpace(): number {
        try {
            const diskSpace = parseInt(
                child_process.execSync("df -k --output=size / | tail -1").toString().trim()
            );
            return Math.round(diskSpace / 1024 / 1024); // Convert to GB
        } catch (error) {
            Logger.error(`Failed to fetch disk space: ${error instanceof Error ? error.message : String(error)}`);
            return 0;
        }
    }

    private getDiskUsagePercent(): number {
        try {
            const output = child_process.execSync("df -k / | tail -1").toString().trim();
            const [,,,, usagePercent] = output.split(/\s+/);
            return parseInt(usagePercent.replace('%', ''));
        } catch (error) {
            Logger.error(`Failed to fetch disk usage percentage: ${error instanceof Error ? error.message : String(error)}`);
            return 0;
        }
    }
}

class MetricsStorage {
    private readonly storagePath: string = './metrics_history.json';

    public async saveMetrics(metrics: MetricData): Promise<void> {
        try {
            let history: MetricData[] = [];
            if (fs.existsSync(this.storagePath)) {
                const data = await fs.promises.readFile(this.storagePath, 'utf8');
                history = JSON.parse(data);
            }

            history.push({
                ...metrics,
                timestamp: new Date().toISOString()
            });

            // Clean up old metrics
            const cutoffDate = new Date();
            cutoffDate.setDate(cutoffDate.getDate() - METRICS_RETENTION_DAYS);
            history = history.filter(metric =>
                new Date(metric.timestamp as string) > cutoffDate
            );

            await fs.promises.writeFile(
                this.storagePath,
                JSON.stringify(history, null, 2)
            );
        } catch (error) {
            Logger.error(`Failed to save metrics: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
}

async function checkApiAvailability(): Promise<void> {
    try {
        const response = await axios.get(API_URL, { timeout: 5000 });
        if (response.status === 200) {
            Logger.info('API is reachable.');
        } else {
            Logger.error(`API is not reachable. Status code: ${response.status}`);
        }
    } catch (e) {
        Logger.error(`Error while checking API: ${e instanceof Error ? e.message : String(e)}`);
        process.exit(1);
    }
}

async function listInstalledPackages(): Promise<string[]> {
    return new Promise((resolve) => {
        if (process.platform !== 'linux') {
            resolve([]);
            return;
        }

        child_process.exec('dpkg-query -W -f=${binary:Package}\n', (error, stdout) => {
            if (error) {
                Logger.error(`Error while listing packages: ${error.message}`);
                resolve([]);
                return;
            }
            resolve(stdout.split('\n').filter(Boolean));
        });
    });
}

class ServerMonitor {
    private quotasManager: QuotasManager;
    private metricsStorage: MetricsStorage;

    constructor() {
        this.quotasManager = new QuotasManager();
        this.metricsStorage = new MetricsStorage();
    }

    async getServerInfo(): Promise<ServerInfo> {
        const networkInterfaces = os.networkInterfaces();
        const ipAddress = Object.values(networkInterfaces)
            .flat()
            .find(interface_ => !interface_?.internal && interface_?.family === 'IPv4')
            ?.address || 'unknown';

        const quotas = this.quotasManager.collectQuotas();

        const info: ServerInfo = {
            ip_address: ipAddress,
            hostname: os.hostname(),
            total_memory: os.totalmem(),
            cpu_count: os.cpus().length,
            disk_space: quotas.diskSpace * 1024 * 1024 * 1024, // Convert GB to bytes
            os: os.platform(),
            os_version: os.release(),
            installed_packages: await listInstalledPackages(),
            quotas
        };

        Logger.info(info);
        return info;
    }

    async sendMetrics(data: MetricData): Promise<void> {
        try {
            const response = await axios.post(API_URL, data);
            if (response.status === 200) {
                await this.metricsStorage.saveMetrics(data);
                Logger.info('Data sent successfully.');
            } else {
                Logger.error(`Failed to send data. Status code: ${response.status}`);
                Logger.error(`Response received: ${response.data}`);
            }
        } catch (e) {
            Logger.error(`Error while sending data: ${e instanceof Error ? e.message : String(e)}`);
        }
    }

    async monitorChanges(interval: number = INTERVAL): Promise<never> {
        let previousStats = await this.getServerInfo();

        while (true) {
            await new Promise(resolve => setTimeout(resolve, interval));
            const currentStats = await this.getServerInfo();

            // Always send quotas metrics as they're likely to change
            await this.sendMetrics({ quotas: currentStats.quotas });

            // Check other metrics for changes
            for (const [key, value] of Object.entries(currentStats)) {
                if (key !== 'quotas' &&
                    JSON.stringify(value) !== JSON.stringify(previousStats[key as keyof ServerInfo])) {
                    Logger.info(`[${new Date().toISOString()}] ${key} changed: ${JSON.stringify(value)}`);
                    await this.sendMetrics({ [key]: value });
                }
            }
            previousStats = currentStats;

            // Alert on high resource usage
            this.checkResourceThresholds(currentStats.quotas);
        }
    }

    private checkResourceThresholds(quotas: QuotasInfo): void {
        if (quotas.cpuUsage > 90) {
            Logger.warn(`High CPU usage detected: ${quotas.cpuUsage}%`);
        }
        if (quotas.memoryUsagePercent > 85) {
            Logger.warn(`High memory usage detected: ${quotas.memoryUsagePercent}%`);
        }
        if (quotas.diskUsagePercent > 85) {
            Logger.warn(`High disk usage detected: ${quotas.diskUsagePercent}%`);
        }
    }
}

function handleExit(signal: string): void {
    Logger.info(`Agent shutting down... (Signal: ${signal})`);
    process.exit(0);
}

async function main(): Promise<void> {
    process.on('SIGINT', () => handleExit('SIGINT'));
    process.on('SIGTERM', () => handleExit('SIGTERM'));

    Logger.info('France Nuage Agent is starting...');
    await checkApiAvailability();

    const monitor = new ServerMonitor();
    await monitor.monitorChanges();
}

main().catch(error => {
    Logger.error(`Unhandled error: ${error instanceof Error ? error.message : String(error)}`);
    process.exit(1);
});