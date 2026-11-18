import { type Locator, type Page } from '@playwright/test'
import { apiDelete, apiPost } from '../data/api'

/**
 * Page Object — registre des procédures judiciaires.
 * Route : /judicial-proceedings (canEdit : manager/board_member).
 */

export class JudicialProceedingsPage {
  readonly page: Page
  readonly modal: Locator

  constructor(page: Page) {
    this.page = page
    this.modal = page.locator('.c-modal__content')
  }

  async goto(): Promise<void> {
    await this.page.goto('/judicial-proceedings')
    await this.page.waitForLoadState('networkidle')
  }

  async openCreateModal(): Promise<void> {
    await this.page
      .locator(`.c-topbar button[title="Nouvelle procédure"]`)
      .first()
      .click()
    await this.modal.locator('#judicial-form input').first().waitFor()
  }

  private field(label: string): Locator {
    return this.modal
      .locator('#judicial-form .c-field')
      .filter({ has: this.page.locator('.form-label', { hasText: label }) })
      .locator('input')
      .first()
  }

  async fill(fields: {
    subject: string
    claimant: string
    defendant: string
    openedAt: string
  }): Promise<void> {
    console.log('FILL: modal visible =', await this.modal.isVisible())
    await this.field('Objet du litige').fill(fields.subject)
    await this.field('Demandeur').fill(fields.claimant)
    await this.field('Défendeur').fill(fields.defendant)
    await this.field("Date d'ouverture").fill(fields.openedAt)
    console.log('FILL done, modal visible =', await this.modal.isVisible())
  }

  async submit(): Promise<void> {
    await this.modal.locator('button[type="submit"]').click()
  }

  rowBySubject(subject: string): Locator {
    return this.page.locator('tr, .task-card, .c-card', { hasText: subject }).first()
  }

  async createViaApi(payload: {
    proceeding_type: string
    subject: string
    claimant: string
    defendant: string
    opened_at: string
  }): Promise<{ id: string }> {
    return apiPost(this.page, '/judicial-proceedings', payload)
  }

  async deleteViaApi(id: string): Promise<void> {
    await apiDelete(this.page, `/judicial-proceedings/${id}`)
  }
}
