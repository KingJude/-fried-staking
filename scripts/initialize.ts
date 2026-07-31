import "dotenv/config";
import * as anchor from "@coral-xyz/anchor";
import { getMint } from "@solana/spl-token";

const FRIED_MINT = new anchor.web3.PublicKey(
  process.env.FRIED_MINT ?? "3LaXmnVQxMMArywUBJjGA5EKbUdJ17PFor3M5oeupump"
);

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.FriedStaking as anchor.Program;
  const mint = await getMint(provider.connection, FRIED_MINT);
  const minimumStake = new anchor.BN(100_000).mul(new anchor.BN(10).pow(new anchor.BN(mint.decimals)));
  const [config] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), FRIED_MINT.toBuffer()],
    program.programId
  );
  const [stakeVault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("stake-vault"), config.toBuffer()],
    program.programId
  );
  const [rewardVault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("reward-vault"), config.toBuffer()],
    program.programId
  );

  const signature = await program.methods
    .initialize(minimumStake)
    .accounts({ admin: provider.wallet.publicKey, mint: FRIED_MINT, config, stakeVault, rewardVault })
    .rpc();
  console.log({ signature, config: config.toBase58(), stakeVault: stakeVault.toBase58(), rewardVault: rewardVault.toBase58() });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
