import { describe, it, expect } from 'vitest'
import { WasteType } from '@scavngr/types'
import { wasteSubmissionSchema, wasteTypeFieldSchema } from './wasteSubmission'

const validInput = {
  wasteType: WasteType.Plastic,
  weight: '250',
  latitude: '6.5',
  longitude: '3.4',
  notes: '',
}

describe('wasteTypeFieldSchema', () => {
  it('rejects the unselected placeholder', () => {
    const result = wasteTypeFieldSchema.safeParse('')
    expect(result.success).toBe(false)
  })

  it('accepts each shared waste type', () => {
    expect(wasteTypeFieldSchema.safeParse(WasteType.Metal).success).toBe(true)
  })
})

describe('wasteSubmissionSchema', () => {
  it('accepts a complete valid submission', () => {
    expect(wasteSubmissionSchema('grams').safeParse(validInput).success).toBe(true)
  })

  it('enforces the 1 gram minimum in grams', () => {
    const result = wasteSubmissionSchema('grams').safeParse({ ...validInput, weight: '0.5' })
    expect(result.success).toBe(false)
  })

  it('enforces the 0.001 kg minimum in kilograms', () => {
    const schema = wasteSubmissionSchema('kilograms')
    expect(schema.safeParse({ ...validInput, weight: '0.001' }).success).toBe(true)
    expect(schema.safeParse({ ...validInput, weight: '0.0005' }).success).toBe(false)
  })

  it('rejects out-of-range coordinates', () => {
    const result = wasteSubmissionSchema().safeParse({ ...validInput, latitude: '91' })
    expect(result.success).toBe(false)
  })
})
