/**
 * Solana client plumbing for Ballot Guardian.
 *
 * Re-exports program IDs and all instruction builders / account readers
 * from programClient.js. Kept for backwards-compatibility with any
 * imports that reference this file.
 */

import { Connection } from "@solana/web3.js";

export {
  REALMS_ADAPTER_PROGRAM_ID,
  QUADRATIC_VOTING_PROGRAM_ID,
  REPUTATION_ENGINE_PROGRAM_ID,
} from "./programClient";

export const PROGRAM_IDS = {
  quadraticVoting: "346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp",
  reputationEngine: "8JpbKjoR4c7n2HqS51WjyjJrLwvVgGGsKN4o2boohdEA",
  realmsAdapter: "E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky",
};

export function getConnection(endpoint) {
  return new Connection(endpoint, "confirmed");
}

export * from "./programClient";
