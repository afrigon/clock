# Maintainer: Alexandre Frigon <clock@frigon.app>
pkgname=clock-git
pkgver=r1.8e55f05
pkgrel=1
pkgdesc="A clock for the terminal: countdown timer, current time, and stopwatch"
arch=('x86_64' 'aarch64')
url="https://github.com/afrigon/clock"
license=('MIT')
depends=('gcc-libs' 'glibc' 'alsa-lib')
makedepends=('git' 'cargo')
provides=('clock')
conflicts=('clock')
source=("clock::git+https://github.com/afrigon/clock.git")
sha256sums=('SKIP')

pkgver() {
  cd clock
  printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

prepare() {
  cd clock
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd clock
  export RUSTUP_TOOLCHAIN=stable
  cargo build --frozen --release
}

check() {
  cd clock
  export RUSTUP_TOOLCHAIN=stable
  cargo test --frozen --release
}

package() {
  cd clock
  install -Dm755 target/release/clock "$pkgdir/usr/bin/clock"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
