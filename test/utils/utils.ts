import { Connection } from '@solana/web3.js';

/// Verify this works
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

export const getBase64FromDatabytes = (dataBytes: Buffer, dataType: string) => {
  console.log({ dataBytes });
  const base = dataBytes.toString('base64');
  return `data:${dataType};base64,${base}`;
};
