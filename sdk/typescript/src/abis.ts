export const TIMETokenABI = [
  {
    inputs: [{ name: "account", type: "address" }],
    name: "balanceOf",
    outputs: [{ name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "totalSupply",
    outputs: [{ name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "account", type: "address" }],
    name: "stakedBalance",
    outputs: [{ name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "amount", type: "uint256" }],
    name: "stake",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "amount", type: "uint256" }],
    name: "requestUnstake",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "completeUnstake",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "amount", type: "uint256" }],
    name: "burn",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { name: "spender", type: "address" },
      { name: "amount", type: "uint256" },
    ],
    name: "approve",
    outputs: [{ name: "", type: "bool" }],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { name: "to", type: "address" },
      { name: "amount", type: "uint256" },
    ],
    name: "transfer",
    outputs: [{ name: "", type: "bool" }],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const;

export const CommitmentRegistryABI = [
  {
    inputs: [
      { name: "start", type: "uint64" },
      { name: "end", type: "uint64" },
      { name: "client", type: "address" },
      { name: "level", type: "uint8" },
    ],
    name: "registerSlot",
    outputs: [{ name: "slotId", type: "uint256" }],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "slotId", type: "uint256" }],
    name: "cancelSlot",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "user", type: "address" }],
    name: "getActiveSlots",
    outputs: [{ name: "", type: "uint256[]" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "user", type: "address" }],
    name: "getUserSlots",
    outputs: [{ name: "", type: "uint256[]" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "slotId", type: "uint256" }],
    name: "slots",
    outputs: [
      { name: "owner", type: "address" },
      { name: "start", type: "uint64" },
      { name: "end", type: "uint64" },
      { name: "client", type: "address" },
      { name: "level", type: "uint8" },
      { name: "challengeStreamSeed", type: "bytes32" },
      { name: "status", type: "uint8" },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { name: "user", type: "address" },
      { name: "start", type: "uint64" },
      { name: "end", type: "uint64" },
      { name: "level", type: "uint8" },
    ],
    name: "checkOverlap",
    outputs: [{ name: "", type: "bool" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "level", type: "uint8" }],
    name: "getLevelMultiplier",
    outputs: [{ name: "", type: "uint256" }],
    stateMutability: "pure",
    type: "function",
  },
] as const;

export const EngagementVerifierABI = [
  {
    inputs: [
      { name: "slotId", type: "uint256" },
      { name: "proofHash", type: "bytes32" },
      { name: "challengesPassed", type: "uint16" },
      { name: "challengesTotal", type: "uint16" },
    ],
    name: "submitProof",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "slotId", type: "uint256" }],
    name: "getVerificationScore",
    outputs: [{ name: "scoreBps", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "slotId", type: "uint256" }],
    name: "getProofCount",
    outputs: [{ name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "slotId", type: "uint256" }],
    name: "finalizeSlot",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "slotId", type: "uint256" }],
    name: "slotFinalized",
    outputs: [{ name: "", type: "bool" }],
    stateMutability: "view",
    type: "function",
  },
] as const;

export const ValidatorRegistryABI = [
  {
    inputs: [],
    name: "register",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "deactivate",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { name: "slotId", type: "uint256" },
      { name: "approved", type: "bool" },
    ],
    name: "reportValidation",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "claimRewards",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [],
    name: "getActiveValidatorCount",
    outputs: [{ name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [{ name: "validator", type: "address" }],
    name: "pendingRewards",
    outputs: [{ name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
] as const;

export const CommitmentMarketplaceABI = [
  {
    inputs: [
      { name: "budget", type: "uint256" },
      { name: "requiredLevel", type: "uint8" },
      { name: "startTime", type: "uint64" },
      { name: "endTime", type: "uint64" },
    ],
    name: "createContract",
    outputs: [{ name: "contractId", type: "uint256" }],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { name: "contractId", type: "uint256" },
      { name: "slotId", type: "uint256" },
    ],
    name: "acceptContract",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "contractId", type: "uint256" }],
    name: "settleContract",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "contractId", type: "uint256" }],
    name: "cancelContract",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [{ name: "contractId", type: "uint256" }],
    name: "disputeContract",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const;
