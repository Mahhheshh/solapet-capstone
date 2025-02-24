import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolapetCapstone } from "../target/types/solapet_capstone";
import { assert, expect } from "chai";

import {
  Ed25519Program,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Transaction,
} from "@solana/web3.js";

import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";

import {
  createNft,
  findMetadataPda,
  mplTokenMetadata,
  TokenStandard,
  transferV1,
  MPL_TOKEN_METADATA_PROGRAM_ID,
  findMasterEditionPda,
} from "@metaplex-foundation/mpl-token-metadata";

import {
  generateSigner,
  keypairIdentity,
  percentAmount,
  publicKey,
  PublicKey as UmiPublickkey,
} from "@metaplex-foundation/umi";
import {
  getAccount,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("solapet-capstone", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const program = anchor.workspace.SolapetCapstone as Program<SolapetCapstone>;

  // accounts
  let gameConfig: PublicKey;
  let gameVault: PublicKey;
  let admin: Keypair = Keypair.generate();
  let player1: Keypair = Keypair.generate();
  let player2: Keypair = Keypair.generate();
  let petStat1: PublicKey;
  let petStat2: PublicKey;
  let petDuel: PublicKey; // player 1 challanger
  let player1ATA: PublicKey;
  let player2ATA: PublicKey;
  let nftMintAddress1: PublicKey;
  let nftMintAddress2: PublicKey;
  let metadata1: UmiPublickkey;
  let metadata2: UmiPublickkey;
  let masterEdition1: UmiPublickkey;
  let masterEdition2: UmiPublickkey;

  // umi stuff
  const umi = createUmi(provider.connection.rpcEndpoint).use(
    mplTokenMetadata()
  );

  let umiAdmin = umi.eddsa.createKeypairFromSecretKey(admin.secretKey);
  umi.use(keypairIdentity(umiAdmin));

  const collectionMint = generateSigner(umi);

  before(async () => {
    await Promise.all([
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          admin.publicKey,
          2 * LAMPORTS_PER_SOL
        ),
        "finalized"
      ),
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          player1.publicKey,
          2 * LAMPORTS_PER_SOL
        ),
        "finalized"
      ),
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(
          player2.publicKey,
          2 * LAMPORTS_PER_SOL
        ),
        "finalized"
      ),
    ]);

    [gameVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    );

    [gameConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      program.programId
    );

    [petStat1] = PublicKey.findProgramAddressSync(
      [Buffer.from("stats"), player1.publicKey.toBuffer()],
      program.programId
    );

    [petStat2] = PublicKey.findProgramAddressSync(
      [Buffer.from("stats"), player2.publicKey.toBuffer()],
      program.programId
    );

    [petDuel] = PublicKey.findProgramAddressSync(
      [Buffer.from("pet_duel"), player1.publicKey.toBuffer()],
      program.programId
    );

    [nftMintAddress1] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_mint"),
        player1.publicKey.toBuffer(),
        new PublicKey(collectionMint.publicKey).toBuffer(),
      ],
      program.programId
    );

    [nftMintAddress2] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("nft_mint"),
        player2.publicKey.toBuffer(),
        new PublicKey(collectionMint.publicKey).toBuffer(),
      ],
      program.programId
    );

    player1ATA = await getAssociatedTokenAddress(
      nftMintAddress1,
      player1.publicKey
    );

    player2ATA = await getAssociatedTokenAddress(
      nftMintAddress2,
      player2.publicKey
    );

    [metadata1] = findMetadataPda(umi, {
      mint: publicKey(nftMintAddress1),
    });

    [metadata2] = findMetadataPda(umi, {
      mint: publicKey(nftMintAddress2),
    });

    [masterEdition1] = findMasterEditionPda(umi, {
      mint: publicKey(nftMintAddress1),
    });

    [masterEdition2] = findMasterEditionPda(umi, {
      mint: publicKey(nftMintAddress2),
    });

    console.log({
      admin: admin.publicKey.toString(),
      player1: player1.publicKey.toString(),
      player2: player2.publicKey.toString(),
      petDuel: petDuel.toString(),
      petStat1: petStat1.toString(),
      petStat2: petStat2.toString(),
      gameConfig: gameConfig.toString(),
      gameVault: gameVault.toString(),
      collectionMint: collectionMint.publicKey,
    });

    // create collection
    await createNft(umi, {
      mint: collectionMint,
      name: "SolaPet Collection",
      symbol: "SP",
      uri: "", // TODO: get uri from ariv
      sellerFeeBasisPoints: percentAmount(0),
      isCollection: true,
    }).sendAndConfirm(umi, {
      confirm: { commitment: "finalized" },
      send: { commitment: "finalized" },
    });
  });

  it("Is initilized game config!", async () => {
    await program.methods
      .initialize(1)
      .accounts({
        admin: admin.publicKey,
        collectionMint: collectionMint.publicKey,
      })
      .signers([admin])
      .rpc();

    const onChainConfig = await program.account.gameConfig.fetch(gameConfig);

    expect(onChainConfig.collectionMint.toString()).to.equal(
      collectionMint.publicKey.toString()
    );
    expect(onChainConfig.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    expect(onChainConfig.gameVault.toBase58()).to.equal(gameVault.toBase58());
    expect(onChainConfig.fees.toString()).to.equal("1");
  });

  it("Should fail to update the the fees for non admin key", async () => {
    try {
      await program.methods
        .updateFees(5)
        .accountsPartial({
          admin: player1.publicKey,
          gameConfig: gameConfig,
        })
        .signers([player1])
        .rpc();
    } catch (error) {
      assert.isOk(error.message, "UnauthorizedAction.");
    }
  });

  it("Should update the the fees for Admin key", async () => {
    await program.methods
      .updateFees(5)
      .accountsPartial({
        admin: admin.publicKey,
        gameConfig: gameConfig,
      })
      .signers([admin])
      .rpc();

    const updatedConfig = await program.account.gameConfig.fetch(gameConfig);
    expect(updatedConfig.fees.toString()).to.equal("5");
    expect(updatedConfig.admin.toBase58()).to.equal(admin.publicKey.toBase58());
  });

  it("should mint a new nft pet for players", async () => {
    await Promise.all([
      program.methods
        .mintPet("") // Empty URI string
        .accountsPartial({
          player: player1.publicKey,
          collectionMint: collectionMint.publicKey,
          gameConfig: gameConfig,
          playerTokenAccount: player1ATA,
          nftMint: nftMintAddress1,
          metadata: metadata1,
          masterEdition: masterEdition1,
          metadataProgramInfo: MPL_TOKEN_METADATA_PROGRAM_ID,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([player1])
        .rpc(),

      program.methods
        .mintPet("") // Empty URI string
        .accountsPartial({
          player: player2.publicKey,
          collectionMint: collectionMint.publicKey,
          gameConfig: gameConfig,
          playerTokenAccount: player2ATA,
          nftMint: nftMintAddress2,
          metadata: metadata2,
          masterEdition: masterEdition2,
          metadataProgramInfo: MPL_TOKEN_METADATA_PROGRAM_ID,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([player2])
        .rpc(),
    ]);

    // Verify NFTs were minted
    const nftMintInfo1 = await provider.connection.getAccountInfo(
      nftMintAddress1
    );
    const nftMintInfo2 = await provider.connection.getAccountInfo(
      nftMintAddress2
    );

    assert.isNotNull(nftMintInfo1, "Player 1 NFT was not minted");
    assert.isNotNull(nftMintInfo2, "Player 2 NFT was not minted");

    // Verify the token accounts (ATAs) were initialized properly
    const [player1TokenAccount, player2TokenAccount] = await Promise.all([
      getAccount(provider.connection, player1ATA),
      getAccount(provider.connection, player2ATA),
    ]);

    // Check if accounts exist and contain the correct information
    assert.isNotNull(
      player1TokenAccount,
      "Player 1 token account was not initialized"
    );
    assert.isNotNull(
      player2TokenAccount,
      "Player 2 token account was not initialized"
    );

    // Verify token accounts hold the correct NFT and amount (should be 1)
    expect(player1TokenAccount.mint.toString()).to.equal(
      nftMintAddress1.toString()
    );
    expect(player1TokenAccount.amount.toString()).to.equal("1");
    expect(player1TokenAccount.owner.toString()).to.equal(
      player1.publicKey.toString()
    );

    expect(player2TokenAccount.mint.toString()).to.equal(
      nftMintAddress2.toString()
    );
    expect(player2TokenAccount.amount.toString()).to.equal("1");
    expect(player2TokenAccount.owner.toString()).to.equal(
      player2.publicKey.toString()
    );
  });

  it("Should freeze the nfts and init players", async () => {
    await Promise.all([
      program.methods
        .initPlayer()
        .accountsPartial({
          player: player1.publicKey,
          collectionMint: collectionMint.publicKey,
          nftMint: nftMintAddress1,
          playerAta: player1ATA,
          masterEdition: masterEdition1,
          config: gameConfig,
          metadata: metadata1,
          tokenMetadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
          petStats: petStat1,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([player1])
        .rpc(),

      program.methods
        .initPlayer()
        .accountsPartial({
          player: player2.publicKey,
          collectionMint: collectionMint.publicKey,
          nftMint: nftMintAddress2,
          playerAta: player2ATA,
          masterEdition: masterEdition2,
          config: gameConfig,
          metadata: metadata2,
          tokenMetadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
          sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
          petStats: petStat2,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([player2])
        .rpc(),
    ]);

    // Verify pet stats were initialized correctly
    const [player1PetStats, player2PetStats] = await Promise.all([
      program.account.petStats.fetch(petStat1),
      program.account.petStats.fetch(petStat2),
    ]);

    // Check that pet stats are properly initialized with default values
    expect(player1PetStats.energy.toString()).to.equal("100");
    expect(player1PetStats.hygiene.toString()).to.equal("100");
    expect(player1PetStats.hunger.toString()).to.equal("100");

    expect(player2PetStats.energy.toString()).to.equal("100");
    expect(player2PetStats.hygiene.toString()).to.equal("100");
    expect(player2PetStats.hunger.toString()).to.equal("100");
  });

  it("Should not allow player to transfer the freezed nft's", async () => {
    try {
      // Create a temporary UMI instance with player1's identity
      const player1Umi = createUmi(provider.connection.rpcEndpoint)
        .use(mplTokenMetadata())
        .use(
          keypairIdentity(
            umi.eddsa.createKeypairFromSecretKey(player1.secretKey)
          )
        );

      const transaction = new Transaction();

      const transferInstruction = transferV1(player1Umi, {
        mint: publicKey(nftMintAddress1),
        authority: player1Umi.identity,
        tokenOwner: publicKey(player1.publicKey),
        destinationOwner: publicKey(player2.publicKey),
        tokenStandard: TokenStandard.NonFungible,
      }).getInstructions()[0];

      const web3Instruction = {
        programId: new PublicKey(transferInstruction.programId),
        keys: transferInstruction.keys.map((key) => ({
          pubkey: new PublicKey(key.pubkey),
          isSigner: key.isSigner,
          isWritable: key.isWritable,
        })),
        data: Buffer.from(transferInstruction.data),
      };

      transaction.add(web3Instruction);

      await sendAndConfirmTransaction(provider.connection, transaction, [
        player1,
      ]);

      assert.fail("Transfer should have failed because the NFT is locked");
    } catch (error) {
      expect(error).to.exist;

      const errorString = error.toString().toLowerCase();
      const isLockError =
        errorString.includes("locked") ||
        errorString.includes("frozen") ||
        errorString.includes("delegated") ||
        errorString.includes("unauthorized") ||
        errorString.includes("permission") ||
        errorString.includes("not allowed");

      expect(isLockError).to.be.true;
    }
  });

  it("Should initialize a new duel challenge", async () => {
    await program.methods
      .initPetDuel(new anchor.BN(0))
      .accountsPartial({
        challanger: player1.publicKey,
        gameConfig: gameConfig,
        gameVault: gameVault,
      })
      .signers([player1])
      .rpc();

    const onChainDuel = await program.account.petDuel.fetch(petDuel);

    expect(onChainDuel.challenger.toBase58()).to.equals(
      player1.publicKey.toBase58()
    );
    expect(onChainDuel.betAmount.toString()).to.equals("0");
    expect(onChainDuel.winner).to.equals(null);
    expect(onChainDuel.challengerPetHealth.toString()).to.equals("100");
    expect(onChainDuel.defenderPetHealth.toString()).to.equals("100");
    expect(onChainDuel.challengerTurn).to.equals(true);
  });

  it("Should not allowed to create another duel, when a duel is active", async () => {
    try {
      await program.methods
        .initPetDuel(new anchor.BN(0))
        .accountsPartial({
          challanger: player1.publicKey,
          gameConfig: gameConfig,
          gameVault: gameVault,
        })
        .signers([player1])
        .rpc();
    } catch (error) {
      assert.isOk(error.message, "AlreadyInitialized");
    }
  });

  it("Should accept an existing duel", async () => {
    await program.methods
      .acceptPetDuel()
      .accountsPartial({
        defender: player2.publicKey,
        challenger: player1.publicKey,
        gameConfig,
        petStats: petStat2,
        petDuelAccount: petDuel,
      })
      .signers([player2])
      .rpc();
    const onChainDuel = await program.account.petDuel.fetch(petDuel);

    expect(onChainDuel.defender.toBase58()).to.equals(
      player2.publicKey.toBase58()
    );
  });

  it("Defender should not allowed to make the first move", async () => {
    const account = await provider.connection.getAccountInfo(
      petDuel,
      "confirmed"
    );

    let sig_ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: player2.secretKey,
      message: account.data.subarray(8),
    });

    const attack_ix = await program.methods
      .petAttack(Buffer.from(sig_ix.data.buffer.slice(16 + 32, 16 + 32 + 64)))
      .accountsPartial({
        attacker: player2.publicKey,
        challanger: player1.publicKey,
        petDuelAccount: petDuel,
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([player2])
      .instruction();

    const tx = new Transaction().add(sig_ix).add(attack_ix);
    try {
      await sendAndConfirmTransaction(provider.connection, tx, [player2]);
    } catch (error) {
      assert.isOk(error.message, "NotChallengerTurn.");
    }
  });

  it("challanger should allowed to perform attack", async () => {
    const account = await provider.connection.getAccountInfo(
      petDuel,
      "confirmed"
    );

    const sig_ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: player1.secretKey,
      message: account.data.subarray(8),
    });

    const attack_ix = await program.methods
      .petAttack(Buffer.from(sig_ix.data.buffer.slice(16 + 32, 16 + 32 + 64)))
      .accountsPartial({
        attacker: player1.publicKey,
        challanger: player1.publicKey,
        petDuelAccount: petDuel,
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([player1])
      .instruction();

    const tx = new Transaction().add(sig_ix).add(attack_ix);

    await sendAndConfirmTransaction(provider.connection, tx, [player1]);

    const onChainDuel = await program.account.petDuel.fetch(petDuel);

    expect(onChainDuel.challengerTurn).to.equal(false);
    expect(onChainDuel.defenderPetHealth).to.lessThan(100);
  });

  it("Challanger should not be allowed to perfrom attack", async () => {
    const account = await provider.connection.getAccountInfo(
      petDuel,
      "confirmed"
    );

    let sig_ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: player1.secretKey,
      message: account.data.subarray(8),
    });

    const attack_ix = await program.methods
      .petAttack(Buffer.from(sig_ix.data.buffer.slice(16 + 32, 16 + 32 + 64)))
      .accountsPartial({
        attacker: player1.publicKey,
        challanger: player1.publicKey,
        petDuelAccount: petDuel,
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([player1])
      .instruction();

    const tx = new Transaction().add(sig_ix).add(attack_ix);
    try {
      await sendAndConfirmTransaction(provider.connection, tx, [player1]);
    } catch (error) {
      assert.isOk(error.message, "NotDefenderTurn");
    }
  });

  it("Defender should allowed to perfrom attack", async () => {
    const account = await provider.connection.getAccountInfo(
      petDuel,
      "confirmed"
    );

    let sig_ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: player2.secretKey,
      message: account.data.subarray(8),
    });

    await program.methods
      .petAttack(Buffer.from(sig_ix.data.buffer.slice(16 + 32, 16 + 32 + 64)))
      .accountsPartial({
        attacker: player2.publicKey,
        challanger: player1.publicKey,
        petDuelAccount: petDuel,
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([player2])
      .rpc();

    const onChainDuel = await program.account.petDuel.fetch(petDuel);

    expect(onChainDuel.challengerTurn).to.equal(true);
    expect(onChainDuel.challengerPetHealth).to.lessThan(100);
  });

  it("Fight should end when pet health drops to zero", async () => {
    let duelAccount = await program.account.petDuel.fetch(petDuel);

    while (
      duelAccount.challengerPetHealth > 0 &&
      duelAccount.defenderPetHealth > 0
    ) {
      const account = await provider.connection.getAccountInfo(
        petDuel,
        "confirmed"
      );

      const currentAttacker = duelAccount.challengerTurn ? player1 : player2;
      let sig_ix = Ed25519Program.createInstructionWithPrivateKey({
        privateKey: currentAttacker.secretKey,
        message: account.data.subarray(8),
      });

      await program.methods
        .petAttack(Buffer.from(sig_ix.data.buffer.slice(16 + 32, 16 + 32 + 64)))
        .accountsPartial({
          attacker: currentAttacker.publicKey,
          challanger: player1.publicKey,
          petDuelAccount: petDuel,
          instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .signers([currentAttacker])
        .rpc();
      duelAccount = await program.account.petDuel.fetch(petDuel);
    }
    const finalDuel = await program.account.petDuel.fetch(petDuel);

    expect(finalDuel.winner).to.not.equal(null);
  });

  it("non winner should not be allowed to claim the win amount", async () => {
    let non_winner: Keypair;

    const onChainDuel = await program.account.petDuel.fetch(petDuel);

    if (onChainDuel.winner.equals(player1.publicKey)) {
      non_winner = player2;
    } else {
      non_winner = player1;
    }

    try {
      await program.methods
        .claimBet()
        .accountsPartial({
          winner: non_winner.publicKey,
          challanger: player1.publicKey,
          gameConfig,
          petDuelAccount: petDuel,
          gameVault,
        })
        .signers([non_winner])
        .rpc();
    } catch (error) {
      assert.isOk(error.message, "UnauthorizedAction.");
    }
  });

  it("Winner should be allowed to claim the win amount", async () => {
    let winner: Keypair;

    const onChainDuel = await program.account.petDuel.fetch(petDuel);

    if (onChainDuel.winner.equals(player1.publicKey)) {
      winner = player1;
    } else {
      winner = player2;
    }

    await program.methods
      .claimBet()
      .accountsPartial({
        winner: onChainDuel.winner,
        challanger: player1.publicKey,
        gameConfig,
        petDuelAccount: petDuel,
        gameVault,
      })
      .signers([winner])
      .rpc();

    try {
      await program.account.petDuel.fetch(petDuel);
    } catch (error) {
      assert.isOk(error.message, "Account does not exist or has no data");
    }
  });

  it("Should interact with pet", async () => {
    await Promise.all([
      program.methods
        .petInteract({ feed: {} })
        .accountsPartial({ player: player1.publicKey, petStats: petStat1 })
        .signers([player1])
        .rpc(),
      program.methods
        .petInteract({ bath: {} })
        .accountsPartial({ player: player1.publicKey, petStats: petStat1 })
        .signers([player1])
        .rpc(),
    ]);
  });

  it("Should unlock the NFT, revoke delegate, and close all the stats accounts", async () => {
    await program.methods
      .closePlayer()
      .accountsPartial({
        player: player1.publicKey,
        collectionMint: collectionMint.publicKey,
        nftMint: nftMintAddress1,
        playerAta: player1ATA,
        config: gameConfig,
        masterEdition: masterEdition1,
        metadata: metadata1,
        petStats: petStat1,
        tokenMetadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
        sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([player1])
      .rpc();

    // Verify that the pet stats account was closed
    try {
      await program.account.petStats.fetch(petStat1);
      assert.fail("Pet stats account should have been closed");
    } catch (error) {
      expect(error.toString()).to.include("Account does not exist");
    }
  });

  it("Should enable player to transfer the nfts back to their wallet", async () => {
    try {
      const player1Umi = createUmi(provider.connection.rpcEndpoint)
        .use(mplTokenMetadata())
        .use(
          keypairIdentity(
            umi.eddsa.createKeypairFromSecretKey(player1.secretKey)
          )
        );

      const transaction = new Transaction();
      const transferInstruction = transferV1(player1Umi, {
        mint: publicKey(nftMintAddress1),
        authority: player1Umi.identity,
        tokenOwner: publicKey(player1.publicKey),
        destinationOwner: publicKey(player2.publicKey),
        tokenStandard: TokenStandard.NonFungible,
      }).getInstructions()[0];

      const web3Instruction = {
        programId: new PublicKey(transferInstruction.programId),
        keys: transferInstruction.keys.map((key) => ({
          pubkey: new PublicKey(key.pubkey),
          isSigner: key.isSigner,
          isWritable: key.isWritable,
        })),
        data: Buffer.from(transferInstruction.data),
      };

      transaction.add(web3Instruction);
      await sendAndConfirmTransaction(provider.connection, transaction, [
        player1,
      ]);
    } catch (error) {
      assert.fail("Should be able to transfer NFT after unlocking");
    }
  });
});
