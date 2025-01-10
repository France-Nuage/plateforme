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

API_URL = "http://localhost:3333/api/v1/servers"


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


# TODO :  def send_info_to_api(info):
def monitor_changes(interval=5):
    previous_stats = get_server_info()
    while True:
        time.sleep(interval)
        current_stats = get_server_info()
        for key in current_stats:
            if current_stats[key] != previous_stats[key]:
                print(f"{key} changed from {previous_stats[key]} to {current_stats[key]}")
                return current_stats
                #TODO : send_info_to_api(current_stats)

        previous_stats = current_stats


if __name__ == "__main__":
    pass
    # check if
