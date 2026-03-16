import {
  createPublicClient,
  createWalletClient,
  http,
  type PublicClient,
  type WalletClient,
  type Chain,
  type Transport,
  type Account,
  formatUnits,
  parseUnits,
} from "viem";
import {
  TIMETokenABI,
  CommitmentRegistryABI,
  EngagementVerifierABI,
  ValidatorRegistryABI,
  CommitmentMarketplaceABI,
} from "./abis";
import {
  type ContractAddresses,
  type CommitmentSlot,
  CommitmentLevel,
  SlotStatus,
  LEVEL_MULTIPLIERS,
} from "./types";

export class PoEHCClient {
  public readonly addresses: ContractAddresses;
  public readonly publicClient: PublicClient;
  private walletClient?: WalletClient;

  constructor(
    addresses: ContractAddresses,
    rpcUrl: string,
    chain: Chain,
    walletClient?: WalletClient
  ) {
    this.addresses = addresses;
    this.publicClient = createPublicClient({
      chain,
      transport: http(rpcUrl),
    });
    this.walletClient = walletClient;
  }

  // ── TIME Token ──

  async getBalance(account: `0x${string}`): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.addresses.timeToken,
      abi: TIMETokenABI,
      functionName: "balanceOf",
      args: [account],
    });
  }

  async getBalanceFormatted(account: `0x${string}`): Promise<string> {
    const balance = await this.getBalance(account);
    return formatUnits(balance, 18);
  }

  async getStakedBalance(account: `0x${string}`): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.addresses.timeToken,
      abi: TIMETokenABI,
      functionName: "stakedBalance",
      args: [account],
    });
  }

  async getTotalSupply(): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.addresses.timeToken,
      abi: TIMETokenABI,
      functionName: "totalSupply",
    });
  }

  async stake(amount: bigint): Promise<`0x${string}`> {
    this.requireWallet();
    const hash = await this.walletClient!.writeContract({
      address: this.addresses.timeToken,
      abi: TIMETokenABI,
      functionName: "stake",
      args: [amount],
      chain: undefined as any,
      account: undefined as any,
    });
    return hash;
  }

  // ── Commitment Registry ──

  async registerSlot(
    start: bigint,
    end: bigint,
    client: `0x${string}`,
    level: CommitmentLevel
  ): Promise<`0x${string}`> {
    this.requireWallet();
    const hash = await this.walletClient!.writeContract({
      address: this.addresses.commitmentRegistry,
      abi: CommitmentRegistryABI,
      functionName: "registerSlot",
      args: [start, end, client, level],
      chain: undefined as any,
      account: undefined as any,
    });
    return hash;
  }

  async cancelSlot(slotId: bigint): Promise<`0x${string}`> {
    this.requireWallet();
    const hash = await this.walletClient!.writeContract({
      address: this.addresses.commitmentRegistry,
      abi: CommitmentRegistryABI,
      functionName: "cancelSlot",
      args: [slotId],
      chain: undefined as any,
      account: undefined as any,
    });
    return hash;
  }

  async getSlot(slotId: bigint): Promise<CommitmentSlot> {
    const result = await this.publicClient.readContract({
      address: this.addresses.commitmentRegistry,
      abi: CommitmentRegistryABI,
      functionName: "slots",
      args: [slotId],
    });
    const [owner, start, end, client, level, seed, status] = result;
    return {
      owner,
      start,
      end,
      client,
      level: level as CommitmentLevel,
      challengeStreamSeed: seed,
      status: status as SlotStatus,
    };
  }

  async getActiveSlots(user: `0x${string}`): Promise<readonly bigint[]> {
    return this.publicClient.readContract({
      address: this.addresses.commitmentRegistry,
      abi: CommitmentRegistryABI,
      functionName: "getActiveSlots",
      args: [user],
    });
  }

  async checkOverlap(
    user: `0x${string}`,
    start: bigint,
    end: bigint,
    level: CommitmentLevel
  ): Promise<boolean> {
    return this.publicClient.readContract({
      address: this.addresses.commitmentRegistry,
      abi: CommitmentRegistryABI,
      functionName: "checkOverlap",
      args: [user, start, end, level],
    });
  }

  // ── Engagement Verifier ──

  async submitProof(
    slotId: bigint,
    proofHash: `0x${string}`,
    challengesPassed: number,
    challengesTotal: number
  ): Promise<`0x${string}`> {
    this.requireWallet();
    const hash = await this.walletClient!.writeContract({
      address: this.addresses.engagementVerifier,
      abi: EngagementVerifierABI,
      functionName: "submitProof",
      args: [slotId, proofHash, challengesPassed, challengesTotal],
      chain: undefined as any,
      account: undefined as any,
    });
    return hash;
  }

  async getVerificationScore(slotId: bigint): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.addresses.engagementVerifier,
      abi: EngagementVerifierABI,
      functionName: "getVerificationScore",
      args: [slotId],
    });
  }

  async getProofCount(slotId: bigint): Promise<bigint> {
    return this.publicClient.readContract({
      address: this.addresses.engagementVerifier,
      abi: EngagementVerifierABI,
      functionName: "getProofCount",
      args: [slotId],
    });
  }

  async finalizeSlot(slotId: bigint): Promise<`0x${string}`> {
    this.requireWallet();
    const hash = await this.walletClient!.writeContract({
      address: this.addresses.engagementVerifier,
      abi: EngagementVerifierABI,
      functionName: "finalizeSlot",
      args: [slotId],
      chain: undefined as any,
      account: undefined as any,
    });
    return hash;
  }

  async isSlotFinalized(slotId: bigint): Promise<boolean> {
    return this.publicClient.readContract({
      address: this.addresses.engagementVerifier,
      abi: EngagementVerifierABI,
      functionName: "slotFinalized",
      args: [slotId],
    });
  }

  // ── Marketplace ──

  async createMarketplaceContract(
    budget: bigint,
    requiredLevel: CommitmentLevel,
    startTime: bigint,
    endTime: bigint
  ): Promise<`0x${string}`> {
    this.requireWallet();
    const hash = await this.walletClient!.writeContract({
      address: this.addresses.marketplace,
      abi: CommitmentMarketplaceABI,
      functionName: "createContract",
      args: [budget, requiredLevel, startTime, endTime],
      chain: undefined as any,
      account: undefined as any,
    });
    return hash;
  }

  async acceptMarketplaceContract(
    contractId: bigint,
    slotId: bigint
  ): Promise<`0x${string}`> {
    this.requireWallet();
    const hash = await this.walletClient!.writeContract({
      address: this.addresses.marketplace,
      abi: CommitmentMarketplaceABI,
      functionName: "acceptContract",
      args: [contractId, slotId],
      chain: undefined as any,
      account: undefined as any,
    });
    return hash;
  }

  async settleMarketplaceContract(
    contractId: bigint
  ): Promise<`0x${string}`> {
    this.requireWallet();
    const hash = await this.walletClient!.writeContract({
      address: this.addresses.marketplace,
      abi: CommitmentMarketplaceABI,
      functionName: "settleContract",
      args: [contractId],
      chain: undefined as any,
      account: undefined as any,
    });
    return hash;
  }

  // ── Helpers ──

  estimateTimeEarned(
    durationHours: number,
    level: CommitmentLevel,
    score: number
  ): number {
    return durationHours * LEVEL_MULTIPLIERS[level] * score;
  }

  parseTime(amount: string): bigint {
    return parseUnits(amount, 18);
  }

  formatTime(amount: bigint): string {
    return formatUnits(amount, 18);
  }

  private requireWallet(): void {
    if (!this.walletClient) {
      throw new Error("Wallet client required for write operations");
    }
  }
}
