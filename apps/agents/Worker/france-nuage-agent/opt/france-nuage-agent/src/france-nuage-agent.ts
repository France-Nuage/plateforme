import fs from 'fs';
import util from 'util';
import { ServerInfoCollector } from "./ServerInfoCollector";
import { QuotasManager } from "./QuotasManager";
import { ApiSender } from "./ApiSender";

const logFile = fs.createWriteStream('/var/log/france-nuage-agent/metrics.logs', { flags: 'a' });const logStdout = process.stdout;
console.log = function (message: any) {
    logFile.write(util.format(message) + '\n');
    logStdout.write(util.format(message) + '\n');
};

console.error = console.log;

const serverInfoEndpoint = "http://localhost:3333/api/v1/infrastructure/metrics";
const quotasEndpoint = "http://localhost:3333/api/v1/infrastructure/metrics/get_utilisation/";

const serverInfoCollector = new ServerInfoCollector();
const quotasManager = new QuotasManager();

const serverInfoSender = new ApiSender(serverInfoEndpoint);
const quotasSender = new ApiSender(quotasEndpoint);

async function sendData() {
    const serverInfo = serverInfoCollector.collect();
    const quotasInfo = quotasManager.collectQuotas();

    await serverInfoSender.send(serverInfo);
    await quotasSender.send(quotasInfo);
}

// Send data immediately and then every 5 seconds
sendData();
setInterval(sendData, 5000);