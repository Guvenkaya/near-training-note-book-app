cargo build -p core-contract -p fungible-token --target wasm32-unknown-unknown --release && \
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
