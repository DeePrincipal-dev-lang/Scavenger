/**
 * @scavngr/sdk - TypeScript SDK for Scavngr Soroban smart contracts.
 *
 * @packageDocumentation
 */

export { ScavengerClient } from './client'
export type { ClientOptions } from './types'

export {
  Role,
  WasteType,
  CertificationLevel,
  Network,
} from './types'

export type {
  Participant,
  Incentive,
  Material,
  Waste,
  WasteTransfer,
  ParticipantStats,
  GlobalMetrics,
  SupplyChainStats,
  NetworkConfig,
  MaterialBatchItem,
} from './types'

export {
  ContractError,
  TransactionError,
  SigningError,
  NetworkError,
  TimeoutError,
} from './errors'
export type { SdkError, ErrorEnvelope } from './errors'
export { ErrorCode } from './errors'

export {
  resolveNetwork,
  isValidStellarAddress,
  getAvailableNetworks,
  getNetworkLabel,
} from './network'

export {
  signWithFreighter,
  signWithSecretKey,
  FreighterSigningStrategy,
  SecretKeySigningStrategy,
} from './signing'
export type { SigningStrategy } from './signing'
