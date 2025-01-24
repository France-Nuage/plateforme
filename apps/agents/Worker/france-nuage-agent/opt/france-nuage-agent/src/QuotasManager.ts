import os from "os";
import { execSync } from "child_process";
export type QuotasInfo = {
    cpuUsage: number; // in percentage
    usedMemory: number; // in MB
    totalMemory: number; // in MB
    diskSpace: number; // in GB
};

export class QuotasManager {
    public collectQuotas(): QuotasInfo {
        const totalMemory = Math.round(os.totalmem() / 1024 / 1024); // Convertir en MB
        const usedMemory = Math.round((os.totalmem() - os.freemem()) / 1024 / 1024); // Convertir en MB
        const cpuUsage = this.getCpuUsage();
        const diskSpace = this.getDiskSpace();

        return {
            cpuUsage,
            usedMemory,
            totalMemory,
            diskSpace,
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

        const idle = totalIdle / cpus.length;
        const total = totalTick / cpus.length;
        return Math.round((1 - idle / total) * 100);
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
