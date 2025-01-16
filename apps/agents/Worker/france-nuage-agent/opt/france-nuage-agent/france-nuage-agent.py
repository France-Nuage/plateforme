import platform
import socket
import subprocess
import time
import psutil
import requests
import signal
import sys

# Configuration
API_URL = "http://localhost:3333/api/v1/infrastructure/metrics"
INTERVAL = 5  # Intervalle en secondes pour surveiller les changements


def check_api_availability():
    try:
        response = requests.get(API_URL, timeout=5)
        if response.status_code == 200:
            print("API is reachable.")
        else:
            print(f"API is not reachable. Status code: {response.status_code}")
    except requests.exceptions.RequestException as e:
        print(f"Error while checking API: {e}")
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
        print(f"Error while listing packages: {e.stderr}")
        return []


def monitor_changes(interval=INTERVAL):
    previous_stats = get_server_info()
    while True:
        time.sleep(interval)
        current_stats = get_server_info()
        for key in current_stats:
            if current_stats[key] != previous_stats[key]:
                print(f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] {key} changed: {current_stats[key]}")
                try:
                    requests.post(API_URL, json={key: current_stats[key]})
                except requests.exceptions.RequestException as e:
                    print(f"Error while sending info to API: {e}")
        previous_stats = current_stats


def handle_exit(signal_received, frame):
    print("Agent shutting down...")
    sys.exit(0)


if __name__ == "__main__":
    signal.signal(signal.SIGINT, handle_exit)
    signal.signal(signal.SIGTERM, handle_exit)
    print("France Nuage Agent is working!")
    check_api_availability()
    monitor_changes()
