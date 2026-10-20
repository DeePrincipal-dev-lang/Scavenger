/**
 * Error thrown when a Soroban smart contract call fails.
 * Contains an optional numeric error code extracted from the contract error.
 */
export class ContractError extends Error {
  /** @param message - Human-readable error description. */
  constructor(
    message: string,
    /** Numeric error code from the contract, if available. */
    public code?: number
  ) {
    super(message)
    this.name = 'ContractError'
  }
}

/**
 * Error thrown when a transaction fails on-chain after submission.
 */
export class TransactionError extends Error {
  /** @param message - Description of what went wrong. */
  constructor(
    message: string,
    /** Transaction hash if available. */
    public txHash?: string,
    /** Result XDR from the failed transaction if available. */
    public resultXdr?: string
  ) {
    super(message)
    this.name = 'TransactionError'
  }
}

/**
 * Error thrown when wallet signing fails or is rejected by the user.
 */
export class SigningError extends Error {
  /** @param message - Description of the signing failure. */
  constructor(message: string) {
    super(message)
    this.name = 'SigningError'
  }
}

/**
 * Error thrown when network configuration is invalid or connection fails.
 */
export class NetworkError extends Error {
  /** @param message - Description of the network issue. */
  constructor(
    message: string,
    /** The RPC URL that failed. */
    public rpcUrl?: string
  ) {
    super(message)
    this.name = 'NetworkError'
  }
}

/**
 * Error thrown when a transaction confirmation times out.
 */
export class TimeoutError extends Error {
  /** @param message - Description of the timeout. */
  constructor(message: string) {
    super(message)
    this.name = 'TimeoutError'
  }
}

/** Union type of all SDK-specific errors. */
export type SdkError = ContractError | TransactionError | SigningError | NetworkError | TimeoutError

/**
 * Canonical error codes shared between the indexer and SDK consumers.
 *
 * Mirrors the codes emitted by `indexer/src/errors/AppError.ts` so that
 * clients consuming both the SDK and the indexer see the same strings.
 */
export enum ErrorCode {
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  DATABASE_ERROR = 'DATABASE_ERROR',
  NETWORK_ERROR = 'NETWORK_ERROR',
  NOT_FOUND = 'NOT_FOUND',
  UNAUTHORIZED = 'UNAUTHORIZED',
  INTERNAL_ERROR = 'INTERNAL_ERROR',
}

/**
 * Envelope used when a service returns a structured error response.
 *
 * Services MUST serialise errors using this shape so that consumers of both
 * the indexer HTTP API and the SDK can parse failures uniformly.
 */
export interface ErrorEnvelope {
  code: ErrorCode
  message: string
  statusCode: number
  details?: Record<string, unknown>
}
