/**
 * Solana client plumbing for Ballot Guardian.
 *
 * Programs are deployed to Solana devnet. Build transaction instructions
 * using @solana/web3.js TransactionInstruction via the helpers below.
 */

import { Connection, PublicKey, TransactionInstruction } from "@solana/web3.js";

/**
 * Program IDs -- deployed to Solana devnet.
 */
export const PROGRAM_IDS = {
  quadraticVoting: "346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp",
  reputationEngine: "8JpbKjoR4c7n2HqS51WjyjJrLwvVgGGsKN4o2boohdEA",
  realmsAdapter: "E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky",
};

/**
 * Creates a connection with confirmed commitment.
 *
 * @param {string} endpoint - RPC endpoint URL
 * @returns {Connection}
 */
export function getConnection(endpoint) {
  return new Connection(endpoint, "confirmed");
}

/**
 * Helper to build a TransactionInstruction for a native program.
 *
 * @param {string} programIdStr - The program ID as a base58 string
 * @param {Buffer} data - Borsh-serialized instruction data
 * @param {Array<{pubkey: PublicKey, isSigner: boolean, isWritable: boolean}>} keys - Account metas
 * @returns {TransactionInstruction}
 */
export function buildInstruction(programIdStr, data, keys) {
  if (!programIdStr) {
    throw new Error(
      "Program not yet deployed. Set the program ID in PROGRAM_IDS.",
    );
  }
  return new TransactionInstruction({
    programId: new PublicKey(programIdStr),
    keys,
    data,
  });
}
