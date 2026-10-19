#!/bin/sh
# Runs only in a disposable Ubuntu container; never execute on the host.
set -eu
[ -f /.dockerenv ] || { echo 'This test requires a disposable Docker container.' >&2; exit 1; }
export DEBIAN_FRONTEND=noninteractive
export PATH=/usr/sbin:/usr/bin:/sbin:/bin
set -- /packages/gscan_*_amd64.deb
[ "$#" -eq 1 ]
package=$1
version=$(dpkg-deb -f "$package" Version)
upstream=${version%-1}
upstream=$(printf '%s' "$upstream" | tr '~' '-')
! command -v git
apt-get update
# Seed an older package version, then exercise apt's upgrade path.
dpkg-deb -R "$package" /tmp/old-package
sed -i 's/^Version:.*/Version: 0.0.0-1/' /tmp/old-package/DEBIAN/control
dpkg-deb --root-owner-group --build /tmp/old-package /tmp/gscan-old.deb
apt-get install -y /tmp/gscan-old.deb
[ "$(dpkg-query -W -f='${Version}' gscan)" = 0.0.0-1 ]
apt-get install -y "$package"
[ "$(dpkg-query -W -f='${Version}' gscan)" = "$version" ]
[ "$(command -v gscan)" = /usr/bin/gscan ]
[ "$(gscan --version)" = "gscan $upstream" ]
command -v git
! command -v cargo
! command -v rustc
mkdir -p /tmp/projects
git init -b main /tmp/projects/fixture
printf 'untracked\n' > /tmp/projects/fixture/file
gscan --dirty /tmp/projects | grep fixture
# Package-manager reinstall must be clean and retain unrelated data.
apt-get install --reinstall -y "$package"
[ "$(gscan --version)" = "gscan $upstream" ]
apt-get remove -y gscan
[ ! -e /usr/bin/gscan ]
[ ! -e /usr/share/doc/gscan ]
[ -f /tmp/projects/fixture/file ]
command -v git
