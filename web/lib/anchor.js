/**
 * Solana client plumbing for Ballot Guardian.
 *
 * After deploying the native programs to devnet:
 * 1. Fill in the PROGRAM_IDS below with the deployed addresses.
 * 2. Build transaction instructions using @solana/web3.js TransactionInstruction.
 */

import { Connection, PublicKey, TransactionInstruction } from "@solana/web3.js";

/**
 * Program IDs -- replace with deployed addresses after deployment.
 */
export const PROGRAM_IDS = {
  quadraticVoting: null, // e.g. "ABcd...1234"
  reputationEngine: null,
  realmsAdapter: null,
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
