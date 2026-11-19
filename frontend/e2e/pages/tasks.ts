import { type Locator, type Page } from '@playwright/test'
import { apiDelete, apiPost } from '../data/api'

/**
 * Page Object — Tasks (liste + modale création/édition).
 * Expose des intentions ; aucun sélecteur brut hors de ce fichier.
 */

export class TasksPage {
  readonly page: Page
  readonly modal: Locator

  constructor(page: Page) {
    this.page = page
    this.modal = page.locator('.c-modal__content')
  }

  async goto(): Promise<void> {
    await this.page.goto('/tasks')
    await this.page.locator('.task-card').first().waitFor()
  }

  async openCreateModal(): Promise<void> {
    await this.page
      .locator('.c-topbar button[title="Nouvelle tâche"]')
      .first()
      .click()
    await this.modal.locator('#task-form input').first().waitFor()
  }

  async fillTitle(title: string): Promise<void> {
    await this.modal.locator('#task-form input').first().fill(title)
  }

  async fillDescription(description: string): Promise<void> {
    await this.modal.locator('#task-form textarea').first().fill(description)
  }

  async submit(): Promise<void> {
    await this.modal.locator('button[type="submit"]').click()
  }

  async cancel(): Promise<void> {
    await this.modal.locator('button.c-btn--ghost').click()
  }

  cardByTitle(title: string): Locator {
    return this.page.locator('.task-card', { has: this.page.locator('h3', { hasText: title }) })
  }

  cardByTitleExact(title: string): Locator {
    const escaped = title.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    return this.page.locator('.task-card', {
      has: this.page.locator('h3', { hasText: new RegExp(`^${escaped}$`) }),
    })
  }

  async createViaApi(title: string, description?: string): Promise<{ id: string; title: string }> {
    return apiPost(this.page, '/tasks', { title, description })
  }

  async deleteViaApi(id: string): Promise<void> {
    await apiDelete(this.page, `/tasks/${id}`)
  }
}
