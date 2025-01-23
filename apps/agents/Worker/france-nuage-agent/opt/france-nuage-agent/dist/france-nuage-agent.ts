import { ServerInfoCollector } from "./ServerInfoCollector";
import { QuotasManager } from "./QuotasManager";
import { ApiSender } from "./ApiSender";

(async () => {
    const serverInfoEndpoint = "http://localhost:3333/api/v1/infrastructure/metrics";
    const quotasEndpoint = "https://your-api-endpoint.com/quotas";

    const serverInfoCollector = new ServerInfoCollector();
    const quotasManager = new QuotasManager();

    const serverInfo = serverInfoCollector.collect();
    const quotasInfo = quotasManager.collectQuotas();

    const serverInfoSender = new ApiSender(serverInfoEndpoint);
    const quotasSender = new ApiSender(quotasEndpoint);

    await serverInfoSender.send(serverInfo);
    await quotasSender.send(quotasInfo);
})();
