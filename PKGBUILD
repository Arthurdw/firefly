# Maintainer: Arthurdw <contact@arthurdw.com>
_name=ffly
pkgname=${_name}
pkgver=0.0.0
pkgrel=1
pkgdesc="An \"blazingly\" fast key-value pair database without bloat written in rust"
arch=(x86_64 i686)
url="https://github.com/Arthurdw/firefly"
license=('MIT')
makedepends=('cargo')
source=("git+https://github.com/Arthurdw/firefly.git")
sha256sums=('SKIP')

prepare() {
    cd "firefly/server"
    
    cargo fetch --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "firefly/server"

    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}


package() {
    cd "firefly/server"

    mv "target/release/firefly" "target/release/$pkgname"
    install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"

}

check() {
    cd "firefly/server"

    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features
}


