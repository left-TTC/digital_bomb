
# SOL GAME

## build
``` shell  
cargo build-sbf
```
 Solana Cil: 2.2.17

## Check
``` bash
solana-keygen pubkey target/deploy/digital_sol_game-keypair.json
```

## Deploy
```bash
solana program deploy --program-id target/deploy/web3_domain_registrar-keypair.json target/sbf-solana-solana/release/web3_domain_registrar.so  --use-rpc
```