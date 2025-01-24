# how to build 
## Prerequisites
- [Node.js](https://nodejs.org/en/download/)
- [npm](https://www.npmjs.com/get-npm)
- [tsc](https://www.typescriptlang.org/) (typescript compiler) : `npm install -g typescript`
- [fpm](https://fpm.readthedocs.io/en/v1.15.1/) (for building the debian package) : `gem install fpm`
- [ruby](https://www.ruby-lang.org/en/documentation/) (for fpm) : `apt-get install ruby`

## Installation
1. go in the /opt/france-nuage-agent/ folder
2. execute `npm install`
3. execute `tsc` in the /opt/france-nuage-agent/src folder
4. in the production environment, execute (modify the version number) :
```bash
fpm -s dir -t deb -n france-nuage-agent -v X.X.X     --prefix /   ./france-nuage-agent/opt/france-nuage-agent/dist/=/opt/france-nuage-agent/dist/     ./france-nuage-agent/usr/lib/systemd/
system/france-nuage-agent.service=/etc/systemd/system/france-nuage-agent.service    ./france-nuage-agent/DEBIAN/
```
5. install the package with `dpkg -i france-nuage-agent_X.X.X_all.deb`