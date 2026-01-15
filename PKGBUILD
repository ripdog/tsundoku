# Maintainer: ripdog <ripdog@users.noreply.github.com>
pkgname=tsundoku
pkgver=1.0.0
pkgrel=1
pkgdesc="Japanese web novel downloader and translator supporting Syosetu, Kakuyomu, and Pixiv"
arch=('x86_64' 'aarch64')
url="https://github.com/ripdog/tsundoku"
license=('GPL-3.0-or-later')
depends=('gcc-libs')
makedepends=('rust' 'cargo' 'cmake')
# Build from local working tree (no network fetch).
# Useful for `makepkg --noextract` in a git checkout.
source=("local://$pkgname-$pkgver")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')" --manifest-path=Cargo.toml
}

build() {
    cd "$srcdir"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    # Clear LDFLAGS and CFLAGS that might interfere with aws-lc-sys
    unset LDFLAGS
    unset CFLAGS
    unset CXXFLAGS
    cargo build --frozen --release --all-features
}

check() {
    cd "$srcdir"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features
}

package() {
    cd "$srcdir"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
