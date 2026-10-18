# TypeScript Bindings Generation

This document describes how to generate TypeScript bindings for the Stellar Tickets Ticketing smart contract using the `stellar contract bindings typescript` command.

## Prerequisites

- Rust toolchain with `wasm32v1-none` target
- `stellar-cli` installed (`cargo install --locked stellar-cli --version ^27`)
- Contract WASM built (`make build` in `contracts/ticketing/`)

## Quick Start

```bash
# From project root
./scripts/bindings/generate-typescript-bindings.sh
```

This will:
1. Verify the contract WASM exists
2. Generate TypeScript bindings in `bindings/typescript/`
3. Output a ready-to-use npm package

## Output Structure

```
bindings/typescript/
├── package.json          # npm package configuration
├── tsconfig.json         # TypeScript configuration
├── src/
│   ├── index.ts          # Main exports
│   ├── client.ts         # Contract client class
│   ├── types.ts          # Type definitions
│   └── ...
└── README.md             # Usage documentation
```

## Using the Generated Bindings

### Installation

Copy the generated `bindings/typescript` directory to your backend project and install dependencies:

```bash
# In your backend project
cp -r /path/to/blockchain/bindings/typescript ./contract-bindings
cd contract-bindings
npm install
```

### Usage Example

```typescript
import { Client, Networks } from './contract-bindings';

// Initialize client for testnet
const client = new Client({
  network: Networks.TESTNET,
  contractId: 'CONTRACT_ID_HERE',
  rpcUrl: 'https://soroban-testnet.stellar.org',
});

// Call contract methods
const result = await client.someMethod({ param: 'value' });
console.log(result);
```

### With Stellar Wallet (Freighter, Albedo, etc.)

```typescript
import { Client, Networks } from './contract-bindings';
import { Wallet } from '@stellar/stellar-wallets-kit';

const wallet = new Wallet();
// ... connect wallet ...

const client = new Client({
  network: Networks.TESTNET,
  contractId: 'CONTRACT_ID_HERE',
  rpcUrl: 'https://soroban-testnet.stellar.org',
  wallet: wallet, // Pass wallet for signing
});
```

## Regenerating Bindings

Run the generation script again after contract changes:

```bash
# Rebuild contract first
cd contracts/ticketing && make build

# Regenerate bindings
cd ../..
./scripts/bindings/generate-typescript-bindings.sh
```

## CI/CD Integration

Add to your backend CI pipeline:

```yaml
- name: Generate contract bindings
  run: |
    cd blockchain
    ./scripts/bindings/generate-typescript-bindings.sh
    cp -r bindings/typescript ../backend/contract-bindings
```

## Troubleshooting

### WASM not found
Ensure you've built the contract first:
```bash
cd contracts/ticketing && make build
```

### stellar CLI version mismatch
Use stellar-cli v27+:
```bash
cargo install --locked stellar-cli --version ^27
```

### TypeScript compilation errors
Ensure you're using TypeScript 5.0+ and have the required dependencies:
```json
{
  "dependencies": {
    "@stellar/stellar-sdk": "^12.0.0",
    "typescript": "^5.0.0"
  }
}
```