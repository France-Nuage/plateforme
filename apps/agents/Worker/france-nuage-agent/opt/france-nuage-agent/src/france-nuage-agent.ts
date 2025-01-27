import fs from 'fs';
import util from 'util';
import { ServerInfoCollector } from "./ServerInfoCollector";
import { QuotasManager } from "./QuotasManager";
import { ApiSender } from "./ApiSender";

console.log('Starting france-nuage-agent service...');

const logFilePath = '/var/log/france-nuage-agent/metrics.logs';
let logFile;
try {
    logFile = fs.createWriteStream(logFilePath, { flags: 'a' });
    console.log(`Log file opened at ${logFilePath}`);
} catch (error) {
    console.error(`Failed to open log file at ${logFilePath}:`, error);
    logFile = fs.createWriteStream('/tmp/france-nuage-agent.log', { flags: 'a' });
    console.log('Fallback log file opened at /tmp/france-nuage-agent.log');
}
const logStdout = process.stdout;
console.log = function (message: any) {
    logFile.write(util.format(message) + '\n');
    logStdout.write(util.format(message) + '\n');
};

console.error = function (message: any) {
    logFile.write(util.format(message) + '\n');
    logStdout.write(util.format(message) + '\n');
};

const serverInfoEndpoint = "http://localhost:3333/api/v1/infrastructure/metrics";
const quotasEndpoint = "http://localhost:3333/api/v1/infrastructure/metrics/get_utilisation/";

console.log('Initializing ServerInfoCollector and QuotasManager...');
const serverInfoCollector = new ServerInfoCollector();
const quotasManager = new QuotasManager();

console.log('Initializing ApiSender for server info and quotas...');
const serverInfoSender = new ApiSender(serverInfoEndpoint);
const quotasSender = new ApiSender(quotasEndpoint);

async function sendData() {
    console.log('Collecting server info...');
    const serverInfo = serverInfoCollector.collect();
    console.log('Server info collected:', serverInfo);

    console.log('Collecting quotas info...');
    const quotasInfo = quotasManager.collectQuotas();
    console.log('Quotas info collected:', quotasInfo);

    console.log('Sending server info...');
    await serverInfoSender.send(serverInfo);
    console.log('Server info sent.');

    console.log('Sending quotas info...');
    await quotasSender.send(quotasInfo);
    console.log('Quotas info sent.');
}

console.log('Sending initial data...');
sendData().then(() => {
    console.log('Initial data sent.');
}).catch(error => {
    console.error('Error sending initial data:', error);
});

console.log('Setting interval to send data every 20 seconds...');
setInterval(() => {
    sendData().catch(error => {
        console.error('Error sending data:', error);
    });
}, 20000);