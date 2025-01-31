import * as os from 'os';
import * as fs from 'fs';
import * as child_process from 'child_process';
import axios from 'axios';
import dotenv from 'dotenv';

dotenv.config(); // Charger les variables d'environnement depuis Docker Compose
if (process.env.API_URL === null || process.env.CF_ID ===null|| !process.env.CF_SECRET === null) {
    throw new Error('Missing required environment variables.');
}
// Configuration des variables d'environnement provenant de Docker Compose
const BASE_API_URL = process.env.API_URL ;
const API_URL = BASE_API_URL + '/api/v1/infrastructure/metrics' ;
const INTERVAL = 5000; // Intervalle en millisecondes entre chaque envoi de métriques
const METRICS_RETENTION_DAYS = 30; // Nombre de jours de conservation des métriques historiques
const API_HEADERS = {
    'CF-Access-Client-Id': process.env.CF_ID,
    'CF-Access-Client-Secret': process.env.CF_SECRET,
    'Content-Type': 'application/json'
};

/**
 * Interface représentant les informations du serveur
 */
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

/**
 * Interface pour les données de métriques envoyées à l'API
 */
interface MetricData {
    [key: string]: string | number | string[] | QuotasInfo;
}

/**
 * Interface des informations sur l'utilisation des ressources
 */
interface QuotasInfo {
    cpuUsage: number;
    usedMemory: number;
    totalMemory: number;
    diskSpace: number;
    memoryUsagePercent: number;
    diskUsagePercent: number;
}

/**
 * Classe de gestion des logs
 */
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

/**
 * Envoie les métriques collectées à l'API
 * @param data - Données des métriques à envoyer
 */
async function sendMetrics(data: MetricData): Promise<void> {
    try {
        const response = await axios.post(API_URL, data, {headers: API_HEADERS});
        if (response.status === 200) {
            Logger.info('Data sent successfully.');
        } else {
            Logger.error(`Failed to send data. Status code: ${response.status}`);
        }
    } catch (e) {
        Logger.error(`Error while sending data: ${e instanceof Error ? e.message : String(e)}`);
    }
}

/**
 * Vérifie la disponibilité de l'API avant de démarrer l'agent
 */
async function checkApiAvailability(): Promise<void> {
    try {
        const response = await axios.get(API_URL, {timeout: 5000, headers: API_HEADERS});
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

/**
 * Fonction principale qui lance l'agent de monitoring
 */
async function main(): Promise<void> {
    Logger.info('France Nuage Agent is starting...');
    await checkApiAvailability();

    // Envoi des métriques à intervalles réguliers
    setInterval(async () => {
        const sampleMetrics: MetricData = {
            cpuUsage: Math.random() * 100,
            memoryUsagePercent: Math.random() * 100,
            diskUsagePercent: Math.random() * 100,
        };
        await sendMetrics(sampleMetrics);
    }, INTERVAL);
}

// Exécution de la fonction principale avec gestion des erreurs
main().catch(error => {
    Logger.error(`Unhandled error: ${error instanceof Error ? error.message : String(error)}`);
    process.exit(1);
});
