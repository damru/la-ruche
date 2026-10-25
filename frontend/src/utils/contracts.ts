import type { ContractStatus, VendorContract } from '@/types/domain'
import { CONTRACT_STATUS_ACTIVE, CONTRACT_STATUS_DRAFT, CONTRACT_STATUS_EXPIRED, CONTRACT_STATUS_TERMINATED } from '@/utils/constants'

function todayIso(): string {
  const today = new Date()
  const year = today.getFullYear()
  const month = String(today.getMonth() + 1).padStart(2, '0')
  const day = String(today.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

export function effectiveContractStatus(contract: VendorContract): ContractStatus {
  if (contract.status === CONTRACT_STATUS_DRAFT) return CONTRACT_STATUS_DRAFT
  if (contract.status === CONTRACT_STATUS_TERMINATED) return CONTRACT_STATUS_TERMINATED
  if (contract.status === CONTRACT_STATUS_EXPIRED) return CONTRACT_STATUS_EXPIRED
  if (contract.end_date && contract.end_date < todayIso()) return CONTRACT_STATUS_EXPIRED
  return CONTRACT_STATUS_ACTIVE
}
