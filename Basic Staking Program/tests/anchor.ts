import * as anchor from "@coral-xyz/anchor";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import type { BasicStakingProgram } from "../target/types/basic_staking_program";

describe("Test", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.BasicStakingProgram as anchor.Program<BasicStakingProgram>;
  
  it("create a user pda account", async () => {
    const [pdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), program.provider.publicKey.toBuffer()],
      program.programId
    );

    // Send Transaction
    const txHash = await program.methods
      .createPdaAccount()
      .accounts({
        payer: program.provider.publicKey,
        pdaAccount: pdaAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm transaction
    await program.provider.connection.confirmTransaction(txHash);

    const account = await program.account.stakeAccount.fetch(pdaAccount);

    console.log("On-chain stake account data: ", {
      staked_amount: account.stakedAmount.toString(),
    });

    // Assertions
    assert.equal(account.stakedAmount.toNumber(), 0);
  });

  it("creates a vault pda account", async () => {
    const [vaultPdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("pdaVault"), program.provider.publicKey.toBuffer()],
      program.programId
    );

    // Send Transaction
    const txHash = await program.methods
      .createVaultPdaAccount()
      .accounts({
        authority: program.provider.publicKey,
        pdaVaultAccount: vaultPdaAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm transaction
    await program.provider.connection.confirmTransaction(txHash);

    const account = await program.account.vaultAccount.fetch(
      vaultPdaAccount
    );

    console.log("On-chain vault stake account data: ", {
      total_staked_amount: account.totalStakedAmount.toString(),
    });

    // Assertions
    assert.equal(account.totalStakedAmount.toNumber(), 0);
  });
});
