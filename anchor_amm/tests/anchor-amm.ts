import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorAmm } from "../target/types/anchor_amm";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";

describe("anchor-amm", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.anchorAmm as Program<AnchorAmm>;
  const payer = provider.wallet.payer;
  const owner = provider.wallet.publicKey;

  const feeBps = 30;
  const mintDecimals = 6;

  const toBn = (value: bigint | number) => new anchor.BN(value.toString());
  const expectTxError = async (
    promise: Promise<string>,
    messagePart: string
  ) => {
    try {
      await promise;
      expect.fail("Expected transaction to fail");
    } catch (error: any) {
      const message = String(error?.message ?? error);
      expect(message.toLowerCase()).to.include(messagePart.toLowerCase());
    }
  };

  const setupPool = async () => {
    const seed = new anchor.BN(Date.now());

    const mintX = await createMint(
      provider.connection,
      payer,
      owner,
      null,
      mintDecimals
    );
    const mintY = await createMint(
      provider.connection,
      payer,
      owner,
      null,
      mintDecimals
    );

    const [config] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [mintLp] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("lp"), config.toBuffer()],
      program.programId
    );

    const vaultX = getAssociatedTokenAddressSync(mintX, config, true);
    const vaultY = getAssociatedTokenAddressSync(mintY, config, true);

    await program.methods
      .initialize(seed, feeBps, null)
      .accountsStrict({
        initializer: owner,
        mintX,
        mintY,
        mintLp,
        vaultX,
        vaultY,
        config,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const userX = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mintX,
        owner
      )
    ).address;
    const userY = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mintY,
        owner
      )
    ).address;
    const userLp = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mintLp,
        owner,
        true
      )
    ).address;

    const initialX = 5_000_000n;
    const initialY = 8_000_000n;
    const lpAmount = 1_000_000n;

    await mintTo(provider.connection, payer, mintX, userX, owner, initialX);
    await mintTo(provider.connection, payer, mintY, userY, owner, initialY);

    await program.methods
      .deposit(toBn(lpAmount), toBn(initialX), toBn(initialY))
      .accountsStrict({
        user: owner,
        mintX,
        mintY,
        config,
        mintLp,
        vaultX,
        vaultY,
        userX,
        userY,
        userLp,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .rpc();

    return {
      seed,
      mintX,
      mintY,
      mintLp,
      config,
      vaultX,
      vaultY,
      userX,
      userY,
      userLp,
      lpAmount,
    };
  };

  it("swaps x for y", async () => {
    const { mintX, mintY, config, vaultX, vaultY, userX, userY } =
      await setupPool();

    const amountIn = 1_000_000n;
    await mintTo(provider.connection, payer, mintX, userX, owner, amountIn);

    const beforeVaultX = (await getAccount(provider.connection, vaultX)).amount;
    const beforeVaultY = (await getAccount(provider.connection, vaultY)).amount;
    const beforeUserX = (await getAccount(provider.connection, userX)).amount;
    const beforeUserY = (await getAccount(provider.connection, userY)).amount;

    const feeAmount = (amountIn * BigInt(feeBps)) / 10_000n;
    const swapIn = amountIn - feeAmount;
    const k = beforeVaultX * beforeVaultY;
    const newX = beforeVaultX + swapIn;
    const newY = k / newX;
    const expectedOut = beforeVaultY - newY;

    await program.methods
      .swap(true, toBn(amountIn), toBn(0n))
      .accountsStrict({
        swapper: owner,
        mintX,
        mintY,
        config,
        vaultX,
        vaultY,
        userX,
        userY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const afterUserX = (await getAccount(provider.connection, userX)).amount;
    const afterUserY = (await getAccount(provider.connection, userY)).amount;

    expect(afterUserX).to.equal(beforeUserX - amountIn);
    expect(afterUserY).to.equal(beforeUserY + expectedOut);
  });

  it("withdraws liquidity", async () => {
    const { mintX, mintY, mintLp, config, vaultX, vaultY, userX, userY, userLp } =
      await setupPool();

    const beforeVaultX = (await getAccount(provider.connection, vaultX)).amount;
    const beforeVaultY = (await getAccount(provider.connection, vaultY)).amount;
    const beforeUserX = (await getAccount(provider.connection, userX)).amount;
    const beforeUserY = (await getAccount(provider.connection, userY)).amount;
    const beforeUserLp = (await getAccount(provider.connection, userLp)).amount;
    const lpSupply = (await getMint(provider.connection, mintLp)).supply;

    const burnAmount = 200_000n;
    const expectedX = (burnAmount * beforeVaultX) / lpSupply;
    const expectedY = (burnAmount * beforeVaultY) / lpSupply;

    await program.methods
      .withdraw(toBn(burnAmount), toBn(0n), toBn(0n))
      .accountsStrict({
        withdrawer: owner,
        mintX,
        mintY,
        config,
        vaultX,
        vaultY,
        mintLp,
        userLp,
        userX,
        userY,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const afterUserX = (await getAccount(provider.connection, userX)).amount;
    const afterUserY = (await getAccount(provider.connection, userY)).amount;
    const afterUserLp = (await getAccount(provider.connection, userLp)).amount;

    expect(afterUserX).to.equal(beforeUserX + expectedX);
    expect(afterUserY).to.equal(beforeUserY + expectedY);
    expect(afterUserLp).to.equal(beforeUserLp - burnAmount);
  });

  it("fails swap when min out is too high", async () => {
    const { mintX, mintY, config, vaultX, vaultY, userX, userY } =
      await setupPool();

    const amountIn = 500_000n;
    await mintTo(provider.connection, payer, mintX, userX, owner, amountIn);

    await expectTxError(
      program.methods
        .swap(true, toBn(amountIn), toBn(9_999_999_999n))
        .accountsStrict({
          swapper: owner,
          mintX,
          mintY,
          config,
          vaultX,
          vaultY,
          userX,
          userY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc(),
      "slippage"
    );
  });

  it("fails swap on zero amount", async () => {
    const { mintX, mintY, config, vaultX, vaultY, userX, userY } =
      await setupPool();

    await expectTxError(
      program.methods
        .swap(true, toBn(0n), toBn(0n))
        .accountsStrict({
          swapper: owner,
          mintX,
          mintY,
          config,
          vaultX,
          vaultY,
          userX,
          userY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc(),
      "invalid"
    );
  });

  it("fails withdraw when amount exceeds balance", async () => {
    const { mintX, mintY, mintLp, config, vaultX, vaultY, userX, userY, userLp } =
      await setupPool();

    const userLpBalance = (await getAccount(provider.connection, userLp)).amount;
    const tooMuch = userLpBalance + 1n;

    await expectTxError(
      program.methods
        .withdraw(toBn(tooMuch), toBn(0n), toBn(0n))
        .accountsStrict({
          withdrawer: owner,
          mintX,
          mintY,
          config,
          vaultX,
          vaultY,
          mintLp,
          userLp,
          userX,
          userY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc(),
      "insufficient"
    );
  });

  it("fails withdraw on zero amount", async () => {
    const { mintX, mintY, mintLp, config, vaultX, vaultY, userX, userY, userLp } =
      await setupPool();

    await expectTxError(
      program.methods
        .withdraw(toBn(0n), toBn(0n), toBn(0n))
        .accountsStrict({
          withdrawer: owner,
          mintX,
          mintY,
          config,
          vaultX,
          vaultY,
          mintLp,
          userLp,
          userX,
          userY,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc(),
      "zero"
    );
  });
});
