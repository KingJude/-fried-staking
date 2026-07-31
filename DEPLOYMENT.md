# Deployment checklist

## 1. Test locally

```bash
yarn install
anchor build
anchor test
yarn app:build
```

## 2. Deploy to devnet

1. Create a dedicated **deployment keypair**. Do not use or share a seed phrase in chat.
2. Set the Solana CLI to devnet and fund the deployer with devnet SOL.
3. Run `anchor keys sync`, then commit the program-ID changes.
4. Run `anchor deploy`.
5. Set `.env` values to the new program ID and devnet RPC.
6. Run `yarn initialize`.
7. Test funding, staking, lock expiry, unstaking, reward calculations, pausing, and insufficient-reward behavior.

## 3. Security gate

Before mainnet:

- commission an independent Solana/Anchor audit;
- add full integration tests using a test FRIED mint;
- verify mint decimals and the exact minimum/reward raw-unit conversions;
- use a dedicated admin wallet or multisig;
- verify the deployed binary matches the audited commit;
- publish the program ID and audit report;
- run a small, capped mainnet pilot before transferring all rewards.

## 4. Mainnet initialization

1. Change provider/RPC settings to mainnet.
2. Deploy the audited program and update `PROGRAM_ID` / `VITE_PROGRAM_ID`.
3. Run `yarn initialize` once from the authorized admin wallet.
4. Verify the config, vault addresses, program end time, and minimum stake on-chain.
5. Run `yarn fund-rewards` to transfer exactly `20,000,000 FRIED`.
6. Confirm the reward-vault balance on a block explorer.
7. Build and publish the web app only after all addresses are independently verified.

The GitHub upload itself does not move tokens or authorize wallet transactions.
