import os
import platform
import socket
import psutil
import requests

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
    # This function should return a list of installed packages
    # Implementation will vary depending on the package manager (e.g., apt, yum, etc.)
    return ["example-package1", "example-package2"]

def send_info_to_api(info):
    response = requests.post(API_URL, json=info)
    return response.status_code

if __name__ == "__main__":
    server_info = get_server_info()
    status = send_info_to_api(server_info)
    print(f"Data sent to API, status code: {status}")
