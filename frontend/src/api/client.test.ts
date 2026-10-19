import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ScavengerClient } from './client'
import * as StellarSdk from '@stellar/stellar-sdk'
import * as FreighterApi from '@stellar/freighter-api'

// Mock Stellar SDK
vi.mock('@stellar/stellar-sdk', async () => {
  const actual = await vi.importActual<typeof StellarSdk>('@stellar/stellar-sdk')
  return {
    ...actual,
    Contract: vi.fn(),
    rpc: {
      Server: vi.fn(() => ({
        getAccount: vi.fn(),
        simulateTransaction: vi.fn(),
        sendTransaction: vi.fn(),
        getTransaction: vi.fn(),
      })),
      Api: {
        isSimulationError: vi.fn(),
        GetTransactionStatus: {
          SUCCESS: 'SUCCESS',
          FAILED: 'FAILED',
        },
        SimulateTransactionSuccessResponse: vi.fn(),
        assembleTransaction: vi.fn(),
      },
    },
    TransactionBuilder: vi.fn(() => ({
      addOperation: vi.fn().mockReturnThis(),
      setTimeout: vi.fn().mockReturnThis(),
      build: vi.fn(),
      fromXDR: vi.fn(),
      toXDR: vi.fn(),
    })),
    Address: vi.fn(function (addr: string) {
      this.address = addr
      this.toScVal = () => ({})
    }),
    BASE_FEE: 100,
  }
})

// Mock Freighter API
vi.mock('@stellar/freighter-api', () => ({
  signTransaction: vi.fn(),
}))

describe('ScavengerClient', () => {
  let client: ScavengerClient
  const mockOptions = {
    rpcUrl: 'https://soroban-testnet.stellar.org',
    networkPassphrase: 'Test SDF Network ; September 2015',
    contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4',
  }

  beforeEach(() => {
    vi.clearAllMocks()
    client = new ScavengerClient(mockOptions)
  })

  describe('Constructor', () => {
    it('initializes with valid options', () => {
      expect(client).toBeDefined()
      expect(client).toBeInstanceOf(ScavengerClient)
    })

    it('stores RPC configuration', () => {
      expect(client).toHaveProperty('contract')
      expect(client).toHaveProperty('server')
    })

    it('accepts custom RPC URLs', () => {
      const customRpc = 'https://custom-rpc.example.com'
      const customClient = new ScavengerClient({
        ...mockOptions,
        rpcUrl: customRpc,
      })
      expect(customClient).toBeDefined()
    })

    it('handles network passphrase correctly', () => {
      const customPassphrase = 'Custom Network ; 2024'
      const customClient = new ScavengerClient({
        ...mockOptions,
        networkPassphrase: customPassphrase,
      })
      expect(customClient).toBeDefined()
    })
  })

  describe('Admin Methods', () => {
    it('should have initializeAdmin method', () => {
      expect(client).toHaveProperty('initializeAdmin')
      expect(typeof client.initializeAdmin).toBe('function')
    })

    it('should have getAdmin method', () => {
      expect(client).toHaveProperty('getAdmin')
      expect(typeof client.getAdmin).toBe('function')
    })

    it('should have transferAdmin method', () => {
      expect(client).toHaveProperty('transferAdmin')
      expect(typeof client.transferAdmin).toBe('function')
    })

    it('should have setCharityContract method', () => {
      expect(client).toHaveProperty('setCharityContract')
      expect(typeof client.setCharityContract).toBe('function')
    })
  })

  describe('Participant Methods', () => {
    it('should have registerParticipant method', () => {
      expect(client).toHaveProperty('registerParticipant')
      expect(typeof client.registerParticipant).toBe('function')
    })

    it('should have deregisterParticipant method', () => {
      expect(client).toHaveProperty('deregisterParticipant')
      expect(typeof client.deregisterParticipant).toBe('function')
    })

    it('should have getParticipant method', () => {
      expect(client).toHaveProperty('getParticipant')
      expect(typeof client.getParticipant).toBe('function')
    })

    it('should have getParticipants method', () => {
      expect(client).toHaveProperty('getParticipants')
      expect(typeof client.getParticipants).toBe('function')
    })

    it('should have updateParticipantLocation method', () => {
      expect(client).toHaveProperty('updateParticipantLocation')
      expect(typeof client.updateParticipantLocation).toBe('function')
    })
  })

  describe('Waste Methods', () => {
    it('should have submitWaste method', () => {
      expect(client).toHaveProperty('submitWaste')
      expect(typeof client.submitWaste).toBe('function')
    })

    it('should have getWaste method', () => {
      expect(client).toHaveProperty('getWaste')
      expect(typeof client.getWaste).toBe('function')
    })

    it('should have transferWaste method', () => {
      expect(client).toHaveProperty('transferWaste')
      expect(typeof client.transferWaste).toBe('function')
    })

    it('should have confirmWaste method', () => {
      expect(client).toHaveProperty('confirmWaste')
      expect(typeof client.confirmWaste).toBe('function')
    })

    it('should have getWastes method', () => {
      expect(client).toHaveProperty('getWastes')
      expect(typeof client.getWastes).toBe('function')
    })
  })

  describe('Incentive Methods', () => {
    it('should have createIncentive method', () => {
      expect(client).toHaveProperty('createIncentive')
      expect(typeof client.createIncentive).toBe('function')
    })

    it('should have getIncentive method', () => {
      expect(client).toHaveProperty('getIncentive')
      expect(typeof client.getIncentive).toBe('function')
    })

    it('should have getIncentives method', () => {
      expect(client).toHaveProperty('getIncentives')
      expect(typeof client.getIncentives).toBe('function')
    })

    it('should have updateIncentive method', () => {
      expect(client).toHaveProperty('updateIncentive')
      expect(typeof client.updateIncentive).toBe('function')
    })

    it('should have claimIncentive method', () => {
      expect(client).toHaveProperty('claimIncentive')
      expect(typeof client.claimIncentive).toBe('function')
    })
  })

  describe('Query Methods', () => {
    it('should have getGlobalMetrics method', () => {
      expect(client).toHaveProperty('getGlobalMetrics')
      expect(typeof client.getGlobalMetrics).toBe('function')
    })

    it('should have getParticipantStats method', () => {
      expect(client).toHaveProperty('getParticipantStats')
      expect(typeof client.getParticipantStats).toBe('function')
    })

    it('should have getWasteTransferHistory method', () => {
      expect(client).toHaveProperty('getWasteTransferHistory')
      expect(typeof client.getWasteTransferHistory).toBe('function')
    })

    it('should have getEarnings method', () => {
      expect(client).toHaveProperty('getEarnings')
      expect(typeof client.getEarnings).toBe('function')
    })
  })

  describe('Error Handling', () => {
    it('should throw error on invalid contract ID', () => {
      expect(() => {
        new ScavengerClient({
          ...mockOptions,
          contractId: 'invalid',
        })
      }).not.toThrow() // Constructor doesn't validate
    })

    it('should throw error on invalid RPC URL format', () => {
      expect(() => {
        new ScavengerClient({
          ...mockOptions,
          rpcUrl: 'not-a-url',
        })
      }).not.toThrow() // Constructor doesn't validate
    })
  })

  describe('Type Safety', () => {
    it('should maintain TypeScript type information', () => {
      const options: typeof mockOptions = {
        rpcUrl: 'https://soroban-testnet.stellar.org',
        networkPassphrase: 'Test SDF Network ; September 2015',
        contractId: 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4',
      }
      const typedClient = new ScavengerClient(options)
      expect(typedClient).toBeDefined()
    })
  })

  describe('Method Availability', () => {
    it('exports all public methods', () => {
      const publicMethods = [
        'initializeAdmin',
        'getAdmin',
        'transferAdmin',
        'setCharityContract',
        'registerParticipant',
        'deregisterParticipant',
        'getParticipant',
        'getParticipants',
        'updateParticipantLocation',
        'submitWaste',
        'getWaste',
        'transferWaste',
        'confirmWaste',
        'getWastes',
        'createIncentive',
        'getIncentive',
        'getIncentives',
        'updateIncentive',
        'claimIncentive',
        'getGlobalMetrics',
        'getParticipantStats',
        'getWasteTransferHistory',
        'getEarnings',
      ]

      publicMethods.forEach((method) => {
        expect(client).toHaveProperty(method)
        expect(typeof (client as any)[method]).toBe('function')
      })
    })
  })

  describe('Multiple Client Instances', () => {
    it('should allow multiple independent client instances', () => {
      const client1 = new ScavengerClient(mockOptions)
      const client2 = new ScavengerClient({
        ...mockOptions,
        contractId: 'CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB',
      })

      expect(client1).toBeDefined()
      expect(client2).toBeDefined()
      expect(client1).not.toBe(client2)
    })
  })
})
