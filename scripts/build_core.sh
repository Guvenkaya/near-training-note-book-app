cargo build -p core-contract --target wasm32-unknown-unknown --release && \
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
