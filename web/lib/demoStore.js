export const DEMO_STORE_KEY = "ballot-guardian-demo:v1";

export const DEFAULT_PROPOSAL_CHOICES = [
  { id: "yes", label: "Yes" },
  { id: "no", label: "No" },
  { id: "abstain", label: "Abstain" },
];

export function createEmptyDemoStore() {
  return {
    version: 1,
    daos: [],
  };
}

export function createSeededDemoStore() {
  const now = new Date();
  const h48 = new Date(now.getTime() + 48 * 60 * 60 * 1000).toISOString();
  const h24 = new Date(now.getTime() + 24 * 60 * 60 * 1000).toISOString();
  const createdAt = new Date(now.getTime() - 2 * 60 * 60 * 1000).toISOString();
  const closedAt = new Date(now.getTime() - 30 * 60 * 1000).toISOString();

  return {
    version: 1,
    daos: [
      {
        id: "dao_seed-bma-jdc",
        name: "BMA Junior Doctors Committee",
        slug: "bma-junior-doctors-committee",
        description:
          "The professional body representing over 190,000 doctors across the UK. This committee oversees ballot governance for junior doctor members.",
        createdAt,
        createdBy: "7xKX...demoSeed",
        proof: null,
        proposals: [
          {
            id: "prop_seed-industrial-action",
            question:
              "Should the BMA authorise industrial action over pay restoration?",
            description:
              "Following failed negotiations with the Department of Health, this ballot seeks member authorisation for a programme of industrial action to secure pay restoration to 2008 levels.",
            createdAt,
            closesAt: h48,
            createdBy: "7xKX...demoSeed",
            proof: null,
            proposalPubkey: null,
            ballotPDA: null,
            bindingPDA: null,
            governingTokenMint: null,
            createTxSignature: null,
            choices: DEFAULT_PROPOSAL_CHOICES,
            votes: [
              {
                id: "vote_seed-1",
                voter: "3aFb...voter1",
                choiceId: "yes",
                castAt: new Date(now.getTime() - 90 * 60 * 1000).toISOString(),
                proof: null,
              },
              {
                id: "vote_seed-2",
                voter: "9kRm...voter2",
                choiceId: "yes",
                castAt: new Date(now.getTime() - 45 * 60 * 1000).toISOString(),
                proof: null,
              },
              {
                id: "vote_seed-3",
                voter: "Dp7Q...voter3",
                choiceId: "no",
                castAt: new Date(now.getTime() - 20 * 60 * 1000).toISOString(),
                proof: null,
              },
            ],
          },
          {
            id: "prop_seed-governance-charter",
            question:
              "Should the committee adopt the new governance charter?",
            description:
              "Proposal to replace the existing committee governance framework with an updated charter that includes on-chain voting procedures and transparent audit trails.",
            createdAt,
            closesAt: h24,
            createdBy: "7xKX...demoSeed",
            proof: null,
            proposalPubkey: null,
            ballotPDA: null,
            bindingPDA: null,
            governingTokenMint: null,
            createTxSignature: null,
            choices: DEFAULT_PROPOSAL_CHOICES,
            votes: [
              {
                id: "vote_seed-4",
                voter: "3aFb...voter1",
                choiceId: "yes",
                castAt: new Date(now.getTime() - 60 * 60 * 1000).toISOString(),
                proof: null,
              },
            ],
          },
          {
            id: "prop_seed-pay-offer",
            question:
              "Should the union accept the government's revised pay offer of 6.5%?",
            description:
              "The government has tabled a revised offer of 6.5% over two years. The BMA negotiating team recommends rejection, citing inflation-adjusted losses of 26% since 2008.",
            createdAt: new Date(now.getTime() - 5 * 60 * 60 * 1000).toISOString(),
            closesAt: closedAt,
            createdBy: "7xKX...demoSeed",
            proof: null,
            proposalPubkey: null,
            ballotPDA: null,
            bindingPDA: null,
            governingTokenMint: null,
            createTxSignature: null,
            choices: DEFAULT_PROPOSAL_CHOICES,
            votes: [
              {
                id: "vote_seed-5",
                voter: "3aFb...voter1",
                choiceId: "no",
                castAt: new Date(now.getTime() - 4 * 60 * 60 * 1000).toISOString(),
                proof: null,
              },
              {
                id: "vote_seed-6",
                voter: "9kRm...voter2",
                choiceId: "no",
                castAt: new Date(now.getTime() - 3.5 * 60 * 60 * 1000).toISOString(),
                proof: null,
              },
              {
                id: "vote_seed-7",
                voter: "Dp7Q...voter3",
                choiceId: "no",
                castAt: new Date(now.getTime() - 3 * 60 * 60 * 1000).toISOString(),
                proof: null,
              },
              {
                id: "vote_seed-8",
                voter: "Hk2L...voter4",
                choiceId: "yes",
                castAt: new Date(now.getTime() - 2.5 * 60 * 60 * 1000).toISOString(),
                proof: null,
              },
              {
                id: "vote_seed-9",
                voter: "Wnx8...voter5",
                choiceId: "abstain",
                castAt: new Date(now.getTime() - 2 * 60 * 60 * 1000).toISOString(),
                proof: null,
              },
            ],
          },
        ],
      },
    ],
  };
}

export function loadDemoStore() {
  if (typeof window === "undefined") {
    return createEmptyDemoStore();
  }

  try {
    const raw = window.localStorage.getItem(DEMO_STORE_KEY);
    if (!raw) return createSeededDemoStore();

    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || !Array.isArray(parsed.daos)) {
      return createSeededDemoStore();
    }

    return {
      version: 1,
      daos: parsed.daos,
    };
  } catch {
    return createSeededDemoStore();
  }
}

export function saveDemoStore(store) {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(DEMO_STORE_KEY, JSON.stringify(store));
}

export function registerDao(store, input) {
  const name = normalizeText(input.name);
  const description = normalizeText(input.description);

  if (!name) {
    throw new Error("Union/DAO name is required.");
  }

  const dao = {
    id: makeId("dao"),
    name,
    slug: slugify(name),
    description,
    createdAt: new Date().toISOString(),
    createdBy: input.createdBy,
    proof: input.proof || null,
    realmPubkey: input.realmPubkey || null,
    adapterConfigPDA: input.adapterConfigPDA || null,
    repConfigPDA: input.repConfigPDA || null,
    initTxSignature: input.initTxSignature || null,
    proposals: [],
  };

  return {
    store: {
      ...store,
      daos: [dao, ...store.daos],
    },
    dao,
  };
}

export function deleteDao(store, input) {
  const { daoId, caller } = input;
  if (!daoId) throw new Error("No DAO selected.");
  if (!caller) throw new Error("Wallet address is required.");

  const dao = store.daos.find((d) => d.id === daoId);
  if (!dao) throw new Error("DAO not found.");
  if (dao.createdBy !== caller) {
    throw new Error("Only the creator can delete this organization.");
  }

  return {
    store: {
      ...store,
      daos: store.daos.filter((d) => d.id !== daoId),
    },
  };
}

export function closeBallot(store, input) {
  const { daoId, proposalId, caller } = input;
  if (!daoId || !proposalId) throw new Error("Missing proposal selection.");
  if (!caller) throw new Error("Wallet address is required.");

  let found = false;

  const daos = store.daos.map((dao) => {
    if (dao.id !== daoId) return dao;

    return {
      ...dao,
      proposals: (dao.proposals || []).map((proposal) => {
        if (proposal.id !== proposalId) return proposal;
        found = true;

        if (proposal.createdBy !== caller) {
          throw new Error("Only the ballot creator can close this ballot.");
        }
        if (isProposalClosed(proposal)) {
          throw new Error("This ballot is already closed.");
        }

        return {
          ...proposal,
          closesAt: new Date().toISOString(),
        };
      }),
    };
  });

  if (!found) throw new Error("Proposal not found.");

  return { store: { ...store, daos } };
}

export function createProposal(store, input) {
  const daoId = input.daoId;
  const question = normalizeText(input.question);
  const description = normalizeText(input.description);
  const durationHours = clampInteger(input.durationHours, 1, 24 * 30);

  if (!daoId) throw new Error("Select a union/DAO first.");
  if (!question) throw new Error("Proposal question is required.");

  const createdAt = new Date();
  const closesAt = new Date(createdAt.getTime() + durationHours * 60 * 60 * 1000);

  let createdProposal = null;
  let foundDao = false;

  const daos = store.daos.map((dao) => {
    if (dao.id !== daoId) return dao;
    foundDao = true;

    const proposal = {
      id: makeId("prop"),
      question,
      description,
      createdAt: createdAt.toISOString(),
      closesAt: closesAt.toISOString(),
      createdBy: input.createdBy,
      proof: input.proof || null,
      proposalPubkey: input.proposalPubkey || null,
      ballotPDA: input.ballotPDA || null,
      bindingPDA: input.bindingPDA || null,
      governingTokenMint: input.governingTokenMint || null,
      createTxSignature: input.createTxSignature || null,
      choices: DEFAULT_PROPOSAL_CHOICES,
      votes: [],
    };

    createdProposal = proposal;

    return {
      ...dao,
      proposals: [proposal, ...(dao.proposals || [])],
    };
  });

  if (!foundDao || !createdProposal) {
    throw new Error("Selected union/DAO could not be found.");
  }

  return {
    store: { ...store, daos },
    proposal: createdProposal,
  };
}

export function castVote(store, input) {
  const { daoId, proposalId, voter, choiceId, proof } = input;
  if (!daoId || !proposalId) throw new Error("Missing proposal selection.");
  if (!voter) throw new Error("Wallet address is required.");

  const now = new Date();
  let targetFound = false;
  let updatedProposal = null;

  const daos = store.daos.map((dao) => {
    if (dao.id !== daoId) return dao;

    return {
      ...dao,
      proposals: (dao.proposals || []).map((proposal) => {
        if (proposal.id !== proposalId) return proposal;
        targetFound = true;

        if (isProposalClosed(proposal, now)) {
          throw new Error("This proposal is closed.");
        }

        const validChoice = (proposal.choices || DEFAULT_PROPOSAL_CHOICES).some(
          (choice) => choice.id === choiceId,
        );
        if (!validChoice) {
          throw new Error("Invalid vote choice.");
        }

        const existingVotes = Array.isArray(proposal.votes) ? proposal.votes : [];
        const existingIndex = existingVotes.findIndex((vote) => vote.voter === voter);

        const nextVote = {
          id: existingIndex >= 0 ? existingVotes[existingIndex].id : makeId("vote"),
          voter,
          choiceId,
          castAt: now.toISOString(),
          proof: proof || null,
          voteTxSignature: input.voteTxSignature || null,
        };

        const nextVotes =
          existingIndex >= 0
            ? existingVotes.map((vote, index) => (index === existingIndex ? nextVote : vote))
            : [...existingVotes, nextVote];

        updatedProposal = {
          ...proposal,
          votes: nextVotes,
        };

        return updatedProposal;
      }),
    };
  });

  if (!targetFound || !updatedProposal) {
    throw new Error("Proposal not found.");
  }

  return {
    store: { ...store, daos },
    proposal: updatedProposal,
  };
}

export function getProposalTallies(proposal) {
  const counts = { yes: 0, no: 0, abstain: 0 };
  for (const vote of proposal.votes || []) {
    if (vote.choiceId in counts) {
      counts[vote.choiceId] += 1;
    }
  }
  return counts;
}

export function determineBallotResult(proposal, onChainBallot) {
  if (!isProposalClosed(proposal)) return null;

  const hasOnChain = onChainBallot && onChainBallot.proposalId === proposal.id;
  let yes, no, abstain, isReputationWeighted, isFinalized;

  if (hasOnChain) {
    yes = Number(onChainBallot.yesTallyScaled);
    no = Number(onChainBallot.noTallyScaled);
    abstain = Number(onChainBallot.abstainTallyScaled);
    isReputationWeighted = true;
    isFinalized = !!onChainBallot.finalized;
  } else {
    const tallies = getProposalTallies(proposal);
    yes = tallies.yes;
    no = tallies.no;
    abstain = tallies.abstain;
    isReputationWeighted = false;
    isFinalized = false;
  }

  const totalWeight = yes + no + abstain;
  const totalVoters = (proposal.votes || []).length;
  const decidingVotes = yes + no;

  let outcome, outcomeLabel;
  if (totalWeight === 0) {
    outcome = "NO_VOTES";
    outcomeLabel = "No votes cast";
  } else if (decidingVotes === 0) {
    outcome = "TIE";
    outcomeLabel = "Tied — no majority";
  } else if (yes > no) {
    outcome = "YES";
    outcomeLabel = "Motion carries";
  } else if (no > yes) {
    outcome = "NO";
    outcomeLabel = "Motion fails";
  } else {
    outcome = "TIE";
    outcomeLabel = "Tied — no majority";
  }

  const marginOfVictory =
    decidingVotes > 0
      ? Math.round((Math.abs(yes - no) / decidingVotes) * 100)
      : 0;

  const registeredVoters = hasOnChain && onChainBallot.registeredVoters
    ? Number(onChainBallot.registeredVoters)
    : null;
  const turnoutPct = registeredVoters
    ? Math.round((totalVoters / registeredVoters) * 100)
    : null;
  const quorumMet = turnoutPct != null ? turnoutPct >= 50 : null;

  return {
    outcome,
    outcomeLabel,
    yes,
    no,
    abstain,
    totalWeight,
    totalVoters,
    registeredVoters,
    turnoutPct,
    quorumMet,
    marginOfVictory,
    isReputationWeighted,
    isFinalized,
    closedAt: proposal.closesAt,
    ballotPDA: proposal.ballotPDA || null,
    createTxSignature: proposal.createTxSignature || null,
  };
}

export function getWalletVote(proposal, walletAddress) {
  if (!walletAddress) return null;
  return (proposal.votes || []).find((vote) => vote.voter === walletAddress) || null;
}

export function isProposalClosed(proposal, now = new Date()) {
  return new Date(proposal.closesAt).getTime() <= now.getTime();
}

export function getDaoById(store, daoId) {
  return store.daos.find((dao) => dao.id === daoId) || null;
}

function normalizeText(value) {
  return String(value || "").trim().replace(/\s+/g, " ");
}

function slugify(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
}

function clampInteger(value, min, max) {
  const n = Number.parseInt(String(value), 10);
  if (Number.isNaN(n)) return min;
  return Math.min(max, Math.max(min, n));
}

export function updateDaoOnChainFields(store, daoId, fields) {
  const daos = store.daos.map((dao) => {
    if (dao.id !== daoId) return dao;
    return { ...dao, ...fields };
  });
  return { ...store, daos };
}

export function updateProposalOnChainFields(store, daoId, proposalId, fields) {
  const daos = store.daos.map((dao) => {
    if (dao.id !== daoId) return dao;
    return {
      ...dao,
      proposals: (dao.proposals || []).map((p) => {
        if (p.id !== proposalId) return p;
        return { ...p, ...fields };
      }),
    };
  });
  return { ...store, daos };
}

function makeId(prefix) {
  const uuid =
    typeof globalThis !== "undefined" && globalThis.crypto && globalThis.crypto.randomUUID
      ? globalThis.crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  return `${prefix}_${uuid}`;
}

