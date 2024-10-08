import { Connection, clusterApiUrl, PublicKey } from '@solana/web3.js';
import { LibreplexEditions } from '../target/types/libreplex_editions';
import { IdlAccounts, IdlTypes } from '@coral-xyz/anchor';
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

export const decodeEditions =
  (program: Program<LibreplexEditions>) =>
  (buffer: Buffer | undefined, pubkey: PublicKey) => {
    const coder = new BorshCoder(program.idl);
    const liquidity = buffer
      ? coder.accounts.decode<EditionsDeployment>('editionsDeployment', buffer)
      : null;

    return {
      item: liquidity,
      pubkey,
    };
  };

export type EditionsControls =
  IdlAccounts<LibreplexEditionsControls>['editionsControls'];

export const getBase64FromDatabytes = (dataBytes: Buffer, dataType: string) => {
  console.log({ dataBytes });
  const base = dataBytes.toString('base64');
  return `data:${dataType};base64,${base}`;
};

export const decodeEditionsControls =
  (program: Program<LibreplexEditionsControls>) =>
  (buffer: Buffer | undefined, pubkey: PublicKey) => {
    const coder = new BorshCoder(program.idl);
    const liquidity = buffer
      ? coder.accounts.decode<EditionsControls>('editionsControls', buffer)
      : null;

    return {
      item: liquidity,
      pubkey,
    };
  };
