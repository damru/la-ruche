import type { Document } from '@/types/domain'

type Translate = (key: string, params?: Record<string, string>) => string

export function buildDocumentDeleteMessage(
  document: Document | undefined,
  t: Translate,
): string {
  if (!document) return t('documents.confirmDeleteMessage', { name: '' })

  const linkedContractNames = document.linked_contract_names ?? []

  if (linkedContractNames.length === 0) {
    return t('documents.confirmDeleteMessage', { name: document.original_name })
  }

  return t('documents.confirmDeleteLinkedMessage', {
    name: document.original_name,
    links: linkedContractNames.join(', '),
  })
}
