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
 fpm -s dir -t deb -n france-nuage-agent -v 1.1.X     --prefix /  --after-install ./france-nuage-agent/DEBIAN/postinst   --before-remove ./france-nuage-agent/DEBIAN/prerm  ./france-nuage-agent/opt/france-nuage-agent/=/opt/france-nuage-agent/    ./france-nuage-agent/usr/lib/systemd/system/=/usr/lib/systemd/system/
```
5. install the package with `dpkg -i france-nuage-agent_X.X.X_all.deb`