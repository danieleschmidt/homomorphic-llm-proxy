#!/bin/bash
# Fix deprecated rand functions and unused variables

# Fix deprecated rand::thread_rng() calls
sed -i 's/rand::thread_rng()/rand::rng()/g' src/fhe.rs

# Fix deprecated gen() method calls  
sed -i 's/rng\.gen()/rng.random()/g' src/fhe.rs

# Fix unused variables by prefixing with underscore
sed -i 's/let timer =/let _timer =/g' src/proxy.rs
sed -i 's/let provider =/let _provider =/g' src/proxy.rs
sed -i 's/let ciphertext =/let _ciphertext =/g' src/proxy.rs src/scaling.rs
sed -i 's/let now =/let _now =/g' src/fhe.rs
sed -i 's/let mut removed_count =/let removed_count =/g' src/fhe.rs

# Remove unused import
sed -i '/use tokio::time::sleep;/d' src/middleware.rs

echo "Fixed Rust warnings"