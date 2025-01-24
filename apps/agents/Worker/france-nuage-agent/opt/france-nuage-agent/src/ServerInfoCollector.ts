import os from "os";
import { execSync } from "child_process";

export type ServerInfo = {
    ipAddress: string;
    hostname: string;
    totalMemory: number; // in MB
    cpuCount: number;
    diskSpace: number; // in GB
    os: string;
    osVersion: string;
    installedPackages: string[];
};

export class ServerInfoCollector {
    public collect(): ServerInfo {
        const ipAddress = execSync("hostname -I").toString().trim();
        const hostname = os.hostname();
        const totalMemory = Math.round(os.totalmem() / 1024 / 1024); // Convertir en MB
        const cpuCount = os.cpus().length;
        const diskSpace = this.getDiskSpace(); // Récupérer l'espace disque
        const osType = os.type();
        const osVersion = os.release();
        const installedPackages = this.getInstalledPackages();

        return {
            ipAddress,
            hostname,
            totalMemory,
            cpuCount,
            diskSpace,
            os: osType,
            osVersion,
            installedPackages,
        };
    }

    private getInstalledPackages(): string[] {
        try {
            const packages = execSync("dpkg-query -W -f='${binary:Package}\n'")
                .toString()
                .trim()
                .split("\n");
            return packages;
        } catch (error) {
            console.error("Failed to fetch installed packages:", error);
            return [];
        }
    }

    private getDiskSpace(): number {
        try {
            const diskSpace = parseInt(
                execSync("df -k --output=size / | tail -1").toString().trim()
            );
            return Math.round(diskSpace / 1024 / 1024); // Convertir en GB
        } catch (error) {
            console.error("Failed to fetch disk space:", error);
            return 0;
        }
    }
}
