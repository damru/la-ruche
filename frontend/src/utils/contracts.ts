import type { ContractStatus, VendorContract } from '@/types/domain'

function todayIso(): string {
  const today = new Date()
  const year = today.getFullYear()
  const month = String(today.getMonth() + 1).padStart(2, '0')
  const day = String(today.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

export function effectiveContractStatus(contract: VendorContract): ContractStatus {
  if (contract.status === 'draft') return 'draft'
  if (contract.status === 'terminated') return 'terminated'
  if (contract.status === 'expired') return 'expired'
  if (contract.end_date && contract.end_date < todayIso()) return 'expired'
  return 'active'
}
