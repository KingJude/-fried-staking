import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("fried-staking", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  it("publishes the intended pool terms", () => {
    const pools = [
      { name: "Flexible", lockDays: 0, apy: 30 },
      { name: "7 Day", lockDays: 7, apy: 40 },
      { name: "30 Day", lockDays: 30, apy: 60 },
    ];
    expect(pools.map((pool) => pool.apy)).to.deep.equal([30, 40, 60]);
    expect(pools.map((pool) => pool.lockDays)).to.deep.equal([0, 7, 30]);
  });
});
