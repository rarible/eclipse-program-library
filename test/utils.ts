import {
  Connection,
  clusterApiUrl,
  PublicKey,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import { LibreplexEditions } from '../target/types/libreplex_editions';
import { Idl, IdlAccounts, IdlTypes, AnchorError } from '@coral-xyz/anchor';
import { BorshCoder, Program } from '@coral-xyz/anchor';
import { LibreplexEditionsControls } from '../target/types/libreplex_editions_controls';
export type EditionsDeployment =
  IdlAccounts<LibreplexEditions>['editionsDeployment'];

export async function getCluster(connection: Connection): Promise<string> {
  // Get the genesis hash
  const genesisHash = await connection.getGenesisHash();

  // Compare the genesis hash with known cluster genesis hashes
  switch (genesisHash) {
    case '5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d':
      return 'mainnet-beta';
    case 'EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG':
      return 'testnet';
    case '4uhcVJyU9pJkvQyS88uRDiswHXSCkY3zQawwpjk2NsNY':
      return 'devnet';
    default:
      // If it doesn't match any known cluster, it's likely localhost
      return 'localhost';
  }
}

export type EditionsControls =
  IdlAccounts<LibreplexEditionsControls>['editionsControls'];

export const getBase64FromDatabytes = (dataBytes: Buffer, dataType: string) => {
  console.log({ dataBytes });
  const base = dataBytes.toString('base64');
  return `data:${dataType};base64,${base}`;
};

export const decodeEditions =
  (program: Program<LibreplexEditions>) =>
  (buffer: Buffer | undefined, pubkey: PublicKey) => {
    const coder = new BorshCoder(program.idl);
    const data = buffer
      ? coder.accounts.decode<EditionsDeployment>('editionsDeployment', buffer)
      : null;

    return {
      data,
      pubkey,
    };
  };

export const decodeEditionsControls =
  (program: Program<LibreplexEditionsControls>) =>
  (buffer: Buffer | undefined, pubkey: PublicKey) => {
    const coder = new BorshCoder(program.idl);
    const data = buffer
      ? coder.accounts.decode<EditionsControls>('editionsControls', buffer)
      : null;

    return {
      data,
      pubkey,
    };
  };

export const getEditions = async (
  connection: Connection,
  editionsPda: PublicKey,
  editionsProgram: Program<LibreplexEditions>
) => {
  const editionsAccountInfo = await connection.getAccountInfo(editionsPda);
  if (!editionsAccountInfo) {
    throw new Error('Editions account not found');
  }
  const editionsDecoded = decodeEditions(editionsProgram)(
    editionsAccountInfo.data,
    editionsPda
  );
  return editionsDecoded;
};

export const getEditionsControls = async (
  connection: Connection,
  editionsControlsPda: PublicKey,
  editionsControlsProgram: Program<LibreplexEditionsControls>
) => {
  const editionsControlsAccountInfo = await connection.getAccountInfo(
    editionsControlsPda
  );
  if (!editionsControlsAccountInfo) {
    throw new Error(
      'EditionsControls account not found. The collection was not initialized with controls.'
    );
  }
  const editionsControlsDecoded = decodeEditionsControls(
    editionsControlsProgram
  )(editionsControlsAccountInfo.data, editionsControlsPda);
  return editionsControlsDecoded;
};

export const logEditions = (editionsDecoded: {
  data: EditionsDeployment;
  pubkey: PublicKey;
}) => {
  console.log({
    Editions: {
      symbol: editionsDecoded.data.symbol,
      creator: editionsDecoded.data.creator.toBase58(),
      groupMint: editionsDecoded.data.groupMint.toBase58(),
      maxNumberOfTokens: editionsDecoded.data.maxNumberOfTokens.toString(),
      cosignerProgramId: editionsDecoded.data.cosignerProgramId
        ? editionsDecoded.data.cosignerProgramId.toBase58()
        : null,
      collectionName: editionsDecoded.data.collectionName,
      collectionUri: editionsDecoded.data.collectionUri,
      tokensMinted: editionsDecoded.data.numberOfTokensIssued.toString(),
      itemBaseName: editionsDecoded.data.itemBaseName,
      itemBaseUri: editionsDecoded.data.itemBaseUri,
      itemNameIsTemplate: editionsDecoded.data.itemNameIsTemplate,
      itemUriIsTemplate: editionsDecoded.data.itemUriIsTemplate,
    },
  });
};

export const logEditionsControls = (editionsControlsDecoded: {
  data: EditionsControls;
  pubkey: PublicKey;
}) => {
  console.log({
    EditionsControls: {
      address: editionsControlsDecoded.pubkey.toBase58(),
      coreDeployment:
        editionsControlsDecoded.data.editionsDeployment.toBase58(),
      creator: editionsControlsDecoded.data.creator.toBase58(),
      treasury: editionsControlsDecoded.data.treasury.toBase58(),
      maxMintsPerWallet: Number(editionsControlsDecoded.data.maxMintsPerWallet),
    },
    phases: editionsControlsDecoded.data.phases.map((item, idx) => ({
      phaseIndex: idx,
      currentMints: Number(item.currentMints),
      maxMintsPerWallet: Number(item.maxMintsPerWallet),
      maxMintsTotal: Number(item.maxMintsTotal),
      startTime: Number(item.startTime),
      endTime: Number(item.endTime),
      priceAmount: Number(item.priceAmount),
      priceToken: item.priceToken ? item.priceToken.toBase58() : null,
      merkleRoot: item.merkleRoot ? JSON.stringify(item.merkleRoot) : null,
    })),
  });
};

export async function ensureAccountHasSol(
  connection: Connection,
  account: PublicKey,
  minBalance: number
) {
  let balance = await connection.getBalance(account);
  console.log(
    `Initial balance of ${account.toBase58()}: ${
      balance / LAMPORTS_PER_SOL
    } SOL`
  );

  if (balance < minBalance) {
    const airdropAmount = minBalance - balance;
    const signature = await connection.requestAirdrop(account, airdropAmount);
    await connection.confirmTransaction(signature, 'confirmed');
    balance = await connection.getBalance(account);
    console.log(`New balance after airdrop: ${balance / LAMPORTS_PER_SOL} SOL`);
  }
}

export function createErrorHandler(...idls: Idl[]) {
  return function handleError(error: unknown): void {
    console.error('Error occurred:', error);

    if (error instanceof AnchorError) {
      const errorCode = error.error.errorCode.number;
      for (const idl of idls) {
        const errorMessage = getErrorMessage(errorCode, idl);
        if (errorMessage !== `Unknown error: ${errorCode}`) {
          console.error('Error code:', errorCode);
          console.error('Error message:', errorMessage);
          console.error('Error name:', error.error.errorCode.code);
          console.error('Program ID:', error.program.programId.toBase58());
          return;
        }
      }
      console.error(`Unknown error code: ${errorCode}`);
    } else if (error instanceof Error) {
      console.error('Error message:', error.message);
      console.error('Stack trace:', error.stack);
    } else {
      console.error('Unexpected error:', error);
    }
  }
}

function getErrorMessage(code: number, idl: Idl): string {
  const idlErrors = idl.errors ?? [];
  const error = idlErrors.find((e) => e.code === code);
  return error ? error.msg : `Unknown error: ${code}`;
}
