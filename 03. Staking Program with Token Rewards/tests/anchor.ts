import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import { StakeWithTokenReward } from '../target/types/stake_with_token_reward';
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

const base58PrivateKey = "Private_Key";
const privateKeySeed = bs58.decode(base58PrivateKey);

const userKeypair = web3.Keypair.fromSecretKey(privateKeySeed);

const connection = new web3.Connection("https://api.devnet.solana.com", "confirmed");
const userWallet = new anchor.Wallet(userKeypair);
const provider = new anchor.AnchorProvider(connection, userWallet, {
  preflightCommitment: "confirmed",
});
anchor.setProvider(provider);

describe("Test", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.StakeWithTokenReward as anchor.Program<StakeWithTokenReward>;

  const METADATA_SEED = "metadata";
  const TOKEN_METADATA_PROGRAM_ID = new web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

  // Mint PDA
  const [mint] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint")],
    program.programId
  );

  const [authority] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("authority")],
    program.programId
  );

  // Metadata PDA
  const [metadataAddress] = web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(METADATA_SEED),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );

  const userPublicKey = new web3.PublicKey("HVw1Z2KFYfKjdL2UThi5RGBvSUpsF4zdsPrucV8TggQm");

  it("create a user pda account", async () => {
    const [pdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), userPublicKey.toBuffer()],
      program.programId
    );

    // Send Transaction
    const txHash = await program.methods
      .createPdaAccount()
      .accounts({
        payer: userPublicKey,
        pdaAccount: pdaAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([userKeypair])
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
        authority: userPublicKey,
        pdaVaultAccount: vaultPdaAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm transaction
    await program.provider.connection.confirmTransaction(txHash);
  });

  it("creates a staking token mint", async () => {
    const metadata = {
      name: "Staking Token",
      symbol: "STAKE",
      uri: "https://jsonkeeper.com/b/THX2",
      decimals: 9,
    };

    const txHash = await program.methods
      .createTokenMint(metadata)
      .accounts({
        metadata: metadataAddress,
        mint: mint,
        authority: authority,
        payer: program.provider.publicKey,
        rent: web3.SYSVAR_RENT_PUBKEY,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID
      })
      .rpc();

      console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);
  })


    it("stake amount", async () => {
    const [vaultPdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("pdaVault"), userPublicKey.toBuffer()],
      program.programId
    );

    const [pdaAccount, bump2] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), userPublicKey.toBuffer()],
      program.programId
    );

    const destination = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: new web3.PublicKey("HVw1Z2KFYfKjdL2UThi5RGBvSUpsF4zdsPrucV8TggQm")
    });

    const amount = new BN(2_000_000_000);

    // Send the Transaction
    const txHash = await program.methods
      .stake(amount)
      .accounts({
        user: userPublicKey,
        pdaAccount: pdaAccount,
        authorityVault: userPublicKey,
        pdaVaultAccount: vaultPdaAccount,
        mint,
        authority,
        destination,
        destinationOwner: userPublicKey,
        payer: userPublicKey,
        rent: web3.SYSVAR_RENT_PUBKEY,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: new web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      })
      .signers([userKeypair])
      .rpc();
    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm transaction
    await program.provider.connection.confirmTransaction(txHash);

    const userAccount = await program.account.stakeAccount.fetch(pdaAccount);

    console.log("This is the data from user's PDA: ", {
      user_staked_amount: userAccount.stakedAmount.toString(),
    });
    
    // Assertions
    assert.equal(userAccount.stakedAmount.toNumber(), 2000000000);
  });

  it("Unstake amount", async () => {
    const [vaultPdaAccount, bump] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("pdaVault"), userPublicKey.toBuffer()],
      program.programId
    );
    
    const [pdaAccount, bump2] = await web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), userPublicKey.toBuffer()],
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
        user: userPublicKey,
        pdaAccount: pdaAccount,
        authority: userPublicKey,
        pdaVaultAccount: vaultPdaAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([userKeypair])
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
    assert.equal(userAccountUpdate.stakedAmount.toNumber(), 1000000000);
  });
});