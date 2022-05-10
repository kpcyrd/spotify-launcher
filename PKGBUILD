# Maintainer: kpcyrd <kpcyrd[at]archlinux[dot]org>

pkgname=spotify-launcher
pkgver=0.0.0
pkgrel=1
pkgdesc='TODO'
url='https://github.com/kpcyrd/spotify-launcher'
arch=('x86_64')
license=('MIT' 'Apache')
depends=('alsa-lib>=1.0.14' 'gtk3' 'libxss' 'desktop-file-utils' 'openssl' 'nss' 'at-spi2-atk' 'libcurl-gnutls' 'libsm')
makedepends=('cargo')

build() {
  cd ..
  cargo build --release --locked
}

package() {
  cd ..
  install -Dm 755 -t "${pkgdir}/usr/bin" \
    target/release/spotify-launcher
  install -Dm 644 data/pubkey_5E3C45D7B312C643.gpg \
    "${pkgdir}/usr/share/spotify-launcher/keyring.pgp"
}

# vim: ts=2 sw=2 et:
