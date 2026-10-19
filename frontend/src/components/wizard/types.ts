import { WasteType } from '@/api/types'
import { FieldErrors, UseFormRegister, UseFormSetValue } from 'react-hook-form'

export interface WasteSubmissionFormData {
  wasteType: WasteType
  weight: number
  latitude: string
  longitude: string
  notes: string
}

export interface TransferFormData {
  wasteId: string
  recipientAddress: string
  latitude: string
  longitude: string
  notes: string
}

export interface LocationStepProps {
  register: UseFormRegister<WasteSubmissionFormData | TransferFormData>
  errors: FieldErrors<WasteSubmissionFormData | TransferFormData>
  formData: WasteSubmissionFormData | TransferFormData
}

export interface WasteTypeStepProps {
  register: UseFormRegister<WasteSubmissionFormData>
  formData: WasteSubmissionFormData
  setValue: UseFormSetValue<WasteSubmissionFormData>
}

export interface WasteSelectionStepProps {
  formData: TransferFormData
  setValue: UseFormSetValue<TransferFormData>
}

export interface RecipientSelectionStepProps {
  formData: TransferFormData
  register: UseFormRegister<TransferFormData>
  errors: FieldErrors<TransferFormData>
}

export interface ReviewStepProps {
  formData: WasteSubmissionFormData | TransferFormData
}

export interface ReviewConfirmStepProps {
  formData: TransferFormData
}
