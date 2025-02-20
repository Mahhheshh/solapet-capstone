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
  verifyCollectionV1,
} from "@metaplex-foundation/mpl-token-metadata";

import {
  generateSigner,
  keypairIdentity,
  percentAmount,
  publicKey,
} from "@metaplex-foundation/umi";
import {
  getAccount,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
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

  // umi stuff
  const umi = createUmi(provider.connection.rpcEndpoint).use(
    mplTokenMetadata()
  );

  let umiAdmin = umi.eddsa.createKeypairFromSecretKey(admin.secretKey);
  umi.use(keypairIdentity(umiAdmin));

  const nftMintPlayer1 = generateSigner(umi);
  const nftMintPlayer2 = generateSigner(umi);
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
      nftMintPlayer1: nftMintPlayer1.publicKey,
      nftMintPlayer2: nftMintPlayer2.publicKey,
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
  });

  it("Should successfully deposit an NFT to the game", async () => {
    // create nfts
    await Promise.all([
      createNft(umi, {
        mint: nftMintPlayer1,
        name: "player1",
        symbol: "p1",
        uri: "", // Add Arweave or IPFS metadata
        sellerFeeBasisPoints: percentAmount(0),
        collection: { key: collectionMint.publicKey, verified: false },
      }).sendAndConfirm(umi, {
        confirm: { commitment: "finalized" },
        send: { commitment: "finalized" },
      }),
      createNft(umi, {
        mint: nftMintPlayer2,
        name: "player2",
        symbol: "p2",
        uri: "", // Add Arweave or IPFS metadata
        sellerFeeBasisPoints: percentAmount(0),
        collection: { key: collectionMint.publicKey, verified: false },
      }).sendAndConfirm(umi, {
        confirm: { commitment: "finalized" },
        send: { commitment: "finalized" },
      }),
    ]);

    const metadata = findMetadataPda(umi, {
      mint: nftMintPlayer1.publicKey,
    });

    const metadataTwo = findMetadataPda(umi, {
      mint: nftMintPlayer2.publicKey,
    });

    // verify the nft with collection
    await Promise.all([
      verifyCollectionV1(umi, {
        metadata,
        collectionMint: collectionMint.publicKey,
      }),

      verifyCollectionV1(umi, {
        metadata: metadataTwo,
        collectionMint: collectionMint.publicKey,
      }),

      transferV1(umi, {
        mint: nftMintPlayer1.publicKey,
        authority: umi.identity,
        destinationOwner: publicKey(player1.publicKey),
        tokenStandard: TokenStandard.NonFungible,
      }).sendAndConfirm(umi),

      transferV1(umi, {
        mint: nftMintPlayer2.publicKey,
        authority: umi.identity,
        destinationOwner: publicKey(player2.publicKey),
        tokenStandard: TokenStandard.NonFungible,
      }).sendAndConfirm(umi),
    ]);

    const [player1ATA, player2ATA] = await Promise.all([
      getAssociatedTokenAddress(
        new PublicKey(nftMintPlayer1.publicKey),
        player1.publicKey
      ),
      getAssociatedTokenAddress(
        new PublicKey(nftMintPlayer2.publicKey),
        player2.publicKey
      ),
    ]);

    await Promise.all([
      program.methods
        .initPlayer()
        .accountsPartial({
          player: player1.publicKey,
          playerAta: player1ATA,
          collectionMint: collectionMint.publicKey,
          nftMint: nftMintPlayer1.publicKey,
        })
        .signers([player1])
        .rpc(),

      program.methods
        .initPlayer()
        .accountsPartial({
          player: player2.publicKey,
          playerAta: player2ATA,
          collectionMint: collectionMint.publicKey,
          nftMint: nftMintPlayer2.publicKey,
        })
        .signers([player2])
        .rpc(),
    ]);

    const [player1pet, player2pet] = await Promise.all([
      await program.account.petStats.fetch(petStat1),
      await program.account.petStats.fetch(petStat2),
    ]);

    expect(player1pet.energy.toString()).to.equal("100");
    expect(player1pet.hygiene.toString()).to.equal("100");
    expect(player1pet.hunger.toString()).to.equal("100");

    expect(player2pet.energy.toString()).to.equal("100");
    expect(player2pet.hygiene.toString()).to.equal("100");
    expect(player2pet.hunger.toString()).to.equal("100");
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

  // it("Should transfer nft back and close all the stats accounts", async () => {
  //   const player1TokenAta = await getAssociatedTokenAddress(
  //     new PublicKey(nftMintPlayer1.publicKey),
  //     player1.publicKey
  //   );
  //   const gameAta = getAssociatedTokenAddressSync(
  //     new PublicKey(nftMintPlayer1.publicKey),
  //     gameConfig,
  //     true
  //   );
  //   console.log(await getAccount(provider.connection, gameAta));
  //   await program.methods
  //     .closePlayer()
  //     .accountsPartial({
  //       player: player1.publicKey,
  //       collectionMint: collectionMint.publicKey,
  //       nftMint: nftMintPlayer1.publicKey,
  //       config: gameConfig,
  //       playerAta: player1TokenAta,
  //       gameAta: gameAta,
  //       petStats: petStat1,
  //     })
  //     .signers([player1])
  //     .rpc();
  // });
});
