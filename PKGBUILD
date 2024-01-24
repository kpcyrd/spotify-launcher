# Maintainer: kpcyrd <kpcyrd[at]archlinux[dot]org>

pkgname=spotify-launcher
pkgver=0.0.0
pkgrel=1
pkgdesc="Client for spotify's apt repository in Rust for Arch Linux"
url='https://github.com/kpcyrd/spotify-launcher'
arch=('x86_64')
license=('MIT' 'Apache')
depends=('sequoia-sqv' 'zenity' 'alsa-lib>=1.0.14' 'gtk3' 'libxss' 'desktop-file-utils' 'openssl' 'nss' 'at-spi2-atk' 'libcurl-gnutls' 'libsm')
makedepends=('cargo')
backup=('etc/spotify-launcher.conf')

build() {
  cd ..
  cargo build --release --locked
}

package() {
  cd ..
  install -Dm 755 -t "${pkgdir}/usr/bin" \
    target/release/spotify-launcher
  install -Dm 644 data/pubkey_6224F9941A8AA6D1.gpg \
    "${pkgdir}/usr/share/spotify-launcher/keyring.pgp"

  install -Dm644 contrib/spotify-launcher.desktop -t "${pkgdir}/usr/share/applications"
  install -Dm644 contrib/icons/spotify-linux-512.png "${pkgdir}/usr/share/pixmaps/spotify-launcher.png"
  install -Dm644 contrib/spotify-launcher.conf -t "${pkgdir}/etc"

  for size in 22 24 32 48 64 128 256 512; do
    install -Dm644 "contrib/icons/spotify-linux-${size}.png" \
      "${pkgdir}/usr/share/icons/hicolor/${size}x${size}/apps/spotify-launcher.png"
  done
}

# vim: ts=2 sw=2 et:
