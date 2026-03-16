export enum CommitmentLevel {
  DeepFocus = 0,
  ActiveEngagement = 1,
  Background = 2,
}

export enum SlotStatus {
  Pending = 0,
  Active = 1,
  Verified = 2,
  Disputed = 3,
  Expired = 4,
  Cancelled = 5,
}

export interface CommitmentSlot {
  owner: `0x${string}`;
  start: bigint;
  end: bigint;
  client: `0x${string}`;
  level: CommitmentLevel;
  challengeStreamSeed: `0x${string}`;
  status: SlotStatus;
}

export interface ContractAddresses {
  timeToken: `0x${string}`;
  commitmentRegistry: `0x${string}`;
  engagementVerifier: `0x${string}`;
  validatorRegistry: `0x${string}`;
  marketplace: `0x${string}`;
}

export const LEVEL_MULTIPLIERS: Record<CommitmentLevel, number> = {
  [CommitmentLevel.DeepFocus]: 3.0,
  [CommitmentLevel.ActiveEngagement]: 1.5,
  [CommitmentLevel.Background]: 1.0,
};

export const LEVEL_NAMES: Record<CommitmentLevel, string> = {
  [CommitmentLevel.DeepFocus]: "Deep Focus",
  [CommitmentLevel.ActiveEngagement]: "Active Engagement",
  [CommitmentLevel.Background]: "Background",
};

export const STATUS_NAMES: Record<SlotStatus, string> = {
  [SlotStatus.Pending]: "Pending",
  [SlotStatus.Active]: "Active",
  [SlotStatus.Verified]: "Verified",
  [SlotStatus.Disputed]: "Disputed",
  [SlotStatus.Expired]: "Expired",
  [SlotStatus.Cancelled]: "Cancelled",
};

export function estimateTimeEarned(
  durationHours: number,
  level: CommitmentLevel,
  score: number
): number {
  return durationHours * LEVEL_MULTIPLIERS[level] * score;
}
