FROM debian:bookworm

ENV container=docker
ENV DEBIAN_FRONTEND=noninteractive

# TODO: ensure apparmor, kvm, lxc and cgroups exist (I need my personal computer for coding this)
# NOTE: wont work on WSL2, tricky on MacOS

# Install systemd
RUN apt update && \
    apt install -y wget systemd systemd-sysv

RUN echo "deb [arch=amd64] http://download.proxmox.com/debian/pve bookworm pve-no-subscription" > /etc/apt/sources.list.d/pve-install-repo.list
RUN wget https://enterprise.proxmox.com/debian/proxmox-release-bookworm.gpg -O /etc/apt/trusted.gpg.d/proxmox-release-bookworm.gpg
RUN apt-get update && apt-get full-upgrade -y
RUN apt-get install -y proxmox-ve postfix open-iscsi chrony

CMD ["/lib/systemd/systemd"]
