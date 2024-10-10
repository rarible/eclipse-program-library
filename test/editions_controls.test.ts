import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
  PublicKey,
  Keypair,
  SystemProgram,
  ComputeBudgetProgram,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
  getTokenMetadata,
} from '@solana/spl-token';
import { LibreplexEditionsControls } from '../../eclipse-program-library/target/types/libreplex_editions_controls';
import { LibreplexEditions } from '../../eclipse-program-library/target/types/libreplex_editions';
import { expect } from 'chai';
import { describe, it } from 'mocha';
import {
  getCluster,
  getEditions,
  getEditionsControls,
  getMinterStats,
  logEditions,
  logEditionsControls,
  logMinterStats,
  logMinterStatsPhase,
  logTokenMetadata,
  parseMetadata,
} from './utils';
import { Transaction } from '@solana/web3.js';
import {
  EDITIONS_CONTROLS_PROGRAM_ID,
  EDITIONS_PROGRAM_ID,
  TOKEN_GROUP_EXTENSION_PROGRAM_ID,
} from '../constants';
import { toBufferLE } from 'bigint-buffer';

const VERBOSE_LOGGING = true;

describe('Editions Controls Test Suite', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let editionsControlsProgram: Program<LibreplexEditionsControls>;
  let editionsProgram: Program<LibreplexEditions>;

  let editionsPda: PublicKey;
  let editionsControlsPda: PublicKey;
  let hashlistPda: PublicKey;

  let payer: Keypair;
  let creator1: Keypair;
  let creator2: Keypair;
  let treasury: Keypair;
  let platformFeeAdmin: Keypair;
  let groupMint: Keypair;
  let group: Keypair;

  let minter1: Keypair;
  let minter2: Keypair;

  let collectionConfig: {
    symbol: string;
    maxMintsPerWallet: anchor.BN;
    maxNumberOfTokens: anchor.BN;
    collectionName: string;
    collectionUri: string;
    royalties: {
      royaltyBasisPoints: anchor.BN;
      creators: { address: PublicKey; share: number }[];
    };
    platformFee: {
      platformFeeValue: anchor.BN;
      recipients: { address: PublicKey; share: number }[];
      isFeeFlat: boolean;
    };
    extraMeta: { field: string; value: string }[];
    itemBaseUri: string;
    itemBaseName: string;
    treasury: PublicKey;
    cosignerProgramId: PublicKey | null;
  };

  let allowListConfig: {
    merkleRoot: Buffer;
    list: {
      address: PublicKey;
      price: anchor.BN;
      max_claims: anchor.BN;
      proof: Buffer[];
    }[];
  };

  before(async () => {
    if (VERBOSE_LOGGING) {
      const cluster = await getCluster(provider.connection);
      console.log('Cluster:', cluster);
    }

    editionsControlsProgram = anchor.workspace
      .LibreplexEditionsControls as Program<LibreplexEditionsControls>;
    editionsProgram = anchor.workspace
      .LibreplexEditions as Program<LibreplexEditions>;

    payer = (provider.wallet as anchor.Wallet).payer;
    creator1 = Keypair.generate();
    creator2 = Keypair.generate();
    treasury = Keypair.generate();
    platformFeeAdmin = Keypair.generate();
    groupMint = Keypair.generate();
    group = Keypair.generate();

    collectionConfig = {
      maxNumberOfTokens: new anchor.BN(1150),
      symbol: 'COOLX55',
      collectionName: 'Collection name with meta, platform fee and royalties',
      collectionUri: 'ipfs://QmbsXNSkPUtYNmKfYw1mUSVuz9QU8nhu7YvzM1aAQsv6xw/0',
      treasury: treasury.publicKey,
      maxMintsPerWallet: new anchor.BN(100),
      royalties: {
        royaltyBasisPoints: new anchor.BN(1000),
        creators: [
          {
            address: creator1.publicKey,
            share: 50,
          },
          {
            address: creator2.publicKey,
            share: 50,
          },
        ],
      },
      platformFee: {
        platformFeeValue: new anchor.BN(500000),
        recipients: [
          {
            address: platformFeeAdmin.publicKey,
            share: 100,
          },
        ],
        isFeeFlat: true,
      },
      extraMeta: [
        { field: 'field1', value: 'value1' },
        { field: 'field2', value: 'value2' },
        { field: 'field3', value: 'value3' },
        { field: 'field4', value: 'value4' },
      ],
      itemBaseUri: 'ipfs://QmbsXNSkPUtYNmKfYw1mUSVuz9QU8nhu7YvzM1aAQsv6xw/{}',
      itemBaseName: 'Item T8 V4 #{}',
      cosignerProgramId: null,
    };

    if (VERBOSE_LOGGING) {
      console.log('Collection config: ', collectionConfig);
    }

    [editionsPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('editions_deployment'),
        Buffer.from(collectionConfig.symbol),
      ],
      editionsProgram.programId
    );

    [editionsControlsPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('editions_controls'), editionsPda.toBuffer()],
      editionsControlsProgram.programId
    );

    [hashlistPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('hashlist'), editionsPda.toBuffer()],
      editionsProgram.programId
    );
  });

  it('should deploy a collection, add a phase, and execute a mint', async () => {
    // Modify compute units for the transaction
    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
      units: 800000,
    });

    try {
      const initialiseIx = await editionsControlsProgram.methods
        .initialiseEditionsControls({
          maxMintsPerWallet: collectionConfig.maxMintsPerWallet,
          treasury: collectionConfig.treasury,
          maxNumberOfTokens: collectionConfig.maxNumberOfTokens,
          symbol: collectionConfig.symbol,
          collectionName: collectionConfig.collectionName,
          collectionUri: collectionConfig.collectionUri,
          cosignerProgramId: collectionConfig.cosignerProgramId,
          royalties: collectionConfig.royalties,
          platformFee: collectionConfig.platformFee,
          extraMeta: collectionConfig.extraMeta,
          itemBaseUri: collectionConfig.itemBaseUri,
          itemBaseName: collectionConfig.itemBaseName,
        })
        .accountsStrict({
          editionsControls: editionsControlsPda,
          editionsDeployment: editionsPda,
          hashlist: hashlistPda,
          payer: payer.publicKey,
          creator: payer.publicKey,
          groupMint: groupMint.publicKey,
          group: group.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          libreplexEditionsProgram: editionsProgram.programId,
          groupExtensionProgram: new PublicKey(
            '5hx15GaPPqsYA61v6QpcGPpo125v7rfvEfZQ4dJErG5V'
          ),
        })
        .instruction();

      const transaction = new Transaction()
        .add(modifyComputeUnits)
        .add(initialiseIx);

      await provider.sendAndConfirm(transaction, [groupMint, group, payer]);

      // Fetch updated state
      const editionsDecoded = await getEditions(
        provider.connection,
        editionsPda,
        editionsProgram
      );
      const editionsControlsDecoded = await getEditionsControls(
        provider.connection,
        editionsControlsPda,
        editionsControlsProgram
      );
      const metadata = await getTokenMetadata(
        provider.connection,
        groupMint.publicKey
      );

      if (VERBOSE_LOGGING) {
        logEditions(editionsDecoded);
        logEditionsControls(editionsControlsDecoded);
        logTokenMetadata(metadata);
      }

      // Verify Editions deployment
      expect(editionsDecoded.data.symbol).to.equal(collectionConfig.symbol);
      expect(editionsDecoded.data.creator.toBase58()).to.equal(
        editionsControlsPda.toBase58()
      );
      expect(editionsDecoded.data.maxNumberOfTokens.toString()).to.equal(
        collectionConfig.maxNumberOfTokens.toString()
      );
      expect(editionsDecoded.data.itemBaseName).to.equal(
        collectionConfig.itemBaseName
      );
      expect(editionsDecoded.data.itemBaseUri).to.equal(
        collectionConfig.itemBaseUri
      );

      // Verify EditionsControls deployment
      expect(
        editionsControlsDecoded.data.editionsDeployment.toBase58()
      ).to.equal(editionsPda.toBase58());
      expect(editionsControlsDecoded.data.creator.toBase58()).to.equal(
        payer.publicKey.toBase58()
      );
      expect(editionsControlsDecoded.data.treasury.toBase58()).to.equal(
        collectionConfig.treasury.toBase58()
      );
      expect(Number(editionsControlsDecoded.data.maxMintsPerWallet)).to.equal(
        Number(collectionConfig.maxMintsPerWallet)
      );
      expect(editionsControlsDecoded.data.phases.length).to.equal(0);

      // Verify metadata
      const parsedMetadata = parseMetadata(metadata.additionalMetadata);
      expect(metadata.name).to.equal(collectionConfig.collectionName);
      expect(metadata.uri).to.equal(collectionConfig.collectionUri);
      expect(metadata.mint.toBase58()).to.equal(groupMint.publicKey.toBase58());
      // Verify that every key in extraMeta is present in metadata.additionalMetadata
      collectionConfig.extraMeta.forEach((meta) => {
        expect(parsedMetadata).to.have.property(meta.field);
        expect(parsedMetadata[meta.field]).to.equal(meta.value);
      });

      // Add more assertions as needed
    } catch (error) {
      console.error('Error in initialiseEditionsControls:', error);
      throw error;
    }
  });

  // 2. Add a phase
  it('Should add a phase without allowlist', async () => {
    const phaseConfig = {
      maxMintsPerWallet: new anchor.BN(100),
      maxMintsTotal: new anchor.BN(1000),
      priceAmount: new anchor.BN(10000000), // 0.01 SOL
      startTime: new anchor.BN(new Date().getTime() / 1000),
      endTime: new anchor.BN(new Date().getTime() / 1000 + 60 * 60 * 24), // 1 day from now
      priceToken: new PublicKey('So11111111111111111111111111111111111111112'),
      merkleRoot: null,
    };

    const phaseIx = await editionsControlsProgram.methods
      .addPhase(phaseConfig)
      .accountsStrict({
        editionsControls: editionsControlsPda,
        creator: payer.publicKey,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        libreplexEditionsProgram: editionsProgram.programId,
      })
      .signers([])
      .instruction();

    const transaction = new Transaction().add(phaseIx);
    await provider.sendAndConfirm(transaction, [payer]);

    // get state
    const editionsDecoded = await getEditions(
      provider.connection,
      editionsPda,
      editionsProgram
    );
    const editionsControlsDecoded = await getEditionsControls(
      provider.connection,
      editionsControlsPda,
      editionsControlsProgram
    );
    if (VERBOSE_LOGGING) {
      logEditions(editionsDecoded);
      logEditionsControls(editionsControlsDecoded);
    }

    // verify state
    expect(editionsControlsDecoded.data.phases.length).to.equal(1);
    expect(
      editionsControlsDecoded.data.phases[0].maxMintsPerWallet.toString()
    ).to.equal(phaseConfig.maxMintsPerWallet.toString());
    expect(
      editionsControlsDecoded.data.phases[0].maxMintsTotal.toString()
    ).to.equal(phaseConfig.maxMintsTotal.toString());
  });

  // Generate allowlist variables
  before(async () => {
    minter1 = Keypair.fromSecretKey(
      new Uint8Array([
        110, 76, 59, 154, 201, 225, 246, 121, 152, 90, 45, 211, 52, 244, 216,
        108, 118, 248, 113, 239, 61, 248, 207, 122, 98, 26, 184, 92, 51, 97, 52,
        218, 104, 164, 83, 51, 23, 177, 193, 29, 252, 241, 86, 132, 173, 155,
        114, 131, 130, 73, 27, 101, 233, 95, 12, 45, 107, 255, 120, 26, 121,
        221, 120, 54,
      ])
    );
    minter2 = Keypair.fromSecretKey(
      new Uint8Array([
        16, 27, 49, 140, 228, 142, 201, 93, 199, 209, 62, 136, 151, 212, 238,
        114, 46, 204, 155, 132, 26, 227, 44, 245, 239, 29, 195, 63, 77, 162, 28,
        220, 186, 39, 133, 92, 39, 241, 42, 161, 180, 15, 92, 18, 15, 101, 248,
        80, 238, 254, 220, 231, 1, 14, 231, 145, 170, 49, 163, 111, 239, 112,
        135, 6,
      ])
    );
    allowListConfig = {
      merkleRoot: Buffer.from([
        125, 184, 194, 116, 52, 36, 65, 219, 171, 135, 154, 27, 188, 122, 207,
        204, 111, 70, 66, 115, 161, 228, 44, 84, 67, 97, 29, 70, 253, 69, 11,
        245,
      ]),
      list: [
        {
          address: minter1.publicKey,
          price: new anchor.BN(500000), // 0.005 SOL
          max_claims: new anchor.BN(3),
          proof: [
            Buffer.from([
              64, 131, 242, 169, 206, 112, 155, 119, 81, 214, 17, 137, 174, 140,
              208, 220, 141, 177, 213, 131, 127, 104, 181, 15, 121, 228, 87, 25,
              232, 172, 235, 168,
            ]),
          ],
        },
        {
          address: minter2.publicKey,
          price: new anchor.BN(500000), // 0.005 SOL
          max_claims: new anchor.BN(3),
          proof: [
            Buffer.from([
              86, 37, 15, 136, 192, 159, 125, 244, 163, 213, 251, 242, 217, 215,
              159, 249, 93, 166, 82, 38, 187, 58, 199, 64, 161, 50, 122, 122,
              17, 125, 63, 188,
            ]),
          ],
        },
      ],
    };
  });

  it('Should add a phase with allowlist', async () => {
    const phaseConfig = {
      maxMintsPerWallet: new anchor.BN(100),
      maxMintsTotal: new anchor.BN(1000),
      priceAmount: new anchor.BN(10000000), // 0.01 SOL
      startTime: new anchor.BN(new Date().getTime() / 1000),
      endTime: new anchor.BN(new Date().getTime() / 1000 + 60 * 60 * 24), // 1 day from now
      priceToken: new PublicKey('So11111111111111111111111111111111111111112'),
      merkleRoot: allowListConfig.merkleRoot,
    };

    const phaseIx = await editionsControlsProgram.methods
      .addPhase(phaseConfig)
      .accountsStrict({
        editionsControls: editionsControlsPda,
        creator: payer.publicKey,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        libreplexEditionsProgram: editionsProgram.programId,
      })
      .signers([])
      .instruction();

    const transaction = new Transaction().add(phaseIx);
    await provider.sendAndConfirm(transaction, [payer]);

    const editionsDecoded = await getEditions(
      provider.connection,
      editionsPda,
      editionsProgram
    );
    const editionsControlsDecoded = await getEditionsControls(
      provider.connection,
      editionsControlsPda,
      editionsControlsProgram
    );
    if (VERBOSE_LOGGING) {
      logEditions(editionsDecoded);
      logEditionsControls(editionsControlsDecoded);
    }

    // verify state
    expect(editionsControlsDecoded.data.phases.length).to.equal(2);
    expect(
      editionsControlsDecoded.data.phases[1].maxMintsPerWallet.toString()
    ).to.equal(phaseConfig.maxMintsPerWallet.toString());
    expect(
      editionsControlsDecoded.data.phases[1].maxMintsTotal.toString()
    ).to.equal(phaseConfig.maxMintsTotal.toString());
  });

  before(async () => {
    // Airdrop SOL to minter1
    const airdropSignature = await provider.connection.requestAirdrop(
      minter1.publicKey,
      1 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);

    // Airdrop SOL to treasury
    const treasuryAirdropSignature = await provider.connection.requestAirdrop(
      collectionConfig.treasury,
      1 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(treasuryAirdropSignature);

    // Airdrop SOL to platformFeeRecipient
    const platformFeeRecipientAirdropSignature =
      await provider.connection.requestAirdrop(
        platformFeeAdmin.publicKey,
        1 * LAMPORTS_PER_SOL
      );
    await provider.connection.confirmTransaction(
      platformFeeRecipientAirdropSignature
    );
  });

  it('Should mint on first phase without allowlist', async () => {
    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
      units: 850000,
    });

    const mintConfig = {
      phaseIndex: 0,
      merkleProof: null,
      allowListPrice: null,
      allowListMaxClaims: null,
    };

    const mint = Keypair.generate();
    const member = Keypair.generate();

    const tokenAccount = getAssociatedTokenAddressSync(
      mint.publicKey,
      minter1.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    const [hashlistMarker] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('hashlist_marker'),
        editionsPda.toBuffer(),
        mint.publicKey.toBuffer(),
      ],
      new PublicKey(EDITIONS_PROGRAM_ID)
    );

    const [minterStats] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('minter_stats'),
        editionsPda.toBuffer(),
        minter1.publicKey.toBuffer(),
      ],
      new PublicKey(EDITIONS_CONTROLS_PROGRAM_ID)
    );

    const [minterStatsPhase] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('minter_stats_phase'),
        editionsPda.toBuffer(),
        minter1.publicKey.toBuffer(),
        toBufferLE(BigInt(0), 4),
      ],
      new PublicKey(EDITIONS_CONTROLS_PROGRAM_ID)
    );

    const mintIx = await editionsControlsProgram.methods
      .mintWithControls(mintConfig)
      .accountsStrict({
        editionsDeployment: editionsPda,
        editionsControls: editionsControlsPda,
        hashlist: hashlistPda,
        hashlistMarker,
        payer: minter1.publicKey,
        mint: mint.publicKey,
        member: member.publicKey,
        signer: minter1.publicKey,
        minter: minter1.publicKey,
        minterStats,
        minterStatsPhase,
        group: group.publicKey,
        groupMint: groupMint.publicKey,
        platformFeeRecipient1: platformFeeAdmin.publicKey,
        groupExtensionProgram: new PublicKey(TOKEN_GROUP_EXTENSION_PROGRAM_ID),
        tokenAccount,
        treasury: collectionConfig.treasury,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        libreplexEditionsProgram: editionsProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .instruction();

    const transaction = new Transaction().add(modifyComputeUnits).add(mintIx);
    transaction.recentBlockhash = (
      await provider.connection.getLatestBlockhash()
    ).blockhash;
    transaction.feePayer = minter1.publicKey;
    transaction.sign(minter1, mint, member);
    const rawTransaction = transaction.serialize();

    try {
      const signature = await provider.connection.sendRawTransaction(
        rawTransaction,
        {
          skipPreflight: false,
          preflightCommitment: 'confirmed',
        }
      );
      await provider.connection.confirmTransaction(signature);

      console.log('\nmintWithControls done!\n');
    } catch (error) {
      if (error.logs) {
        console.error('Full error logs:');
        error.logs.forEach((log, index) => {
          console.error(`${index + 1}: ${log}`);
        });
      } else {
        console.error(error);
      }

      throw error;
    }

    // get state
    const editionsDecoded = await getEditions(
      provider.connection,
      editionsPda,
      editionsProgram
    );
    const editionsControlsDecoded = await getEditionsControls(
      provider.connection,
      editionsControlsPda,
      editionsControlsProgram
    );
    const minterStatsDecoded = await getMinterStats(
      provider.connection,
      minterStats,
      editionsControlsProgram
    );
    const minterStatsPhaseDecoded = await getMinterStats(
      provider.connection,
      minterStatsPhase,
      editionsControlsProgram
    );
    if (VERBOSE_LOGGING) {
      logEditions(editionsDecoded);
      logEditionsControls(editionsControlsDecoded);
      logMinterStats(minterStatsDecoded);
      logMinterStatsPhase(minterStatsPhaseDecoded);
    }

    // verify state
    expect(
      editionsControlsDecoded.data.phases[0].currentMints.toString()
    ).to.equal(new anchor.BN(1).toString());
    expect(editionsDecoded.data.numberOfTokensIssued.toString()).to.equal(
      new anchor.BN(1).toString()
    );
    expect(minterStatsDecoded.data.mintCount.toString()).to.equal(
      new anchor.BN(1).toString()
    );
    expect(minterStatsPhaseDecoded.data.mintCount.toString()).to.equal(
      new anchor.BN(1).toString()
    );
  });

  it('Should mint on second phase with allowlist', async () => {
    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
      units: 850000,
    });

    const mintConfig = {
      phaseIndex: 1, // Use the second phase (index 1)
      merkleProof: allowListConfig.list[0].proof, // Use the proof for minter1
      allowListPrice: allowListConfig.list[0].price,
      allowListMaxClaims: allowListConfig.list[0].max_claims,
    };

    const mint = Keypair.generate();
    const member = Keypair.generate();

    const tokenAccount = getAssociatedTokenAddressSync(
      mint.publicKey,
      minter1.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    const [hashlistMarker] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('hashlist_marker'),
        editionsPda.toBuffer(),
        mint.publicKey.toBuffer(),
      ],
      new PublicKey(EDITIONS_PROGRAM_ID)
    );

    const [minterStats] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('minter_stats'),
        editionsPda.toBuffer(),
        minter1.publicKey.toBuffer(),
      ],
      new PublicKey(EDITIONS_CONTROLS_PROGRAM_ID)
    );

    const [minterStatsPhase] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('minter_stats_phase'),
        editionsPda.toBuffer(),
        minter1.publicKey.toBuffer(),
        toBufferLE(BigInt(1), 4), // Use phase index 1
      ],
      new PublicKey(EDITIONS_CONTROLS_PROGRAM_ID)
    );

    const mintIx = await editionsControlsProgram.methods
      .mintWithControls(mintConfig)
      .accountsStrict({
        editionsDeployment: editionsPda,
        editionsControls: editionsControlsPda,
        hashlist: hashlistPda,
        hashlistMarker,
        payer: minter1.publicKey,
        mint: mint.publicKey,
        member: member.publicKey,
        signer: minter1.publicKey,
        minter: minter1.publicKey,
        minterStats,
        minterStatsPhase,
        group: group.publicKey,
        groupMint: groupMint.publicKey,
        platformFeeRecipient1: platformFeeAdmin.publicKey,
        groupExtensionProgram: new PublicKey(TOKEN_GROUP_EXTENSION_PROGRAM_ID),
        tokenAccount,
        treasury: collectionConfig.treasury,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        libreplexEditionsProgram: editionsProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .instruction();

    const transaction = new Transaction().add(modifyComputeUnits).add(mintIx);
    transaction.recentBlockhash = (
      await provider.connection.getLatestBlockhash()
    ).blockhash;
    transaction.feePayer = minter1.publicKey;
    transaction.sign(minter1, mint, member);
    const rawTransaction = transaction.serialize();

    try {
      const signature = await provider.connection.sendRawTransaction(
        rawTransaction,
        {
          skipPreflight: false,
          preflightCommitment: 'confirmed',
        }
      );
      // wait for transaction to be confirmed
      await provider.connection.confirmTransaction(signature);

      console.log('\nmintWithControls on allowlist phase done!\n');

      // Fetch updated state
      const editionsDecoded = await getEditions(
        provider.connection,
        editionsPda,
        editionsProgram
      );
      const editionsControlsDecoded = await getEditionsControls(
        provider.connection,
        editionsControlsPda,
        editionsControlsProgram
      );
      const minterStatsDecoded = await getMinterStats(
        provider.connection,
        minterStats,
        editionsControlsProgram
      );
      const minterStatsPhaseDecoded = await getMinterStats(
        provider.connection,
        minterStatsPhase,
        editionsControlsProgram
      );
      if (VERBOSE_LOGGING) {
        logEditions(editionsDecoded);
        logEditionsControls(editionsControlsDecoded);
        logMinterStats(minterStatsDecoded);
        logMinterStatsPhase(minterStatsPhaseDecoded);
      }

      // Verify state
      expect(
        editionsControlsDecoded.data.phases[1].currentMints.toString()
      ).to.equal('1');
      expect(editionsDecoded.data.numberOfTokensIssued.toString()).to.equal(
        '2'
      );
      expect(minterStatsDecoded.data.mintCount.toString()).to.equal('2');
      expect(minterStatsPhaseDecoded.data.mintCount.toString()).to.equal('1');

      console.log('Allowlist mint verified successfully');
    } catch (error) {
      console.error('Error in allowlist minting:', error);
      if (error.logs) {
        console.error('Full error logs:');
        error.logs.forEach((log, index) => {
          console.error(`${index + 1}: ${log}`);
        });
      }
      throw error;
    }
  });
});
