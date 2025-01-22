const os = require('os');
const fs = require('fs');
const axios = require('axios');
const childProcess = require('child_process');
const process = require('process');

class MonitoringAgent {
    constructor(apiUrl, interval = 5) {
        this.apiUrl = apiUrl;
        this.interval = interval * 1000; // Convert to milliseconds
        this.previousStats = {};
    }

    async checkApiAvailability() {
        try {
            const response = await axios.get(this.apiUrl, { timeout: 5000 });
            if (response.status === 200) {
                console.info("API is reachable.");
            } else {
                console.error(`API is not reachable. Status code: ${response.status}`);
            }
        } catch (error) {
            console.error(`Error while checking API: ${error.message}`);
            process.exit(1);
        }
    }

    getServerInfo() {
        return {
            ipAddress: this.getIpAddress(),
            hostname: os.hostname(),
            totalMemory: os.totalmem(),
            cpuCount: os.cpus().length,
            diskSpace: this.getDiskSpace(),
            os: os.type(),
            osVersion: os.release(),
            installedPackages: this.listInstalledPackages()
        };
    }

    getIpAddress() {
        const interfaces = os.networkInterfaces();
        for (const interfaceName in interfaces) {
            for (const net of interfaces[interfaceName]) {
                if (!net.internal && net.family === 'IPv4') {
                    return net.address;
                }
            }
        }
        return '0.0.0.0';
    }

    getDiskSpace() {
        const stat = fs.statSync('/');
        return stat.blocks * stat.blksize;
    }

    listInstalledPackages() {
        try {
            const result = childProcess.execSync("dpkg-query -W -f='${binary:Package}\n'", { encoding: 'utf8' });
            return result.split('\n').filter(Boolean);
        } catch (error) {
            console.error(`Error while listing packages: ${error.message}`);
            return [];
        }
    }

    async sendMetrics(data) {
        try {
            const response = await axios.post(this.apiUrl, data);
            if (response.status === 200) {
                console.info("Data sent successfully.");
            } else {
                console.error(`Failed to send data. Status code: ${response.status}`);
            }
        } catch (error) {
            console.error(`Error while sending data: ${error.message}`);
        }
    }

    async monitorChanges() {
        this.previousStats = this.getServerInfo();
        setInterval(async () => {
            const currentStats = this.getServerInfo();
            for (const key in currentStats) {
                if (currentStats[key] !== this.previousStats[key]) {
                    console.info(`[${new Date().toISOString()}] ${key} changed: ${currentStats[key]}`);
                    await this.sendMetrics({ [key]: currentStats[key] });
                }
            }
            this.previousStats = currentStats;
        }, this.interval);
    }

    start() {
        console.info("Monitoring Agent is starting...");
        this.checkApiAvailability().then(() => this.monitorChanges());
    }
}

// Initialize and start the agent
const apiUrl = "http://localhost:3333/api/v1/infrastructure/metrics";
const interval = 5; // in seconds
const agent = new MonitoringAgent(apiUrl, interval);

process.on('SIGINT', () => {
    console.info("Agent shutting down...");
    process.exit(0);
});

process.on('SIGTERM', () => {
    console.info("Agent shutting down...");
    process.exit(0);
});

agent.start();
