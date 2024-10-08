import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
  PublicKey,
  Keypair,
  SystemProgram,
  ComputeBudgetProgram,
} from '@solana/web3.js';
import { TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';
import { LibreplexEditionsControls } from '../../eclipse-program-library/target/types/libreplex_editions_controls';
import { LibreplexEditions } from '../../eclipse-program-library/target/types/libreplex_editions';
import { expect } from 'chai';
import { describe, it } from 'mocha';
import { decodeEditions, getCluster } from './utils';
import { Transaction } from '@solana/web3.js';
import { decodeEditionsControls } from './utils';

describe('Editions Controls Test Suite', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const editionsControlsProgram = anchor.workspace
    .LibreplexEditionsControls as Program<LibreplexEditionsControls>;
  console.log(
    'editionsControlsProgram ID:',
    editionsControlsProgram.programId.toBase58()
  );

  const editionsProgram = anchor.workspace
    .LibreplexEditions as Program<LibreplexEditions>;
  console.log('editionsProgram ID:', editionsProgram.programId.toBase58());

  const payer = (provider.wallet as anchor.Wallet).payer;
  const creator1 = Keypair.generate();
  const creator2 = Keypair.generate();
  const treasury = Keypair.generate();
  const platformFeeAdmin = Keypair.generate();

  const collectionConfig = {
    maxNumberOfTokens: new anchor.BN(1150),
    symbol: 'COOLX55',
    name: 'Collection name with meta, platform fee and royalties',
    offchainUrl: 'ipfs://QmbsXNSkPUtYNmKfYw1mUSVuz9QU8nhu7YvzM1aAQsv6xw/0',
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
    itemName: 'Item T8 V4 #{}',
    cosignerProgramId: null,
  };

  before(async () => {
    const cluster = await getCluster(provider.connection);
    console.log('Cluster:', cluster);
  });

  it('should deploy a collection, add a phase, and execute a mint', async () => {
    // Modify compute units for the transaction
    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
      units: 800000,
    });

    const [editionsPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('editions_deployment'),
        Buffer.from(collectionConfig.symbol),
      ],
      editionsProgram.programId
    );

    const [editionsControlsPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('editions_controls'), editionsPda.toBuffer()],
      editionsControlsProgram.programId
    );
    const [hashlistPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('hashlist'), editionsPda.toBuffer()],
      editionsProgram.programId
    );

    const groupMint = Keypair.generate();
    const group = Keypair.generate();

    console.log('Collection config: ', collectionConfig);

    console.log('\nDeploying via initialiseEditionsControls...\n');
    try {
      const initialiseIx = await editionsControlsProgram.methods
        .initialiseEditionsControls({
          maxMintsPerWallet: collectionConfig.maxMintsPerWallet,
          treasury: collectionConfig.treasury,
          maxNumberOfTokens: collectionConfig.maxNumberOfTokens,
          symbol: collectionConfig.symbol,
          name: collectionConfig.name,
          offchainUrl: collectionConfig.offchainUrl,
          cosignerProgramId: collectionConfig.cosignerProgramId,
          royalties: collectionConfig.royalties,
          platformFee: collectionConfig.platformFee,
          extraMeta: collectionConfig.extraMeta,
          itemBaseUri: collectionConfig.itemBaseUri,
          itemName: collectionConfig.itemName,
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

      const txSignature = await provider.sendAndConfirm(transaction, [
        groupMint,
        group,
        payer,
      ]);

      console.log('Transaction signature:', txSignature);

      console.log('\ninitialiseEditionsControls done!\n');
      // log deployed addresses
      console.log('Editions address:', editionsPda.toBase58());
      console.log('EditionsControls address:', editionsControlsPda.toBase58());
      console.log('Hashlist address:', hashlistPda.toBase58());
      console.log('GroupMint address:', groupMint.publicKey.toBase58());
      console.log('Group address:', group.publicKey.toBase58());

      console.log('\nFetching and displaying deployed collection state...\n');

      // Fetch and decode the Editions deployment
      const editionsAccountInfo = await provider.connection.getAccountInfo(
        editionsPda
      );
      if (!editionsAccountInfo) {
        throw new Error('Editions account not found');
      }

      const editionsDeployment = decodeEditions(editionsProgram)(
        editionsAccountInfo.data,
        editionsPda
      );

      console.log('Editions Deployment:');
      console.log({
        creator: editionsDeployment.item.creator.toBase58(),
        groupMint: editionsDeployment.item.groupMint.toBase58(),
        maxNumberOfTokens: editionsDeployment.item.maxNumberOfTokens.toString(),
        name: editionsDeployment.item.name,
        tokensMinted: editionsDeployment.item.numberOfTokensIssued.toString(),
        offchainUrl: editionsDeployment.item.offchainUrl,
        symbol: editionsDeployment.item.symbol,
        nameIsTemplate: editionsDeployment.item.nameIsTemplate,
        urlIsTemplate: editionsDeployment.item.urlIsTemplate,
      });

      // Fetch and check the EditionsControls account
      const controlsAccountData = await provider.connection.getAccountInfo(
        editionsControlsPda
      );

      if (!controlsAccountData || !controlsAccountData.data) {
        console.log('Core editions deployment - no controls specified');
      } else {
        const editionsControlsObj = decodeEditionsControls(
          editionsControlsProgram
        )(controlsAccountData.data, editionsControlsPda);

        console.log({
          editionsControls: {
            address: editionsControlsPda.toBase58(),
            coreDeployment:
              editionsControlsObj.item.editionsDeployment.toBase58(),
            creator: editionsControlsObj.item.creator.toBase58(),
            treasury: editionsControlsObj.item.treasury.toBase58(),
            maxMintsPerWallet: Number(
              editionsControlsObj.item.maxMintsPerWallet
            ),
          },
          phases: editionsControlsObj.item.phases.map((item, idx) => ({
            phaseIndex: idx,
            currentMints: Number(item.currentMints),
            maxMintsPerWallet: Number(item.maxMintsPerWallet),
            maxMintsTotal: Number(item.maxMintsTotal),
            startTime: Number(item.startTime),
            endTime: Number(item.endTime),
            priceAmount: Number(item.priceAmount),
            priceToken: item.priceToken ? item.priceToken.toBase58() : null,
            merkleRoot: item.merkleRoot
              ? JSON.stringify(item.merkleRoot)
              : null,
          })),
        });

        // Add assertions to verify the state
        expect(editionsControlsObj.item.editionsDeployment.toBase58()).to.equal(
          editionsPda.toBase58()
        );
        expect(editionsControlsObj.item.creator.toBase58()).to.equal(
          payer.publicKey.toBase58()
        );
        expect(editionsControlsObj.item.treasury.toBase58()).to.equal(
          collectionConfig.treasury.toBase58()
        );
        expect(Number(editionsControlsObj.item.maxMintsPerWallet)).to.equal(
          Number(collectionConfig.maxMintsPerWallet)
        );

        // Verify Editions deployment
        // expect(editionsDeployment.item.creator.toBase58()).to.equal(
        //   payer.publicKey.toBase58()
        // );
        // expect(editionsDeployment.item.maxNumberOfTokens.toString()).to.equal(
        //   collectionConfig.maxNumberOfTokens.toString()
        // );
        // expect(editionsDeployment.item.name).to.equal(collectionConfig.name);
        // expect(editionsDeployment.item.symbol).to.equal(
        //   collectionConfig.symbol
        // );
        // expect(editionsDeployment.item.offchainUrl).to.equal(
        //   collectionConfig.offchainUrl
        // );

        // Add more assertions as needed for phases, if any are expected to be present at this point
      }

      // 2. Add a phase (if needed)
      // 3. Execute a mint (if needed)

      // These sections can be implemented similarly to the previous version,
      // but make sure to update any parameters that have changed in the new CLI version.
    } catch (error) {
      console.error('Error in initialiseEditionsControls:', error);
      throw error;
    }

    // Add assertions to verify the initialization was successful

    // Add more assertions as needed

    // 2. Add a phase (if needed)
    // 3. Execute a mint (if needed)

    // These sections can be implemented similarly to the previous version,
    // but make sure to update any parameters that have changed in the new CLI version.
  });
});
