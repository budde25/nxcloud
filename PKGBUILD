# Maintainer: Ethan Budd <budde25@protonmail.com>
pkgname=nxcloud
pkgver=0.2.0
pkgrel=1
makedepends=('rust' 'cargo')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
pkgdesc="A client for interacting with your NextCloud server"
url="https://github.com/budde25/nextcloud-client-cli"
license=('GPL-3.0-or-later')

build() {
    return 0
}

package() {
    cd $srcdir
    cargo install --root="$pkgdir" --git=https://github.com/budde25/nextcloud-client-cli
}
