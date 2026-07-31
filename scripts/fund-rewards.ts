import "dotenv/config";
import * as anchor from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync, getMint } from "@solana/spl-token";

const FRIED_MINT = new anchor.web3.PublicKey(
  process.env.FRIED_MINT ?? "3LaXmnVQxMMArywUBJjGA5EKbUdJ17PFor3M5oeupump"
);

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.FriedStaking as anchor.Program;
  const mint = await getMint(provider.connection, FRIED_MINT);
  const amount = new anchor.BN(20_000_000).mul(new anchor.BN(10).pow(new anchor.BN(mint.decimals)));
  const [config] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), FRIED_MINT.toBuffer()],
    program.programId
  );
  const configAccount: any = await program.account.config.fetch(config);
  const adminToken = getAssociatedTokenAddressSync(FRIED_MINT, provider.wallet.publicKey);
  const signature = await program.methods
    .fundRewards(amount)
    .accounts({
      config,
      admin: provider.wallet.publicKey,
      mint: FRIED_MINT,
      adminToken,
      rewardVault: configAccount.rewardVault,
    })
    .rpc();
  console.log({ signature, funded: "20,000,000 FRIED" });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
