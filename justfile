default: check test

check:
    cargo check
    cargo clippy -- -D warnings

test:
    cargo test

qa: fmt check test
    @echo "All checks passed."

run *ARGS:
    sudo cargo run -- {{ARGS}}

fmt:
    cargo fmt --all

audit:
    cargo audit

install prefix="/usr/local":
    cargo build --release
    install -m 755 target/release/veln {{prefix}}/bin/veln
    @echo "Installed veln to {{prefix}}/bin/veln"

uninstall prefix="/usr/local":
    rm -f {{prefix}}/bin/veln
    @echo "Uninstalled veln from {{prefix}}/bin/veln"

port-test:
    @echo "Testing port from local source..."
    make -C port/sysutils/veln LOCAL_SOURCE=on WRKSRC=$(PWD) stage
