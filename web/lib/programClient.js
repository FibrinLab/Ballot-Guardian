/**
 * Ballot Guardian — Solana Program Client
 *
 * Manual Borsh serialization, PDA derivation, instruction builders,
 * account deserializers, and high-level action functions for:
 *   - Realms Adapter
 *   - Quadratic Voting
 *   - Reputation Engine
 *
 * Zero additional npm dependencies — uses DataView for integer encoding.
 */

import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";

// ---------------------------------------------------------------------------
// 1a. Constants
// ---------------------------------------------------------------------------

export const REALMS_ADAPTER_PROGRAM_ID = new PublicKey(
  "E5CHyQY6gsxWB4cdTCSMxS3aY3J4eCVCXEe1KVTfk4Ky",
);
export const QUADRATIC_VOTING_PROGRAM_ID = new PublicKey(
  "346RNEQcBBff4skHhQiRCPe9cDeWpaPTsT2TpQUFYomp",
);
export const REPUTATION_ENGINE_PROGRAM_ID = new PublicKey(
  "8JpbKjoR4c7n2HqS51WjyjJrLwvVgGGsKN4o2boohdEA",
);

// ---------------------------------------------------------------------------
// 1b. Borsh helpers (manual, no deps)
// ---------------------------------------------------------------------------

function allocBuffer(size) {
  const buf = new ArrayBuffer(size);
  return { u8: new Uint8Array(buf), view: new DataView(buf) };
}

function writeU8(buf, offset, value) {
  buf[offset] = value & 0xff;
  return offset + 1;
}

function writeU16LE(buf, offset, value) {
  const dv = new DataView(buf.buffer, buf.byteOffset);
  dv.setUint16(offset, value, true);
  return offset + 2;
}

function writeU32LE(buf, offset, value) {
  const dv = new DataView(buf.buffer, buf.byteOffset);
  dv.setUint32(offset, value, true);
  return offset + 4;
}

function writeU64LE(buf, offset, value) {
  const dv = new DataView(buf.buffer, buf.byteOffset);
  const big = BigInt(value);
  dv.setBigUint64(offset, big, true);
  return offset + 8;
}

function writeI64LE(buf, offset, value) {
  const dv = new DataView(buf.buffer, buf.byteOffset);
  dv.setBigInt64(offset, BigInt(value), true);
  return offset + 8;
}

function writePubkey(buf, offset, pubkey) {
  const bytes = pubkey instanceof PublicKey ? pubkey.toBytes() : pubkey;
  buf.set(bytes, offset);
  return offset + 32;
}

function writeBool(buf, offset, value) {
  buf[offset] = value ? 1 : 0;
  return offset + 1;
}

function writeOptionPubkey(buf, offset, pubkeyOrNull) {
  if (pubkeyOrNull) {
    offset = writeU8(buf, offset, 1);
    offset = writePubkey(buf, offset, pubkeyOrNull);
  } else {
    offset = writeU8(buf, offset, 0);
  }
  return offset;
}

function writeOptionU64(buf, offset, valueOrNull) {
  if (valueOrNull != null) {
    offset = writeU8(buf, offset, 1);
    offset = writeU64LE(buf, offset, valueOrNull);
  } else {
    offset = writeU8(buf, offset, 0);
  }
  return offset;
}

function writeOptionU8(buf, offset, valueOrNull) {
  if (valueOrNull != null) {
    offset = writeU8(buf, offset, 1);
    offset = writeU8(buf, offset, valueOrNull);
  } else {
    offset = writeU8(buf, offset, 0);
  }
  return offset;
}

// Read helpers for deserialization
function readU8(data, offset) {
  return { value: data[offset], next: offset + 1 };
}

function readU16LE(data, offset) {
  const dv = new DataView(data.buffer, data.byteOffset);
  return { value: dv.getUint16(offset, true), next: offset + 2 };
}

function readU32LE(data, offset) {
  const dv = new DataView(data.buffer, data.byteOffset);
  return { value: dv.getUint32(offset, true), next: offset + 4 };
}

function readU64LE(data, offset) {
  const dv = new DataView(data.buffer, data.byteOffset);
  return { value: dv.getBigUint64(offset, true), next: offset + 8 };
}

function readI64LE(data, offset) {
  const dv = new DataView(data.buffer, data.byteOffset);
  return { value: dv.getBigInt64(offset, true), next: offset + 8 };
}

function readU128LE(data, offset) {
  const dv = new DataView(data.buffer, data.byteOffset);
  const lo = dv.getBigUint64(offset, true);
  const hi = dv.getBigUint64(offset + 8, true);
  return { value: (hi << 64n) | lo, next: offset + 16 };
}

function readPubkey(data, offset) {
  const bytes = data.slice(offset, offset + 32);
  return { value: new PublicKey(bytes), next: offset + 32 };
}

function readBool(data, offset) {
  return { value: data[offset] !== 0, next: offset + 1 };
}

function readOptionU64(data, offset) {
  if (data[offset] === 0) return { value: null, next: offset + 1 };
  const dv = new DataView(data.buffer, data.byteOffset);
  return { value: dv.getBigUint64(offset + 1, true), next: offset + 9 };
}

function readOptionU8(data, offset) {
  if (data[offset] === 0) return { value: null, next: offset + 1 };
  return { value: data[offset + 1], next: offset + 2 };
}

function readOptionPubkey(data, offset) {
  if (data[offset] === 0) return { value: null, next: offset + 1 };
  const bytes = data.slice(offset + 1, offset + 33);
  return { value: new PublicKey(bytes), next: offset + 33 };
}

function readBytes(data, offset, len) {
  return { value: data.slice(offset, offset + len), next: offset + len };
}

// ---------------------------------------------------------------------------
// 1c. PDA derivation
// ---------------------------------------------------------------------------

export function findAdapterConfigPDA(realm) {
  const realmKey = realm instanceof PublicKey ? realm : new PublicKey(realm);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("adapter-config"), realmKey.toBuffer()],
    REALMS_ADAPTER_PROGRAM_ID,
  );
}

export function findProposalBindingPDA(realm, proposal) {
  const realmKey = realm instanceof PublicKey ? realm : new PublicKey(realm);
  const propKey = proposal instanceof PublicKey ? proposal : new PublicKey(proposal);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("proposal-binding"), realmKey.toBuffer(), propKey.toBuffer()],
    REALMS_ADAPTER_PROGRAM_ID,
  );
}

export function findVoterWeightRecordPDA(binding, voter) {
  const bindKey = binding instanceof PublicKey ? binding : new PublicKey(binding);
  const voterKey = voter instanceof PublicKey ? voter : new PublicKey(voter);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("voter-weight"), bindKey.toBuffer(), voterKey.toBuffer()],
    REALMS_ADAPTER_PROGRAM_ID,
  );
}

export function findBallotPDA(realm, proposal) {
  const realmKey = realm instanceof PublicKey ? realm : new PublicKey(realm);
  const propKey = proposal instanceof PublicKey ? proposal : new PublicKey(proposal);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("ballot"), realmKey.toBuffer(), propKey.toBuffer()],
    QUADRATIC_VOTING_PROGRAM_ID,
  );
}

export function findAllocationPDA(ballot, voter) {
  const ballotKey = ballot instanceof PublicKey ? ballot : new PublicKey(ballot);
  const voterKey = voter instanceof PublicKey ? voter : new PublicKey(voter);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("allocation"), ballotKey.toBuffer(), voterKey.toBuffer()],
    QUADRATIC_VOTING_PROGRAM_ID,
  );
}

export function findReputationConfigPDA(realm) {
  const realmKey = realm instanceof PublicKey ? realm : new PublicKey(realm);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("reputation-config"), realmKey.toBuffer()],
    REPUTATION_ENGINE_PROGRAM_ID,
  );
}

export function findReputationProfilePDA(realm, member) {
  const realmKey = realm instanceof PublicKey ? realm : new PublicKey(realm);
  const memberKey = member instanceof PublicKey ? member : new PublicKey(member);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("reputation-profile"), realmKey.toBuffer(), memberKey.toBuffer()],
    REPUTATION_ENGINE_PROGRAM_ID,
  );
}

// ---------------------------------------------------------------------------
// 1d. Instruction builders
// ---------------------------------------------------------------------------

// --- Borsh 1.x: enum variant index is u8 (1 byte) ---
// --- VoteChoice / VoterWeightAction nested enums are also u8 ---

// --- Realms Adapter ---

export function buildInitializeAdapterIx({
  admin,
  realm,
  governanceProgramId,
  quadraticVotingProgram,
  reputationEngineProgram,
  councilOverrideAuthority,
  minReputationBps,
  maxReputationBps,
}) {
  const hasAuthority = !!councilOverrideAuthority;
  const size = 1 + 32 + 32 + 32 + 32 + (hasAuthority ? 33 : 1) + 2 + 2;
  const { u8: buf } = allocBuffer(size);

  let off = 0;
  off = writeU8(buf, off, 0); // variant 0 — InitializeAdapter
  off = writePubkey(buf, off, realm);
  off = writePubkey(buf, off, governanceProgramId);
  off = writePubkey(buf, off, quadraticVotingProgram);
  off = writePubkey(buf, off, reputationEngineProgram);
  off = writeOptionPubkey(buf, off, councilOverrideAuthority || null);
  off = writeU16LE(buf, off, minReputationBps);
  writeU16LE(buf, off, maxReputationBps);

  const [configPDA] = findAdapterConfigPDA(realm);

  return new TransactionInstruction({
    programId: REALMS_ADAPTER_PROGRAM_ID,
    keys: [
      { pubkey: admin, isSigner: true, isWritable: true },
      { pubkey: configPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(buf),
  });
}

export function buildBindProposalIx({
  admin,
  realm,
  proposal,
  quadraticBallot,
  governingTokenMint,
  councilOverrideEnabled,
  defaultWeightExpirySlot,
}) {
  const size = 1 + 32 + 32 + 32 + 1 + 8;
  const { u8: buf } = allocBuffer(size);

  let off = 0;
  off = writeU8(buf, off, 1); // variant 1 — BindProposal
  off = writePubkey(buf, off, proposal);
  off = writePubkey(buf, off, quadraticBallot);
  off = writePubkey(buf, off, governingTokenMint);
  off = writeBool(buf, off, councilOverrideEnabled || false);
  writeU64LE(buf, off, defaultWeightExpirySlot || 0);

  const [configPDA] = findAdapterConfigPDA(realm);
  const [bindingPDA] = findProposalBindingPDA(realm, proposal);

  return new TransactionInstruction({
    programId: REALMS_ADAPTER_PROGRAM_ID,
    keys: [
      { pubkey: admin, isSigner: true, isWritable: true },
      { pubkey: configPDA, isSigner: false, isWritable: false },
      { pubkey: bindingPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(buf),
  });
}

export function buildCreateVWRIx({
  payer,
  realm,
  proposal,
  voter,
}) {
  const { u8: buf } = allocBuffer(1);
  writeU8(buf, 0, 3); // variant 3 — CreateVoterWeightRecord

  const [configPDA] = findAdapterConfigPDA(realm);
  const [bindingPDA] = findProposalBindingPDA(realm, proposal);
  const [recordPDA] = findVoterWeightRecordPDA(bindingPDA, voter);

  return new TransactionInstruction({
    programId: REALMS_ADAPTER_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: configPDA, isSigner: false, isWritable: false },
      { pubkey: proposal instanceof PublicKey ? proposal : new PublicKey(proposal), isSigner: false, isWritable: false },
      { pubkey: bindingPDA, isSigner: false, isWritable: false },
      { pubkey: voter instanceof PublicKey ? voter : new PublicKey(voter), isSigner: false, isWritable: false },
      { pubkey: recordPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(buf),
  });
}

export function buildRefreshVWRIx({
  caller,
  realm,
  proposal,
  voter,
  tokenAmountAllocated,
  qvVotesAllocated,
  reputationMultiplierBps,
  voterWeightExpiry,
  weightAction,
  weightActionTarget,
}) {
  const hasExpiry = voterWeightExpiry != null;
  const hasAction = weightAction != null;
  const hasTarget = !!weightActionTarget;
  // VoterWeightAction is also a Borsh enum → u8 inside Option
  const size = 1 + 8 + 4 + 2 + (hasExpiry ? 9 : 1) + (hasAction ? 2 : 1) + (hasTarget ? 33 : 1);
  const { u8: buf } = allocBuffer(size);

  let off = 0;
  off = writeU8(buf, off, 4); // variant 4 — RefreshVoterWeightRecord
  off = writeU64LE(buf, off, tokenAmountAllocated || 0);
  off = writeU32LE(buf, off, qvVotesAllocated || 0);
  off = writeU16LE(buf, off, reputationMultiplierBps || 10000);
  off = writeOptionU64(buf, off, hasExpiry ? voterWeightExpiry : null);
  // Option<VoterWeightAction> — the enum value is u8
  if (hasAction) {
    off = writeU8(buf, off, 1);
    off = writeU8(buf, off, weightAction);
  } else {
    off = writeU8(buf, off, 0);
  }
  writeOptionPubkey(buf, off, hasTarget ? weightActionTarget : null);

  const [configPDA] = findAdapterConfigPDA(realm);
  const [bindingPDA] = findProposalBindingPDA(realm, proposal);
  const [recordPDA] = findVoterWeightRecordPDA(bindingPDA, voter);

  return new TransactionInstruction({
    programId: REALMS_ADAPTER_PROGRAM_ID,
    keys: [
      { pubkey: caller, isSigner: true, isWritable: false },
      { pubkey: configPDA, isSigner: false, isWritable: false },
      { pubkey: proposal instanceof PublicKey ? proposal : new PublicKey(proposal), isSigner: false, isWritable: false },
      { pubkey: bindingPDA, isSigner: false, isWritable: true },
      { pubkey: voter instanceof PublicKey ? voter : new PublicKey(voter), isSigner: false, isWritable: false },
      { pubkey: recordPDA, isSigner: false, isWritable: true },
    ],
    data: Buffer.from(buf),
  });
}

// --- Quadratic Voting ---

export function buildInitializeBallotIx({
  authority,
  realm,
  proposal,
  minReputationBps,
  maxReputationBps,
  votingStartsAt,
  votingEndsAt,
}) {
  const size = 1 + 32 + 32 + 2 + 2 + 8 + 8;
  const { u8: buf } = allocBuffer(size);

  let off = 0;
  off = writeU8(buf, off, 0); // variant 0 — InitializeBallot
  off = writePubkey(buf, off, realm);
  off = writePubkey(buf, off, proposal);
  off = writeU16LE(buf, off, minReputationBps || 0);
  off = writeU16LE(buf, off, maxReputationBps || 20000);
  off = writeI64LE(buf, off, votingStartsAt);
  writeI64LE(buf, off, votingEndsAt);

  const [ballotPDA] = findBallotPDA(realm, proposal);

  return new TransactionInstruction({
    programId: QUADRATIC_VOTING_PROGRAM_ID,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: ballotPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(buf),
  });
}

export function buildRegisterVoterIx({
  authority,
  realm,
  proposal,
  voter,
  creditsBudget,
  reputationMultiplierBps,
}) {
  const size = 1 + 8 + 2;
  const { u8: buf } = allocBuffer(size);

  let off = 0;
  off = writeU8(buf, off, 1); // variant 1 — RegisterVoter
  off = writeU64LE(buf, off, creditsBudget || 100);
  writeU16LE(buf, off, reputationMultiplierBps || 10000);

  const [ballotPDA] = findBallotPDA(realm, proposal);
  const [allocPDA] = findAllocationPDA(ballotPDA, voter);

  return new TransactionInstruction({
    programId: QUADRATIC_VOTING_PROGRAM_ID,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: ballotPDA, isSigner: false, isWritable: true },
      { pubkey: voter instanceof PublicKey ? voter : new PublicKey(voter), isSigner: false, isWritable: false },
      { pubkey: allocPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(buf),
  });
}

export function buildCastVoteIx({
  voter,
  realm,
  proposal,
  choice,
  additionalVotes,
}) {
  // CastVote { choice: VoteChoice, additional_votes: u32 }
  // VoteChoice is a Borsh enum with #[repr(u8)] → u8
  const size = 1 + 1 + 4;
  const { u8: buf } = allocBuffer(size);

  let off = 0;
  off = writeU8(buf, off, 3); // variant 3 — CastVote
  off = writeU8(buf, off, choice); // VoteChoice enum: 0=Yes, 1=No, 2=Abstain
  writeU32LE(buf, off, additionalVotes || 0);

  const [ballotPDA] = findBallotPDA(realm, proposal);
  const [allocPDA] = findAllocationPDA(ballotPDA, voter);

  return new TransactionInstruction({
    programId: QUADRATIC_VOTING_PROGRAM_ID,
    keys: [
      { pubkey: voter, isSigner: true, isWritable: false },
      { pubkey: ballotPDA, isSigner: false, isWritable: true },
      { pubkey: allocPDA, isSigner: false, isWritable: true },
    ],
    data: Buffer.from(buf),
  });
}

export function buildFinalizeBallotIx({ authority, realm, proposal }) {
  const { u8: buf } = allocBuffer(1);
  writeU8(buf, 0, 4); // variant 4 — FinalizeBallot

  const [ballotPDA] = findBallotPDA(realm, proposal);

  return new TransactionInstruction({
    programId: QUADRATIC_VOTING_PROGRAM_ID,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: ballotPDA, isSigner: false, isWritable: true },
    ],
    data: Buffer.from(buf),
  });
}

// --- Reputation Engine ---

export function buildInitRealmConfigIx({
  admin,
  realm,
  oracleAuthority,
  minMultiplierBps,
  baseMultiplierBps,
  maxMultiplierBps,
  participationWeight,
  proposalWeight,
  stakingWeight,
  tenureWeight,
  delegationWeight,
  pointsPerBonusBps,
  penaltyUnitBps,
  maxBonusBps,
  maxPenaltyBps,
}) {
  const hasOracle = !!oracleAuthority;
  const size = 1 + 32 + (hasOracle ? 33 : 1) + 2 + 2 + 2 + (5 * 2) + 4 + 2 + 2 + 2;
  const { u8: buf } = allocBuffer(size);

  let off = 0;
  off = writeU8(buf, off, 0); // variant 0 — InitializeRealmConfig
  off = writePubkey(buf, off, realm);
  off = writeOptionPubkey(buf, off, oracleAuthority || null);
  off = writeU16LE(buf, off, minMultiplierBps || 5000);
  off = writeU16LE(buf, off, baseMultiplierBps || 10000);
  off = writeU16LE(buf, off, maxMultiplierBps || 20000);
  off = writeU16LE(buf, off, participationWeight || 3000);
  off = writeU16LE(buf, off, proposalWeight || 2000);
  off = writeU16LE(buf, off, stakingWeight || 2000);
  off = writeU16LE(buf, off, tenureWeight || 1500);
  off = writeU16LE(buf, off, delegationWeight || 1500);
  off = writeU32LE(buf, off, pointsPerBonusBps || 100);
  off = writeU16LE(buf, off, penaltyUnitBps || 500);
  off = writeU16LE(buf, off, maxBonusBps || 5000);
  writeU16LE(buf, off, maxPenaltyBps || 5000);

  const [configPDA] = findReputationConfigPDA(realm);

  return new TransactionInstruction({
    programId: REPUTATION_ENGINE_PROGRAM_ID,
    keys: [
      { pubkey: admin, isSigner: true, isWritable: true },
      { pubkey: configPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(buf),
  });
}

export function buildCreateProfileIx({ payer, realm, member }) {
  const { u8: buf } = allocBuffer(1);
  writeU8(buf, 0, 2); // variant 2 — CreateProfile

  const [configPDA] = findReputationConfigPDA(realm);
  const [profilePDA] = findReputationProfilePDA(realm, member);

  return new TransactionInstruction({
    programId: REPUTATION_ENGINE_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: configPDA, isSigner: false, isWritable: false },
      { pubkey: member instanceof PublicKey ? member : new PublicKey(member), isSigner: false, isWritable: false },
      { pubkey: profilePDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(buf),
  });
}

// ---------------------------------------------------------------------------
// 1e. Account deserializers
// ---------------------------------------------------------------------------

export function parseAdapterConfig(data) {
  let off = 0;
  const tag = readU8(data, off); off = tag.next;
  const realm = readPubkey(data, off); off = realm.next;
  const admin = readPubkey(data, off); off = admin.next;
  const governanceProgramId = readPubkey(data, off); off = governanceProgramId.next;
  const quadraticVotingProgram = readPubkey(data, off); off = quadraticVotingProgram.next;
  const reputationEngineProgram = readPubkey(data, off); off = reputationEngineProgram.next;
  const councilOverrideAuthority = readPubkey(data, off); off = councilOverrideAuthority.next;
  const bump = readU8(data, off); off = bump.next;
  const minReputationBps = readU16LE(data, off); off = minReputationBps.next;
  const maxReputationBps = readU16LE(data, off);

  return {
    tag: tag.value,
    realm: realm.value,
    admin: admin.value,
    governanceProgramId: governanceProgramId.value,
    quadraticVotingProgram: quadraticVotingProgram.value,
    reputationEngineProgram: reputationEngineProgram.value,
    councilOverrideAuthority: councilOverrideAuthority.value,
    bump: bump.value,
    minReputationBps: minReputationBps.value,
    maxReputationBps: maxReputationBps.value,
  };
}

export function parseProposalBinding(data) {
  let off = 0;
  const tag = readU8(data, off); off = tag.next;
  const adapterConfig = readPubkey(data, off); off = adapterConfig.next;
  const realm = readPubkey(data, off); off = realm.next;
  const proposal = readPubkey(data, off); off = proposal.next;
  const quadraticBallot = readPubkey(data, off); off = quadraticBallot.next;
  const governingTokenMint = readPubkey(data, off); off = governingTokenMint.next;
  const bump = readU8(data, off); off = bump.next;
  const councilOverrideEnabled = readBool(data, off); off = councilOverrideEnabled.next;
  const councilOverrideActive = readBool(data, off); off = councilOverrideActive.next;
  const councilOverrideReasonCode = readU16LE(data, off); off = councilOverrideReasonCode.next;
  const defaultWeightExpirySlot = readU64LE(data, off); off = defaultWeightExpirySlot.next;
  const lastWeightRefreshSlot = readU64LE(data, off);

  return {
    tag: tag.value,
    adapterConfig: adapterConfig.value,
    realm: realm.value,
    proposal: proposal.value,
    quadraticBallot: quadraticBallot.value,
    governingTokenMint: governingTokenMint.value,
    bump: bump.value,
    councilOverrideEnabled: councilOverrideEnabled.value,
    councilOverrideActive: councilOverrideActive.value,
    councilOverrideReasonCode: councilOverrideReasonCode.value,
    defaultWeightExpirySlot: defaultWeightExpirySlot.value,
    lastWeightRefreshSlot: lastWeightRefreshSlot.value,
  };
}

export function parseVoterWeightRecord(data) {
  let off = 0;
  // 8-byte discriminator
  const discriminator = readBytes(data, off, 8); off = discriminator.next;
  // SPL-compatible fields
  const realm = readPubkey(data, off); off = realm.next;
  const governingTokenMint = readPubkey(data, off); off = governingTokenMint.next;
  const governingTokenOwner = readPubkey(data, off); off = governingTokenOwner.next;
  const voterWeight = readU64LE(data, off); off = voterWeight.next;
  const voterWeightExpiry = readOptionU64(data, off); off = voterWeightExpiry.next;
  const weightAction = readOptionU8(data, off); off = weightAction.next;
  const weightActionTarget = readOptionPubkey(data, off); off = weightActionTarget.next;
  const reserved = readBytes(data, off, 8); off = reserved.next;
  // Plugin extension fields
  const binding = readPubkey(data, off); off = binding.next;
  const bump = readU8(data, off); off = bump.next;
  const tokenAmountAllocated = readU64LE(data, off); off = tokenAmountAllocated.next;
  const qvVotesAllocated = readU32LE(data, off); off = qvVotesAllocated.next;
  const reputationMultiplierBps = readU16LE(data, off); off = reputationMultiplierBps.next;
  const lastUpdatedSlot = readU64LE(data, off); off = lastUpdatedSlot.next;
  const councilOverrideActive = readBool(data, off);

  return {
    realm: realm.value,
    governingTokenMint: governingTokenMint.value,
    governingTokenOwner: governingTokenOwner.value,
    voterWeight: voterWeight.value,
    voterWeightExpiry: voterWeightExpiry.value,
    weightAction: weightAction.value,
    weightActionTarget: weightActionTarget.value,
    binding: binding.value,
    bump: bump.value,
    tokenAmountAllocated: tokenAmountAllocated.value,
    qvVotesAllocated: qvVotesAllocated.value,
    reputationMultiplierBps: reputationMultiplierBps.value,
    lastUpdatedSlot: lastUpdatedSlot.value,
    councilOverrideActive: councilOverrideActive.value,
  };
}

export function parseQuadraticBallot(data) {
  let off = 0;
  const tag = readU8(data, off); off = tag.next;
  const authority = readPubkey(data, off); off = authority.next;
  const realm = readPubkey(data, off); off = realm.next;
  const proposal = readPubkey(data, off); off = proposal.next;
  const bump = readU8(data, off); off = bump.next;
  const minReputationBps = readU16LE(data, off); off = minReputationBps.next;
  const maxReputationBps = readU16LE(data, off); off = maxReputationBps.next;
  const votingStartsAt = readI64LE(data, off); off = votingStartsAt.next;
  const votingEndsAt = readI64LE(data, off); off = votingEndsAt.next;
  const finalized = readBool(data, off); off = finalized.next;
  const totalRegisteredVoters = readU32LE(data, off); off = totalRegisteredVoters.next;
  const totalCreditsBudget = readU64LE(data, off); off = totalCreditsBudget.next;
  const yesTallyScaled = readU128LE(data, off); off = yesTallyScaled.next;
  const noTallyScaled = readU128LE(data, off); off = noTallyScaled.next;
  const abstainTallyScaled = readU128LE(data, off);

  return {
    tag: tag.value,
    authority: authority.value,
    realm: realm.value,
    proposal: proposal.value,
    bump: bump.value,
    minReputationBps: minReputationBps.value,
    maxReputationBps: maxReputationBps.value,
    votingStartsAt: votingStartsAt.value,
    votingEndsAt: votingEndsAt.value,
    finalized: finalized.value,
    totalRegisteredVoters: totalRegisteredVoters.value,
    totalCreditsBudget: totalCreditsBudget.value,
    yesTallyScaled: yesTallyScaled.value,
    noTallyScaled: noTallyScaled.value,
    abstainTallyScaled: abstainTallyScaled.value,
  };
}

export function parseVoterAllocation(data) {
  let off = 0;
  const tag = readU8(data, off); off = tag.next;
  const ballot = readPubkey(data, off); off = ballot.next;
  const voter = readPubkey(data, off); off = voter.next;
  const bump = readU8(data, off); off = bump.next;
  const reputationMultiplierBps = readU16LE(data, off); off = reputationMultiplierBps.next;
  const creditsBudget = readU64LE(data, off); off = creditsBudget.next;
  const creditsSpent = readU64LE(data, off); off = creditsSpent.next;
  const yesVotes = readU32LE(data, off); off = yesVotes.next;
  const noVotes = readU32LE(data, off); off = noVotes.next;
  const abstainVotes = readU32LE(data, off); off = abstainVotes.next;
  const lastUpdatedSlot = readU64LE(data, off);

  return {
    tag: tag.value,
    ballot: ballot.value,
    voter: voter.value,
    bump: bump.value,
    reputationMultiplierBps: reputationMultiplierBps.value,
    creditsBudget: creditsBudget.value,
    creditsSpent: creditsSpent.value,
    yesVotes: yesVotes.value,
    noVotes: noVotes.value,
    abstainVotes: abstainVotes.value,
    lastUpdatedSlot: lastUpdatedSlot.value,
  };
}

// ---------------------------------------------------------------------------
// 1f. Account fetchers
// ---------------------------------------------------------------------------

async function fetchAccountData(connection, pubkey) {
  const info = await connection.getAccountInfo(pubkey);
  if (!info || !info.data) return null;
  return new Uint8Array(info.data);
}

export async function fetchAdapterConfig(connection, realm) {
  const [pda] = findAdapterConfigPDA(realm);
  const data = await fetchAccountData(connection, pda);
  return data ? parseAdapterConfig(data) : null;
}

export async function fetchProposalBinding(connection, realm, proposal) {
  const [pda] = findProposalBindingPDA(realm, proposal);
  const data = await fetchAccountData(connection, pda);
  return data ? parseProposalBinding(data) : null;
}

export async function fetchVoterWeightRecord(connection, binding, voter) {
  const [pda] = findVoterWeightRecordPDA(binding, voter);
  const data = await fetchAccountData(connection, pda);
  return data ? parseVoterWeightRecord(data) : null;
}

export async function fetchQuadraticBallot(connection, realm, proposal) {
  const [pda] = findBallotPDA(realm, proposal);
  const data = await fetchAccountData(connection, pda);
  return data ? parseQuadraticBallot(data) : null;
}

export async function fetchVoterAllocation(connection, ballot, voter) {
  const [pda] = findAllocationPDA(ballot, voter);
  const data = await fetchAccountData(connection, pda);
  return data ? parseVoterAllocation(data) : null;
}

// ---------------------------------------------------------------------------
// 1g. High-level action functions
// ---------------------------------------------------------------------------

/** Map choice string id ("yes"/"no"/"abstain") to on-chain enum value */
function choiceIdToEnum(choiceId) {
  if (choiceId === "yes") return 0;
  if (choiceId === "no") return 1;
  if (choiceId === "abstain") return 2;
  throw new Error(`Unknown choice: ${choiceId}`);
}

/**
 * Extract a human-readable error from a wallet adapter or Solana error.
 * Wallet adapters often wrap the real error in a generic message.
 */
function extractTxError(err) {
  if (!err) return "Transaction failed.";

  // Check for simulation logs with a program error
  const logs = err?.logs || err?.simulationResponse?.logs;
  if (Array.isArray(logs)) {
    for (let i = logs.length - 1; i >= 0; i--) {
      if (logs[i].includes("Error") || logs[i].includes("failed")) {
        return logs[i];
      }
    }
  }

  // WalletSendTransactionError wraps a cause or message
  if (err.error) return extractTxError(err.error);

  // Solana SendTransactionError has a message with logs
  const msg = err.message || String(err);

  // "Attempt to debit an account but found no record of a prior credit"
  if (msg.includes("no record of a prior credit") || msg.includes("0x1")) {
    return "Insufficient SOL balance. Request an airdrop from a devnet faucet.";
  }
  if (msg.includes("blockhash not found") || msg.includes("Blockhash")) {
    return "Transaction expired. Please try again.";
  }
  if (msg.includes("User rejected")) {
    return "Transaction cancelled by wallet.";
  }
  if (msg === "Unexpected error") {
    return "Transaction simulation failed. This usually means insufficient SOL or the program rejected the instruction. Check your devnet balance.";
  }

  return msg;
}

/** Wraps sendTransaction + confirmTransaction with progress callbacks and better errors. */
async function sendAndConfirm(connection, wallet, tx, onProgress) {
  try {
    onProgress?.("sending");
    const txSignature = await wallet.sendTransaction(tx, connection, {
      skipPreflight: false,
      preflightCommitment: "confirmed",
    });
    onProgress?.("confirming");
    await connection.confirmTransaction(txSignature, "confirmed");
    onProgress?.("confirmed");
    return txSignature;
  } catch (err) {
    const readable = extractTxError(err);
    throw new Error(readable);
  }
}

/**
 * Initialize a DAO on-chain: creates RealmReputationConfig + AdapterConfig in a single tx.
 * @param {Function} [onProgress] - callback: "sending" | "confirming" | "confirmed"
 */
export async function initializeDao(connection, wallet, onProgress) {
  if (!wallet.publicKey || !wallet.sendTransaction) {
    throw new Error("Wallet not connected or does not support sendTransaction.");
  }

  const admin = wallet.publicKey;
  const realmKeypair = Keypair.generate();
  const realm = realmKeypair.publicKey;

  onProgress?.("building");

  const repIx = buildInitRealmConfigIx({
    admin,
    realm,
  });

  const adapterIx = buildInitializeAdapterIx({
    admin,
    realm,
    governanceProgramId: admin,
    quadraticVotingProgram: QUADRATIC_VOTING_PROGRAM_ID,
    reputationEngineProgram: REPUTATION_ENGINE_PROGRAM_ID,
    minReputationBps: 5000,
    maxReputationBps: 20000,
  });

  const tx = new Transaction().add(repIx, adapterIx);
  tx.feePayer = admin;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

  const txSignature = await sendAndConfirm(connection, wallet, tx, onProgress);

  const [adapterConfigPDA] = findAdapterConfigPDA(realm);
  const [repConfigPDA] = findReputationConfigPDA(realm);

  return {
    realmPubkey: realm.toBase58(),
    adapterConfigPDA: adapterConfigPDA.toBase58(),
    repConfigPDA: repConfigPDA.toBase58(),
    txSignature,
  };
}

/**
 * Create a proposal on-chain: InitializeBallot + BindProposal in a single tx.
 * @param {Function} [onProgress] - callback: "building" | "sending" | "confirming" | "confirmed"
 */
export async function createProposalOnChain(connection, wallet, realmPubkey, durationHours, onProgress) {
  if (!wallet.publicKey || !wallet.sendTransaction) {
    throw new Error("Wallet not connected or does not support sendTransaction.");
  }

  const authority = wallet.publicKey;
  const realm = new PublicKey(realmPubkey);
  const proposalKeypair = Keypair.generate();
  const proposal = proposalKeypair.publicKey;
  const mintKeypair = Keypair.generate();
  const mint = mintKeypair.publicKey;

  onProgress?.("building");

  const nowSec = Math.floor(Date.now() / 1000);
  const endSec = nowSec + (durationHours || 24) * 3600;

  const ballotIx = buildInitializeBallotIx({
    authority,
    realm,
    proposal,
    minReputationBps: 5000,
    maxReputationBps: 20000,
    votingStartsAt: nowSec,
    votingEndsAt: endSec,
  });

  const [ballotPDA] = findBallotPDA(realm, proposal);

  const bindIx = buildBindProposalIx({
    admin: authority,
    realm,
    proposal,
    quadraticBallot: ballotPDA,
    governingTokenMint: mint,
    councilOverrideEnabled: false,
    defaultWeightExpirySlot: 0,
  });

  // Auto-register the ballot creator as a voter so they can vote immediately
  const registerIx = buildRegisterVoterIx({
    authority,
    realm,
    proposal,
    voter: authority,
    creditsBudget: 100,
    reputationMultiplierBps: 10000,
  });

  const tx = new Transaction().add(ballotIx, bindIx, registerIx);
  tx.feePayer = authority;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

  const txSignature = await sendAndConfirm(connection, wallet, tx, onProgress);

  const [bindingPDA] = findProposalBindingPDA(realm, proposal);

  return {
    proposalPubkey: proposal.toBase58(),
    ballotPDA: ballotPDA.toBase58(),
    bindingPDA: bindingPDA.toBase58(),
    mintPubkey: mint.toBase58(),
    txSignature,
  };
}

/**
 * Cast a vote on-chain. Automatically registers voter + creates VWR if needed.
 * @param {Function} [onProgress] - callback: "building" | "sending" | "confirming" | "confirmed"
 */
export async function castVoteOnChain(connection, wallet, realmPubkey, proposalPubkey, bindingPDA, choiceId, onProgress) {
  if (!wallet.publicKey || !wallet.sendTransaction) {
    throw new Error("Wallet not connected or does not support sendTransaction.");
  }

  const voter = wallet.publicKey;
  const realm = new PublicKey(realmPubkey);
  const proposal = new PublicKey(proposalPubkey);
  const binding = new PublicKey(bindingPDA);
  const [ballotPDA] = findBallotPDA(realm, proposal);
  const [allocPDA] = findAllocationPDA(ballotPDA, voter);
  const choice = choiceIdToEnum(choiceId);

  onProgress?.("building");

  const tx = new Transaction();

  // Check if voter allocation exists — only the ballot authority can register voters
  const allocInfo = await connection.getAccountInfo(allocPDA);
  if (!allocInfo) {
    // Fetch the ballot to check if this voter is the authority
    const ballotData = await fetchAccountData(connection, ballotPDA);
    if (!ballotData) {
      throw new Error("Ballot account not found on-chain.");
    }
    const ballot = parseQuadraticBallot(ballotData);
    if (ballot.authority.toBase58() === voter.toBase58()) {
      // Voter is the ballot authority — can self-register
      tx.add(
        buildRegisterVoterIx({
          authority: voter,
          realm,
          proposal,
          voter,
          creditsBudget: 100,
          reputationMultiplierBps: 10000,
        }),
      );
    } else {
      throw new Error("You are not registered for this ballot. The ballot creator must register you first.");
    }
  }

  // Check if VWR exists
  const [vwrPDA] = findVoterWeightRecordPDA(binding, voter);
  const vwrInfo = await connection.getAccountInfo(vwrPDA);
  if (!vwrInfo) {
    tx.add(buildCreateVWRIx({ payer: voter, realm, proposal, voter }));
  }

  // Cast the vote (additional_votes must be >= 1)
  tx.add(
    buildCastVoteIx({
      voter,
      realm,
      proposal,
      choice,
      additionalVotes: 1,
    }),
  );

  // Refresh VWR after voting
  tx.add(buildRefreshVWRIx({
    caller: voter,
    realm,
    proposal,
    voter,
    tokenAmountAllocated: 0,
    qvVotesAllocated: 1,
    reputationMultiplierBps: 10000,
    weightAction: 0,
    weightActionTarget: proposal,
  }));

  tx.feePayer = voter;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

  const txSignature = await sendAndConfirm(connection, wallet, tx, onProgress);

  return { txSignature };
}

/**
 * Finalize a ballot on-chain.
 * @param {Function} [onProgress] - callback: "building" | "sending" | "confirming" | "confirmed"
 */
export async function finalizeBallotOnChain(connection, wallet, realmPubkey, proposalPubkey, onProgress) {
  if (!wallet.publicKey || !wallet.sendTransaction) {
    throw new Error("Wallet not connected or does not support sendTransaction.");
  }

  const authority = wallet.publicKey;
  const realm = new PublicKey(realmPubkey);
  const proposal = new PublicKey(proposalPubkey);

  onProgress?.("building");

  const ix = buildFinalizeBallotIx({ authority, realm, proposal });
  const tx = new Transaction().add(ix);
  tx.feePayer = authority;
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

  const txSignature = await sendAndConfirm(connection, wallet, tx, onProgress);

  return { txSignature };
}
