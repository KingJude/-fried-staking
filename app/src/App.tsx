import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

const pools = [
  { name: "Flexible", lock: "No lock", apy: "30%", accent: "gold" },
  { name: "7 Day", lock: "7-day lock", apy: "40%", accent: "orange" },
  { name: "30 Day", lock: "30-day lock", apy: "60%", accent: "red" },
];

export default function App() {
  const configured = Boolean(import.meta.env.VITE_PROGRAM_ID) &&
    import.meta.env.VITE_PROGRAM_ID !== "REPLACE_AFTER_ANCHOR_DEPLOY";

  return (
    <main>
      <nav>
        <div className="brand"><span>🍗</span> FRIED STAKING</div>
        <WalletMultiButton />
      </nav>
      <section className="hero">
        <p className="eyebrow">FRIED CHICKEN ON SOLANA</p>
        <h1>Stake it.<br /><em>Keep it crispy.</em></h1>
        <p className="lead">Earn FRIED rewards with flexible, 7-day, or 30-day staking.</p>
        <div className="stats">
          <div><strong>20M</strong><span>Reward allocation</span></div>
          <div><strong>100K</strong><span>Minimum FRIED</span></div>
          <div><strong>1 year</strong><span>Program duration</span></div>
        </div>
      </section>
      <section className="pools">
        {pools.map((pool) => (
          <article key={pool.name} className={pool.accent}>
            <p>{pool.name} Pool</p>
            <h2>{pool.apy} <small>APY</small></h2>
            <span>{pool.lock}</span>
            <label>
              Amount
              <input type="number" min="100000" placeholder="Minimum 100,000 FRIED" disabled={!configured} />
            </label>
            <button disabled={!configured}>{configured ? "Stake FRIED" : "Available after deployment"}</button>
          </article>
        ))}
      </section>
      {!configured && (
        <aside>
          The interface is in safe setup mode. Deploy the audited program and set
          <code> VITE_PROGRAM_ID </code> to enable transactions.
        </aside>
      )}
      <footer>
        <span>Mint: 3LaX…pump</span>
        <span>Never share your seed phrase.</span>
      </footer>
    </main>
  );
}
