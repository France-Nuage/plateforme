"""
Todo:
1 - finir ce qui à faire sur le proxmox (script tourne quand le proxomox est On )
2 -finir la route api
3 - push dans mimir
"""

import platform
import socket
import subprocess
import time

import psutil
import requests

API_URL = "http://localhost:3333/api/v1/infrastructure/metrics"


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
        # Use dpkg-query to list installed packages
        result = subprocess.run(
            ["dpkg-query", "-W", "-f=${binary:Package}\n"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True
        )
        # Split the output into a dict of packages
        packages = {pkg: None for pkg in result.stdout.splitlines()}
        return packages
    except subprocess.CalledProcessError as e:
        print(f"Error while listing packages: {e.stderr}")
        return []



def monitor_changes(interval=5):
    previous_stats = get_server_info()
    while True:
        time.sleep(interval)
        current_stats = get_server_info()
        for key in current_stats:
            if current_stats[key] != previous_stats[key]:
                print(f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] {key}: {current_stats[key]}")
                try:
                    requests.post(API_URL, json=current_stats)
                except requests.exceptions.RequestException as e:
                    print(f"Error while sending info to API: {e}")
                return current_stats
                #TODO : send_info_to_api(current_stats)

        previous_stats = current_stats


if __name__ == "__main__":
    monitor_changes()
