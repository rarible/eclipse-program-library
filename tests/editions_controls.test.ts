import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { TOKEN_2022_PROGRAM_ID } from '@solana/spl-token';
import { LibreplexEditionsControls } from '../target/types/libreplex_editions_controls';
import { LibreplexEditions } from '../target/types/libreplex_editions';
import { expect } from 'chai';
import { describe, it } from 'mocha';

describe('Editions Controls', () => {
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

  const payer = provider.wallet as anchor.Wallet;
  const creator1 = Keypair.generate();
  const creator2 = Keypair.generate();
  const treasury = Keypair.generate();

  it('should deploy a collection, add a phase, and execute a mint', async () => {
    const collectionConfig = {
      maxNumberOfTokens: new anchor.BN(100),
      symbol: 'TEST',
      name: 'Test Collection',
      offchainUrl: 'https://example.com/metadata.json',
      treasury: treasury.publicKey,
      maxMintsPerWallet: new anchor.BN(5),
      royaltyBasisPoints: new anchor.BN(1000),
      creators: [
        {
          address: creator1.publicKey,
          share: 50,
        },
        // {
        //   address: creator2.publicKey,
        //   share: 50,
        // },
      ],
      extraMeta: [
        { field: 'f1', value: 'v1' },
        // { field: 'f2', value: 'v2' },
      ],
      phases: [
        {
          priceAmount: new anchor.BN(1000000), // 0.001 SOL
          priceToken: new PublicKey(
            'So11111111111111111111111111111111111111112'
          ),
          startTime: new anchor.BN(Math.floor(Date.now() / 1000)),
          maxMintsPerWallet: new anchor.BN(5),
          maxMintsTotal: new anchor.BN(50),
          endTime: new anchor.BN(Math.floor(Date.now() / 1000) + 3600), // 1 hour from now
        },
      ],
    };

    // 1. Deploy a collection
    const [editions] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('editions_deployment'),
        Buffer.from(collectionConfig.symbol),
      ],
      editionsProgram.programId
    );

    const [editionsControls] = PublicKey.findProgramAddressSync(
      [Buffer.from('editions_controls'), editions.toBuffer()],
      editionsControlsProgram.programId
    );

    const [hashlist] = PublicKey.findProgramAddressSync(
      [Buffer.from('hashlist'), editions.toBuffer()],
      editionsProgram.programId
    );

    const groupMint = Keypair.generate();
    const group = Keypair.generate();

    console.log('Initializing accounts...');
    console.log('Editions address:', editions.toBase58());
    console.log('EditionsControls address:', editionsControls.toBase58());
    console.log('Hashlist address:', hashlist.toBase58());
    console.log('GroupMint address:', groupMint.publicKey.toBase58());
    console.log('Group address:', group.publicKey.toBase58());

    console.log('initialiseEditionsControls...');
    try {
      await editionsControlsProgram.methods
        .initialiseEditionsControls({
          maxNumberOfTokens: collectionConfig.maxNumberOfTokens,
          symbol: collectionConfig.symbol,
          name: collectionConfig.name,
          offchainUrl: collectionConfig.offchainUrl,
          cosignerProgramId: null,
          treasury: collectionConfig.treasury,
          maxMintsPerWallet: collectionConfig.maxMintsPerWallet,
          royaltyBasisPoints: collectionConfig.royaltyBasisPoints,
          creators: collectionConfig.creators,
          extraMeta: collectionConfig.extraMeta,
        })
        .accountsStrict({
          editionsControls,
          editionsDeployment: editions,
          hashlist,
          payer: payer.publicKey,
          creator: payer.publicKey,
          groupMint: groupMint.publicKey,
          group: group.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          libreplexEditionsProgram: editionsProgram.programId,
          groupExtensionProgram: new PublicKey(
            '2TBWcwXdtwQEN8gXoEg6xFtUCYcBRpQaunWTDJwDp5Tx'
          ),
        })
        .signers([groupMint, group])
        .rpc();
      console.log('initialiseEditionsControls done');
    } catch (error) {
      console.error('Error in initialiseEditionsControls:', error);
      console.log('Accounts used and their executability:');

      const accountsToCheck = {
        editionsControls,
        editionsDeployment: editions,
        hashlist,
        payer: payer.publicKey,
        creator: payer.publicKey,
        groupMint: groupMint.publicKey,
        group: group.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        libreplexEditionsProgram: editionsProgram.programId,
        groupExtensionProgram: new PublicKey(
          '2TBWcwXdtwQEN8gXoEg6xFtUCYcBRpQaunWTDJwDp5Tx'
        ),
      };

      for (const [name, pubkey] of Object.entries(accountsToCheck)) {
        try {
          const accountInfo = await provider.connection.getAccountInfo(pubkey);
          console.log(`${name}:`, pubkey.toBase58());
          console.log(
            `  Executable: ${
              accountInfo ? accountInfo.executable : 'Account not found'
            }`
          );
        } catch (err) {
          console.log(`${name}:`, pubkey.toBase58());
          console.log(`  Error fetching account info: ${err.message}`);
        }
      }

      throw error;
    }

    //     // 2. Add a phase
    //     console.log('addPhase...');
    //     await editionsControlsProgram.methods
    //       .addPhase({
    //         priceAmount: new anchor.BN(1000000), // 0.001 SOL
    //         priceToken: new PublicKey(
    //           'So11111111111111111111111111111111111111112'
    //         ),
    //         startTime: new anchor.BN(Math.floor(Date.now() / 1000)),
    //         maxMintsPerWallet: new anchor.BN(5),
    //         maxMintsTotal: new anchor.BN(50),
    //         endTime: new anchor.BN(Math.floor(Date.now() / 1000) + 3600), // 1 hour from now
    //       })
    //       .accounts({
    //         editionsControls,
    //         creator: payer.publicKey,
    //         payer: payer.publicKey,
    //         systemProgram: SystemProgram.programId,
    //         tokenProgram: TOKEN_2022_PROGRAM_ID,
    //         libreplexEditionsProgram: editionsProgram.programId,
    //       })
    //       .rpc();
    //     console.log('addPhase done');

    //     // 3. Execute a mint
    //     const mint = Keypair.generate();
    //     const member = Keypair.generate();

    //     const [hashlistMarker] = PublicKey.findProgramAddressSync(
    //       [
    //         Buffer.from('hashlist_marker'),
    //         editionsDeployment.toBuffer(),
    //         mint.publicKey.toBuffer(),
    //       ],
    //       editionsProgram.programId
    //     );

    //     const [minterStats] = PublicKey.findProgramAddressSync(
    //       [
    //         Buffer.from('minter_stats'),
    //         editionsDeployment.toBuffer(),
    //         payer.publicKey.toBuffer(),
    //       ],
    //       editionsControlsProgram.programId
    //     );

    //     const [minterStatsPhase] = PublicKey.findProgramAddressSync(
    //       [
    //         Buffer.from('minter_stats_phase'),
    //         editionsDeployment.toBuffer(),
    //         payer.publicKey.toBuffer(),
    //         Buffer.from([0]),
    //       ],
    //       editionsControlsProgram.programId
    //     );

    //     console.log('mintWithControls...');
    //     await editionsControlsProgram.methods
    //       .mintWithControls({
    //         phaseIndex: 0,
    //       })
    //       .accounts({
    //         editionsDeployment,
    //         editionsControls,
    //         hashlist,
    //         hashlistMarker,
    //         payer: payer.publicKey,
    //         mint: mint.publicKey,
    //         member: member.publicKey,
    //         signer: payer.publicKey,
    //         minter: payer.publicKey,
    //         minterStats,
    //         minterStatsPhase,
    //         group: group.publicKey,
    //         groupExtensionProgram: new PublicKey('GExfnHgvdPcg7uQh9vHJYKdNbpGfUzb'),
    //         tokenAccount: payer.publicKey, // This should be the correct associated token account
    //         treasury: payer.publicKey,
    //         systemProgram: SystemProgram.programId,
    //         tokenProgram: TOKEN_2022_PROGRAM_ID,
    //         libreplexEditionsProgram: editionsProgram.programId,
    //       })
    //       .signers([mint, member])
    //       .rpc();
    //     console.log('mintWithControls done');

    //     // Add assertions here to verify the mint was successful
    //     const editionsControlsAccount =
    //       await editionsControlsProgram.account.editionsControls.fetch(
    //         editionsControls
    //       );
    //     expect(editionsControlsAccount.phases[0].currentMints.toNumber()).to.equal(
    //       1
    //     );
  });
});
