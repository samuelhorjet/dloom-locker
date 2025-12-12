import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DloomLocker } from "../target/types/dloom_locker";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  getAccount,
  createInitializeTransferFeeConfigInstruction,
  ExtensionType,
  getMintLen,
  createInitializeMintInstruction,
  withdrawWithheldTokensFromAccounts,
} from "@solana/spl-token";
import { assert } from "chai";

describe("dloom_locker", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DloomLocker as Program<DloomLocker>;
  const wallet = provider.wallet as anchor.Wallet;

  // Test Variables
  let standardMint: anchor.web3.PublicKey;
  let token22Mint: anchor.web3.PublicKey;
  let feeMint: anchor.web3.PublicKey;

  let userStandardAccount: anchor.web3.PublicKey;
  let userToken22Account: anchor.web3.PublicKey;
  let userFeeAccount: anchor.web3.PublicKey;

  const LOCK_AMOUNT = new anchor.BN(1000);
  const BURN_AMOUNT = new anchor.BN(500);

  // Helper to reduce repetitive PDA code
  const findPDAs = (mint: anchor.web3.PublicKey, lockId: anchor.BN) => {
    const [lockRecordPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("lock_record"),
        wallet.publicKey.toBuffer(),
        mint.toBuffer(),
        lockId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), lockRecordPda.toBuffer()],
      program.programId
    );

    return { lockRecordPda, vaultPda };
  };

  it("Setup: Create Mints and Token Accounts", async () => {
    // 1. Create Standard SPL Mint
    standardMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );

    const userStandardTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      standardMint,
      wallet.publicKey,
      false,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );
    userStandardAccount = userStandardTokenAccount.address;

    await mintTo(
      provider.connection,
      wallet.payer,
      standardMint,
      userStandardAccount,
      wallet.payer,
      10000,
      [],
      undefined,
      TOKEN_PROGRAM_ID
    );

    // 2. Create Basic Token-2022 Mint
    token22Mint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    const userToken22TokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      token22Mint,
      wallet.publicKey,
      false,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    userToken22Account = userToken22TokenAccount.address;

    await mintTo(
      provider.connection,
      wallet.payer,
      token22Mint,
      userToken22Account,
      wallet.payer,
      10000,
      [],
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    console.log("Setup complete: Standard and Token22 mints created.");
  });

  it("Standard Token: Lock, Burn Wallet, and Withdraw", async () => {
    const lockId = new anchor.BN(1);

    // FIX: Increased buffer to 10 seconds to avoid UnlockDateInPast error due to clock drift
    const unlockTime = new anchor.BN(Math.floor(Date.now() / 1000) + 10);

    const { lockRecordPda, vaultPda } = findPDAs(standardMint, lockId);

    // 1. LOCK
    await program.methods
      .handleLockTokens(LOCK_AMOUNT, unlockTime, lockId)
      .accountsPartial({
        owner: wallet.publicKey,
        tokenMint: standardMint,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        userTokenAccount: userStandardAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    let vaultAccount = await getAccount(provider.connection, vaultPda);
    assert.equal(
      Number(vaultAccount.amount),
      1000,
      "Vault should hold 1000 tokens"
    );

    // 2. BURN FROM WALLET
    await program.methods
      .handleBurnFromWallet(BURN_AMOUNT)
      .accounts({
        burner: wallet.publicKey,
        tokenMint: standardMint,
        userTokenAccount: userStandardAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // 3. WAIT & WITHDRAW
    console.log("Standard Token: Waiting 12s for lock to expire...");
    // FIX: Increased wait time to 12000ms to exceed the 10s lock time
    await new Promise((r) => setTimeout(r, 12000));

    await program.methods
      .handleWithdrawTokens(lockId, LOCK_AMOUNT)
      .accountsPartial({
        owner: wallet.publicKey,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        userTokenAccount: userStandardAccount,
        tokenMint: standardMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Final Check: User started with 10000 -> Locked 1000 -> Burned 500 from wallet -> Withdrew 1000
    // Result should be 9500
    let userAccount = await getAccount(
      provider.connection,
      userStandardAccount
    );
    assert.equal(Number(userAccount.amount), 9500, "Final balance incorrect");

    const info = await provider.connection.getAccountInfo(vaultPda);
    assert.isNull(info, "Vault account should be closed");
    console.log("Standard Token Cycle Passed!");
  });

  // --- NEW TEST CASE ---
  it("Burn Locked Tokens: Lock -> Burn Partial -> Verify State", async () => {
    const lockId = new anchor.BN(99);
    const unlockTime = new anchor.BN(Math.floor(Date.now() / 1000) + 1000); // Long lock is fine
    const { lockRecordPda, vaultPda } = findPDAs(standardMint, lockId);
    const lockAmount = new anchor.BN(2000);
    const burnLockedAmount = new anchor.BN(500);

    // 1. Lock
    await program.methods
      .handleLockTokens(lockAmount, unlockTime, lockId)
      .accountsPartial({
        owner: wallet.publicKey,
        tokenMint: standardMint,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        userTokenAccount: userStandardAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // 2. Burn From Lock
    await program.methods
      .handleBurnFromLock(burnLockedAmount, lockId)
      .accountsPartial({
        owner: wallet.publicKey,
        tokenMint: standardMint,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // 3. Verify State
    const lockRecord = await program.account.lockRecord.fetch(lockRecordPda);
    assert.equal(
      lockRecord.amount.toNumber(),
      1500,
      "LockRecord amount should be reduced to 1500"
    );

    const vaultAccount = await getAccount(provider.connection, vaultPda);
    assert.equal(
      Number(vaultAccount.amount),
      1500,
      "Vault balance should be reduced to 1500"
    );

    console.log("Burn Locked Tokens Cycle Passed!");
  });

  it("Token-2022: Lock, Burn Wallet, and Withdraw", async () => {
    const lockId = new anchor.BN(2);

    // FIX: Increased buffer to 10 seconds
    const unlockTime = new anchor.BN(Math.floor(Date.now() / 1000) + 10);

    const { lockRecordPda, vaultPda } = findPDAs(token22Mint, lockId);

    // 1. LOCK
    await program.methods
      .handleLockTokens(LOCK_AMOUNT, unlockTime, lockId)
      .accountsPartial({
        owner: wallet.publicKey,
        tokenMint: token22Mint,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        userTokenAccount: userToken22Account,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    // 2. BURN FROM WALLET
    await program.methods
      .handleBurnFromWallet(BURN_AMOUNT)
      .accounts({
        burner: wallet.publicKey,
        tokenMint: token22Mint,
        userTokenAccount: userToken22Account,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    // 3. WAIT & WITHDRAW
    console.log("Token-2022: Waiting 12s for lock to expire...");
    // FIX: Increased wait to 12s
    await new Promise((r) => setTimeout(r, 12000));

    await program.methods
      .handleWithdrawTokens(lockId, LOCK_AMOUNT)
      .accountsPartial({
        owner: wallet.publicKey,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        userTokenAccount: userToken22Account,
        tokenMint: token22Mint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    const userAccount = await getAccount(
      provider.connection,
      userToken22Account,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    // Start 10000 -> Lock 1000 -> Burn 500 -> Withdraw 1000 = 9500
    assert.equal(
      Number(userAccount.amount),
      9500,
      "Final Token-2022 balance incorrect"
    );
    console.log("Token-2022 Cycle Passed!");
  });

  it("Token-2022: Transfer Fees (Lock -> Burn -> Withdraw -> Harvest -> Close)", async () => {
    const mintKeypair = anchor.web3.Keypair.generate();
    feeMint = mintKeypair.publicKey;
    const decimals = 9;
    const feeBasisPoints = 1000; // 10%
    const maxFee = new anchor.BN(5000);

    const extensions = [ExtensionType.TransferFeeConfig];
    const mintLen = getMintLen(extensions);
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(mintLen);

    const createAccountIx = anchor.web3.SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: feeMint,
      space: mintLen,
      lamports,
      programId: TOKEN_2022_PROGRAM_ID,
    });

    const initFeeIx = createInitializeTransferFeeConfigInstruction(
      feeMint,
      wallet.publicKey, // TransferFeeConfig Authority
      wallet.publicKey, // WithdrawWithheldAuthority
      feeBasisPoints,
      BigInt(maxFee.toString()),
      TOKEN_2022_PROGRAM_ID
    );

    const initMintIx = createInitializeMintInstruction(
      feeMint,
      decimals,
      wallet.publicKey,
      null,
      TOKEN_2022_PROGRAM_ID
    );

    const tx = new anchor.web3.Transaction().add(
      createAccountIx,
      initFeeIx,
      initMintIx
    );
    await provider.sendAndConfirm(tx, [mintKeypair]);

    const userFeeTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      feeMint,
      wallet.publicKey,
      false,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    userFeeAccount = userFeeTokenAccount.address;

    await mintTo(
      provider.connection,
      wallet.payer,
      feeMint,
      userFeeAccount,
      wallet.payer,
      5000,
      [],
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    // --- EXECUTE FULL CYCLE ---
    const lockId = new anchor.BN(3);

    // FIX: Increased buffer to 10 seconds
    const unlockTime = new anchor.BN(Math.floor(Date.now() / 1000) + 10);

    const { lockRecordPda, vaultPda } = findPDAs(feeMint, lockId);

    // 1. LOCK
    await program.methods
      .handleLockTokens(LOCK_AMOUNT, unlockTime, lockId)
      .accountsPartial({
        owner: wallet.publicKey,
        tokenMint: feeMint,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        userTokenAccount: userFeeAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    let lockRecordAccount = await program.account.lockRecord.fetch(
      lockRecordPda
    );
    assert.equal(
      lockRecordAccount.amount.toNumber(),
      900,
      "Should store 900 (after 10% fee)"
    );

    // 2. BURN FROM WALLET
    await program.methods
      .handleBurnFromWallet(BURN_AMOUNT)
      .accounts({
        burner: wallet.publicKey,
        tokenMint: feeMint,
        userTokenAccount: userFeeAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    // 3. WAIT & WITHDRAW
    console.log("Fee Token: Waiting 12s for lock to expire...");
    // FIX: Increased wait to 12s
    await new Promise((r) => setTimeout(r, 12000));

    // This will withdraw the 900 tokens but fail to close the account because fees are stuck
    await program.methods
      .handleWithdrawTokens(lockId, LOCK_AMOUNT)
      .accountsPartial({
        owner: wallet.publicKey,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        userTokenAccount: userFeeAccount,
        tokenMint: feeMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    let vaultInfo = await provider.connection.getAccountInfo(vaultPda);
    assert.isNotNull(vaultInfo, "Vault should still exist due to stuck fees");

    // 4. HARVEST
    console.log("Harvesting fees from Vault...");
    await withdrawWithheldTokensFromAccounts(
      provider.connection,
      wallet.payer, // Payer
      feeMint, // Mint
      userFeeAccount, // Destination
      wallet.payer, // Authority
      [],
      [vaultPda], // Harvest FROM the Vault
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    // 5. CLOSE VAULT
    console.log("Closing cleaned Vault...");
    await program.methods
      .handleCloseVault(lockId)
      .accountsPartial({
        owner: wallet.publicKey,
        lockRecord: lockRecordPda,
        vault: vaultPda,
        tokenMint: feeMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    vaultInfo = await provider.connection.getAccountInfo(vaultPda);
    assert.isNull(vaultInfo, "Vault should be fully closed now");

    console.log("Full Transfer Fee Cycle Passed!");
  });

  it("Multiple Locks: Same Token, Different IDs", async () => {
    const lockIdA = new anchor.BN(10);
    const lockIdB = new anchor.BN(11);
    const unlockTime = new anchor.BN(Math.floor(Date.now() / 1000) + 1000);
    const pdaA = findPDAs(standardMint, lockIdA);
    const pdaB = findPDAs(standardMint, lockIdB);

    // Lock A
    await program.methods
      .handleLockTokens(new anchor.BN(100), unlockTime, lockIdA)
      .accountsPartial({
        owner: wallet.publicKey,
        tokenMint: standardMint,
        lockRecord: pdaA.lockRecordPda,
        vault: pdaA.vaultPda,
        userTokenAccount: userStandardAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Lock B
    await program.methods
      .handleLockTokens(new anchor.BN(100), unlockTime, lockIdB)
      .accountsPartial({
        owner: wallet.publicKey,
        tokenMint: standardMint,
        lockRecord: pdaB.lockRecordPda,
        vault: pdaB.vaultPda,
        userTokenAccount: userStandardAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("Multiple locks created successfully.");
  });
});
