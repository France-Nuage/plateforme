import platform
import socket
import subprocess
import time
import psutil
import requests
import signal
import sys
import logging
from systemd.journal import JournalHandler

# Configuration
API_URL = "http://localhost:3333/api/v1/infrastructure/metrics"
INTERVAL = 5  # Intervalle en secondes pour surveiller les changements

# Configure logging
logger = logging.getLogger("FranceNuageAgent")
journal_handler = JournalHandler()
journal_handler.setLevel(logging.INFO)
logger.addHandler(journal_handler)
logger.setLevel(logging.INFO)

def check_api_availability():
    try:
        response = requests.get(API_URL, timeout=5)
        if response.status_code == 200:
            logger.info("API is reachable.")
        else:
            logger.error(f"API is not reachable. Status code: {response.status_code}")
    except requests.exceptions.RequestException as e:
        logger.error(f"Error while checking API: {e}")
        exit(1)

def get_server_info():
    info = {
        "ip_address": socket.gethostbyname(socket.gethostname()),
        "hostname": socket.gethostname(),
        "total_memory": psutil.virtual_memory().total,
        "cpu_count": psutil.cpu_count(logical=True),
        "disk_space": psutil.disk_usage('/').total,
        "os": platform.system(),
        "os_version": platform.release(),
        "installed_packages": list_installed_packages()
    }
    logger.info(info)
    return info

def list_installed_packages():
    try:
        result = subprocess.run(
            ["dpkg-query", "-W", "-f=${binary:Package}\n"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True
        )
        return result.stdout.splitlines()
    except subprocess.CalledProcessError as e:
        logger.error(f"Error while listing packages: {e.stderr}")
        return []

def send_metrics(data):
    try:
        response = requests.post(API_URL, json=data)
        if response.status_code == 200:
            logger.info("Data sent successfully.")
        else:
            logger.error(f"Failed to send data. Status code: {response.status_code}")
            logger.error(f"Response received: {response.text}")
    except requests.exceptions.RequestException as e:
        logger.error(f"Error while sending data: {e}")

def monitor_changes(interval=INTERVAL):
    previous_stats = get_server_info()
    while True:
        time.sleep(interval)
        current_stats = get_server_info()
        for key in current_stats:
            if current_stats[key] != previous_stats[key]:
                logger.info(f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] {key} changed: {current_stats[key]}")
                send_metrics({key: current_stats[key]})
        previous_stats = current_stats

def handle_exit(signal_received, frame):
    logger.info("Agent shutting down...")
    sys.exit(0)

if __name__ == "__main__":
    signal.signal(signal.SIGINT, handle_exit)
    signal.signal(signal.SIGTERM, handle_exit)
    logger.info("France Nuage Agent is working!")
    check_api_availability()
    monitor_changes()
