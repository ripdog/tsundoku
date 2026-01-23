# Maintainer: ripdog <ripdog@users.noreply.github.com>
pkgname=tsundoku
pkgver=1.0.4
pkgrel=1
pkgdesc="Japanese web novel downloader and translator supporting Syosetu, Kakuyomu, and Pixiv"
arch=('x86_64' 'aarch64')
url="https://github.com/ripdog/tsundoku"
license=('GPL-3.0-or-later')
depends=('gcc-libs')
makedepends=('rust' 'cargo' 'cmake')
# Build from local working tree (no network fetch).
source=()
sha256sums=()

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cd "$startdir"
    cp -r src "$srcdir/"
    cp Cargo.toml "$srcdir/"
    cp Cargo.lock "$srcdir/"
    cp LICENSE "$srcdir/"
    cd "$srcdir"
    cargo fetch --target "$(rustc -vV | sed -n 's/host: //p')" --manifest-path=Cargo.toml
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    cd "$srcdir"
    export CARGO_TARGET_DIR=target
    # Clear LDFLAGS and CFLAGS that might interfere with aws-lc-sys
    unset LDFLAGS
    unset CFLAGS
    unset CXXFLAGS
    cargo build --release --all-features
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    cd "$srcdir"
    cargo test --all-features
}

package() {
    cd "$srcdir"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
