import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
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
    console.log("This is the VaultPdaAccount: ", vaultPdaAccount.toString());
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
  });
  it("stake amount", async () => {
    const [vaultPdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("pdaVault"), program.provider.publicKey.toBuffer()],
      program.programId
    );
    const [pdaAccount, bump2] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), program.provider.publicKey.toBuffer()],
      program.programId
    );
    console.log("This is the vault PDA account: ", vaultPdaAccount.toString());
    const amount = new BN(1_000_000_000);
    // Send the Transaction
    const txHash = await program.methods
      .stake(amount)
      .accounts({
        user: program.provider.publicKey,
        pdaAccount: pdaAccount,
        authority: program.provider.publicKey,
        pdaVaultAccount: vaultPdaAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
    // Confirm transaction
    await program.provider.connection.confirmTransaction(txHash);
    const userAccount = await program.account.stakeAccount.fetch(pdaAccount);
    console.log("This is the data from user's PDA: ", {
      user_staked_amount: userAccount.stakedAmount.toString(),
    });
    // Assertions
    assert.equal(userAccount.stakedAmount.toNumber(), 4000000000);
  });
  it("Unstake amount", async () => {
    const [vaultPdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("pdaVault"), program.provider.publicKey.toBuffer()],
      program.programId
    );
    const [pdaAccount, bump2] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), program.provider.publicKey.toBuffer()],
      program.programId
    );
    const userAccount = await program.account.stakeAccount.fetch(pdaAccount);
    console.log(
      "User staked amount before unstaking: ",
      userAccount.stakedAmount.toString()
    );
    console.log("This is the vaultPdaAccount: ", vaultPdaAccount.toString());
    const amount = new BN(1_000_000_000);
    // Send the transaction
    const txHash = await program.methods
      .unstake(amount)
      .accounts({
        user: program.provider.publicKey,
        pdaAccount: pdaAccount,
        authority: program.provider.publicKey,
        pdaVaultAccount: vaultPdaAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
    // Confirm the transaction
    await program.provider.connection.confirmTransaction(txHash);
    const userAccountUpdate = await program.account.stakeAccount.fetch(
      pdaAccount
    );
    console.log("User staked amount after unstaking: ", {
      user_staked_amount: userAccountUpdate.stakedAmount.toString(),
    });
    // Assertions
    assert.equal(userAccountUpdate.stakedAmount.toNumber(), 3000000000);
  });
  it("should get the reward points", async () => {
    const [pdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), program.provider.publicKey.toBuffer()],
      program.programId
    );
    const userAccount = await program.account.stakeAccount.fetch(pdaAccount);
    console.log(
      "User Staked Amount: ",
      userAccount.stakedAmount.toNumber() / 1000000000,
      "Sol"
    );
    console.log(
      "These are the reward points of user: ",
      userAccount.totalPoints.toNumber()
    );
    console.log("This is the current time: ", Math.floor(Date.now() / 1000));
    const current_time = Math.floor(Date.now() / 1000);
    const user_staked_sol_amount =
      userAccount.stakedAmount.toNumber() / 1000000000;
    const last_update_time = userAccount.lastUpdateTime.toNumber();
    const time_elapsed = current_time - last_update_time;
    const RewardPoints = user_staked_sol_amount * time_elapsed;

    if (userAccount.totalPoints.toNumber() == 0) {
      console.log("The updated reward point of the user is: ", RewardPoints);
    } else {
      const newRewardPoints = RewardPoints + userAccount.totalPoints.toNumber();
      console.log("The updated reward point of the user is: ", newRewardPoints);
    }
  });
  it("should reset the points", async () => {
    const [pdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), program.provider.publicKey.toBuffer()],
      program.programId
    );
    // Send Transction
    const txHash = await program.methods
      .claimPoints()
      .accounts({
        payer: program.provider.publicKey,
        pdaAccount: pdaAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
    // Confirm Transaction
    await program.provider.connection.confirmTransaction(txHash);
    const account = await program.account.stakeAccount.fetch(pdaAccount);
    console.log(
      "These are the reward points of user after claiming points: ",
      account.totalPoints.toNumber()
    );
    // Assertions
    assert.equal(account.totalPoints.toNumber(), 0);
  });
});
