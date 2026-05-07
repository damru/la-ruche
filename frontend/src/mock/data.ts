/**
 * Données fictives pour le mode démonstration.
 * Aucune connexion au backend — à retirer avant la mise en production.
 */
import type {
  Building, Unit, User, Meeting, Resolution, VoteResult,
  Budget, BudgetLine, LevyCall, WorksFund,
  Document, VendorContract, ContractReminder,
  Task, Claim, MaintenanceEntry, Report, Consultation,
} from '@/types/domain'

// ── Bâtiment ─────────────────────────────────────────────────────
export const mockBuilding: Building = {
  id: 'bld-001',
  name: 'Résidence Les Acacias',
  address_line1: '12 allée des Acacias',
  address_line2: '',
  postal_code: '75015',
  city: 'Paris',
  manager_siret: '81234567800012',
  shares_total: 1000,
  bylaws_date: '2008-03-15',
  created_at: '2024-01-10T09:00:00Z',
}

export const mockBuildingStats = {
  name: 'Résidence Les Acacias',
  nb_lots: 24,
  works_fund_available: 18_400,
  works_fund_pct: 38,
}

// ── Utilisateurs ──────────────────────────────────────────────────
export const mockUsers: User[] = [
  { id: 'usr-001', email: 'sophie.martin@syndic-elite.fr', last_name: 'Martin', first_name: 'Sophie', active: true, roles: ['manager'], created_at: '2024-01-10T09:00:00Z' },
  { id: 'usr-002', email: 'jp.dupont@email.fr', last_name: 'Dupont', first_name: 'Jean-Pierre', active: true, roles: ['board_member', 'co_owner'], created_at: '2024-01-12T10:00:00Z' },
  { id: 'usr-003', email: 'marie.lefebvre@email.fr', last_name: 'Lefebvre', first_name: 'Marie', active: true, roles: ['co_owner'], created_at: '2024-01-15T11:00:00Z' },
  { id: 'usr-004', email: 'robert.cazeneuve@email.fr', last_name: 'Cazeneuve', first_name: 'Robert', active: true, roles: ['co_owner'], created_at: '2024-01-20T14:00:00Z' },
  { id: 'usr-005', email: 'fatima.bensalem@email.fr', last_name: 'Bensalem', first_name: 'Fatima', active: true, roles: ['co_owner'], created_at: '2024-02-01T09:30:00Z' },
  { id: 'usr-006', email: 'thomas.girard@email.fr', last_name: 'Girard', first_name: 'Thomas', active: true, roles: ['tenant'], created_at: '2024-03-05T16:00:00Z' },
  { id: 'usr-007', email: 'claire.renaud@email.fr', last_name: 'Renaud', first_name: 'Claire', active: true, roles: ['co_owner'], created_at: '2024-02-14T08:00:00Z' },
  { id: 'usr-008', email: 'michel.laborde@email.fr', last_name: 'Laborde', first_name: 'Michel', active: false, roles: ['co_owner'], created_at: '2024-01-18T10:00:00Z' },
]

export const mockAuthUser = {
  user_id: 'usr-001',
  building_id: 'bld-001',
  roles: ['manager'] as const,
  last_name: 'Martin',
  first_name: 'Sophie',
  email: 'sophie.martin@syndic-elite.fr',
}

// ── Lots ─────────────────────────────────────────────────────────
export const mockUnits: Unit[] = [
  { id: 'unt-001', building_id: 'bld-001', unit_number: '1', unit_type: 'apartment', staircase: 'A', floor: 'RDC', active: true, current_owner: mockUsers[2] },
  { id: 'unt-002', building_id: 'bld-001', unit_number: '2', unit_type: 'apartment', staircase: 'A', floor: '1er', active: true, current_owner: mockUsers[3] },
  { id: 'unt-003', building_id: 'bld-001', unit_number: '3', unit_type: 'apartment', staircase: 'A', floor: '2ème', active: true, current_owner: mockUsers[6] },
  { id: 'unt-004', building_id: 'bld-001', unit_number: '4', unit_type: 'apartment', staircase: 'A', floor: '3ème', active: true, current_owner: mockUsers[4] },
  { id: 'unt-005', building_id: 'bld-001', unit_number: '5', unit_type: 'apartment', staircase: 'B', floor: 'RDC', active: true, current_owner: mockUsers[1] },
  { id: 'unt-006', building_id: 'bld-001', unit_number: '6', unit_type: 'apartment', staircase: 'B', floor: '1er', active: true, current_owner: mockUsers[7] },
  { id: 'unt-007', building_id: 'bld-001', unit_number: '7', unit_type: 'apartment', staircase: 'B', floor: '2ème', active: true },
  { id: 'unt-008', building_id: 'bld-001', unit_number: '8', unit_type: 'apartment', staircase: 'B', floor: '3ème', active: true, current_owner: mockUsers[4] },
  { id: 'unt-009', building_id: 'bld-001', unit_number: 'Cave 1', unit_type: 'cellar', active: true, current_owner: mockUsers[2] },
  { id: 'unt-010', building_id: 'bld-001', unit_number: 'Cave 2', unit_type: 'cellar', active: true, current_owner: mockUsers[3] },
  { id: 'unt-011', building_id: 'bld-001', unit_number: 'Parking 1', unit_type: 'parking', active: true, current_owner: mockUsers[1] },
  { id: 'unt-012', building_id: 'bld-001', unit_number: 'Parking 2', unit_type: 'parking', active: true, current_owner: mockUsers[4] },
]

// ── Assemblées générales ──────────────────────────────────────────
export const mockMeetings: Meeting[] = [
  {
    id: 'mtg-001',
    meeting_type: 'annual',
    status: 'called',
    notice_date: '2025-05-10',
    meeting_date: '2025-06-12T18:00:00Z',
    location: 'Salle polyvalente, 14 allée des Acacias – Paris 15e',
    created_at: '2025-05-01T09:00:00Z',
    resolution_count: 4,
    passed_count: 0,
  },
  {
    id: 'mtg-002',
    meeting_type: 'annual',
    status: 'closed',
    notice_date: '2024-05-12',
    meeting_date: '2024-06-08T18:00:00Z',
    location: 'Salle polyvalente, 14 allée des Acacias – Paris 15e',
    created_at: '2024-05-02T09:00:00Z',
    resolution_count: 4,
    passed_count: 3,
  },
  {
    id: 'mtg-003',
    meeting_type: 'extraordinary',
    status: 'closed',
    notice_date: '2024-09-01',
    meeting_date: '2024-09-20T18:30:00Z',
    location: 'En ligne (visioconférence)',
    created_at: '2024-08-26T14:00:00Z',
    resolution_count: 2,
    passed_count: 2,
  },
]

export const mockResolutions: Resolution[] = [
  // AG 2025 (called - résolutions prévisionnelles)
  { id: 'res-001', meeting_id: 'mtg-001', number: 1, title: 'Approbation des comptes 2024', description: 'Approbation des comptes de gestion de l\'exercice 2024 présentés par le syndic.', vote_type: 'simple_majority', status: 'pending' },
  { id: 'res-002', meeting_id: 'mtg-001', number: 2, title: 'Approbation du budget prévisionnel 2025', description: 'Vote du budget prévisionnel de charges pour l\'exercice 2025, arrêté à 48 500 €.', vote_type: 'simple_majority', status: 'pending' },
  { id: 'res-003', meeting_id: 'mtg-001', number: 3, title: 'Ravalement de façade', description: 'Autorisation de travaux de ravalement de la façade côté rue. Devis Bâtisoins : 32 400 € TTC.', vote_type: 'absolute_majority', status: 'pending' },
  { id: 'res-004', meeting_id: 'mtg-001', number: 4, title: 'Renouvellement mandat du syndic', description: 'Renouvellement du contrat de syndic pour 3 ans avec Syndic Elite.', vote_type: 'simple_majority', status: 'pending' },
  // AG 2024 (closed)
  { id: 'res-005', meeting_id: 'mtg-002', number: 1, title: 'Approbation des comptes 2023', description: '', vote_type: 'simple_majority', status: 'passed' },
  { id: 'res-006', meeting_id: 'mtg-002', number: 2, title: 'Approbation du budget 2024', description: '', vote_type: 'simple_majority', status: 'passed' },
  { id: 'res-007', meeting_id: 'mtg-002', number: 3, title: 'Remplacement chaudière collective', description: 'Remplacement de la chaudière gaz par une pompe à chaleur air/eau. Devis : 18 200 €.', vote_type: 'absolute_majority', status: 'rejected' },
  { id: 'res-008', meeting_id: 'mtg-002', number: 4, title: 'Désignation conseil syndical', description: 'Élection des membres du conseil syndical pour 3 ans.', vote_type: 'simple_majority', status: 'passed' },
  // AGE 2024
  { id: 'res-009', meeting_id: 'mtg-003', number: 1, title: 'Travaux d\'urgence — toiture', description: 'Autorisation des travaux de réfection d\'urgence de la toiture suite aux infiltrations.', vote_type: 'simple_majority', status: 'passed' },
  { id: 'res-010', meeting_id: 'mtg-003', number: 2, title: 'Financement avance de trésorerie', description: 'Autorisation d\'une avance de trésorerie de 8 000 € sur le fonds de travaux.', vote_type: 'absolute_majority', status: 'passed' },
]

export const mockVoteResults: Record<string, VoteResult> = {
  'res-005': { adopted: true, votes_for: 840, votes_against: 0, abstentions: 80, shares_present: 920, total_shares: 1000, explanation: 'Majorité article 24' },
  'res-006': { adopted: true, votes_for: 780, votes_against: 60, abstentions: 80, shares_present: 920, total_shares: 1000, explanation: 'Majorité article 24' },
  'res-007': { adopted: false, votes_for: 320, votes_against: 500, abstentions: 100, shares_present: 920, total_shares: 1000, explanation: 'Majorité article 25 non atteinte' },
  'res-008': { adopted: true, votes_for: 900, votes_against: 20, abstentions: 0, shares_present: 920, total_shares: 1000, explanation: 'Majorité article 24' },
  'res-009': { adopted: true, votes_for: 850, votes_against: 40, abstentions: 30, shares_present: 920, total_shares: 1000, explanation: 'Majorité article 24' },
  'res-010': { adopted: true, votes_for: 780, votes_against: 100, abstentions: 40, shares_present: 920, total_shares: 1000, explanation: 'Majorité article 25' },
}

// ── Budget ────────────────────────────────────────────────────────
export const mockBudgets: Budget[] = [
  { id: 'bud-001', fiscal_year: 2025, status: 'approved', approval_meeting_id: 'mtg-002', created_at: '2025-01-01T00:00:00Z', total_forecast: 48_500, total_actual: 21_340 },
  { id: 'bud-002', fiscal_year: 2024, status: 'closed', approval_meeting_id: 'mtg-002', created_at: '2024-01-01T00:00:00Z', total_forecast: 44_200, total_actual: 43_800 },
]

export const mockBudgetLines: BudgetLine[] = [
  { id: 'bl-001', budget_id: 'bud-001', label: 'Entretien parties communes', key_id: 'k1', key_name: 'Clé générale', budgeted_amount: 8_000, actual_amount: 3_200 },
  { id: 'bl-002', budget_id: 'bud-001', label: 'Ascenseur – maintenance', key_id: 'k1', key_name: 'Clé générale', budgeted_amount: 5_400, actual_amount: 2_700 },
  { id: 'bl-003', budget_id: 'bud-001', label: 'Assurance immeuble', key_id: 'k1', key_name: 'Clé générale', budgeted_amount: 6_800, actual_amount: 6_800 },
  { id: 'bl-004', budget_id: 'bud-001', label: 'Nettoyage et gardiennage', key_id: 'k1', key_name: 'Clé générale', budgeted_amount: 9_200, actual_amount: 4_600 },
  { id: 'bl-005', budget_id: 'bud-001', label: 'Eau froide parties communes', key_id: 'k2', key_name: 'Eau commune', budgeted_amount: 3_600, actual_amount: 1_240 },
  { id: 'bl-006', budget_id: 'bud-001', label: 'Honoraires syndic', key_id: 'k1', key_name: 'Clé générale', budgeted_amount: 6_600, actual_amount: 2_800 },
  { id: 'bl-007', budget_id: 'bud-001', label: 'Électricité parties communes', key_id: 'k1', key_name: 'Clé générale', budgeted_amount: 2_800, actual_amount: 0 },
  { id: 'bl-008', budget_id: 'bud-001', label: 'Fonds de travaux (art. 14-2)', key_id: 'k1', key_name: 'Clé générale', budgeted_amount: 6_100, actual_amount: 0 },
]

export const mockLevies: LevyCall[] = [
  { id: 'lev-001', budget_id: 'bud-001', unit_id: 'unt-001', unit_number: 'Lot 1', owner_name: 'Lefebvre Marie', amount: 1_820, due_date: '2025-01-15', status: 'paid', issued_at: '2025-01-01T00:00:00Z', paid_at: '2025-01-12T00:00:00Z' },
  { id: 'lev-002', budget_id: 'bud-001', unit_id: 'unt-002', unit_number: 'Lot 2', owner_name: 'Cazeneuve Robert', amount: 2_100, due_date: '2025-01-15', status: 'paid', issued_at: '2025-01-01T00:00:00Z', paid_at: '2025-01-14T00:00:00Z' },
  { id: 'lev-003', budget_id: 'bud-001', unit_id: 'unt-003', unit_number: 'Lot 3', owner_name: 'Renaud Claire', amount: 1_650, due_date: '2025-01-15', status: 'overdue', issued_at: '2025-01-01T00:00:00Z' },
  { id: 'lev-004', budget_id: 'bud-001', unit_id: 'unt-004', unit_number: 'Lot 4', owner_name: 'Bensalem Fatima', amount: 1_950, due_date: '2025-04-15', status: 'issued', issued_at: '2025-04-01T00:00:00Z' },
  { id: 'lev-005', budget_id: 'bud-001', unit_id: 'unt-005', unit_number: 'Lot 5', owner_name: 'Dupont Jean-Pierre', amount: 2_250, due_date: '2025-04-15', status: 'paid', issued_at: '2025-04-01T00:00:00Z', paid_at: '2025-04-10T00:00:00Z' },
  { id: 'lev-006', budget_id: 'bud-001', unit_id: 'unt-006', unit_number: 'Lot 6', owner_name: 'Laborde Michel', amount: 1_780, due_date: '2025-01-15', status: 'overdue', issued_at: '2025-01-01T00:00:00Z' },
  { id: 'lev-007', budget_id: 'bud-001', unit_id: 'unt-007', unit_number: 'Lot 7', owner_name: '(vacant)', amount: 1_420, due_date: '2025-04-15', status: 'issued', issued_at: '2025-04-01T00:00:00Z' },
  { id: 'lev-008', budget_id: 'bud-001', unit_id: 'unt-008', unit_number: 'Lot 8', owner_name: 'Bensalem Fatima', amount: 1_900, due_date: '2025-04-15', status: 'paid', issued_at: '2025-04-01T00:00:00Z', paid_at: '2025-04-08T00:00:00Z' },
]

export const mockWorksFund: WorksFund = {
  id: 'wf-001',
  fiscal_year: 2025,
  levied_amount: 24_300,
  available_amount: 18_400,
}

// ── Documents ─────────────────────────────────────────────────────
export const mockDocuments: Document[] = [
  { id: 'doc-001', original_name: 'Convocation_AG_2025.pdf', mime_type: 'application/pdf', size_bytes: 245_000, visible_roles: ['manager', 'board_member', 'co_owner', 'tenant'], created_at: '2025-05-10T10:00:00Z' },
  { id: 'doc-002', original_name: 'PV_AG_2024.pdf', mime_type: 'application/pdf', size_bytes: 890_000, visible_roles: ['manager', 'board_member', 'co_owner'], created_at: '2024-07-01T14:00:00Z' },
  { id: 'doc-003', original_name: 'PV_AGE_Septembre_2024.pdf', mime_type: 'application/pdf', size_bytes: 540_000, visible_roles: ['manager', 'board_member', 'co_owner'], created_at: '2024-10-05T09:00:00Z' },
  { id: 'doc-004', original_name: 'Budget_previsionnel_2025.xlsx', mime_type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', size_bytes: 125_000, visible_roles: ['manager', 'board_member', 'co_owner'], created_at: '2025-01-15T11:00:00Z' },
  { id: 'doc-005', original_name: 'Comptes_2024_approuves.pdf', mime_type: 'application/pdf', size_bytes: 1_240_000, visible_roles: ['manager', 'board_member', 'co_owner'], created_at: '2025-01-10T10:00:00Z' },
  { id: 'doc-006', original_name: 'Contrat_Schindler_ascenseur.pdf', mime_type: 'application/pdf', size_bytes: 380_000, visible_roles: ['manager', 'board_member'], created_at: '2024-02-20T09:00:00Z' },
  { id: 'doc-007', original_name: 'Contrat_AXA_assurance_2025.pdf', mime_type: 'application/pdf', size_bytes: 520_000, visible_roles: ['manager'], created_at: '2025-01-05T10:00:00Z' },
  { id: 'doc-008', original_name: 'Reglement_de_copropriete.pdf', mime_type: 'application/pdf', size_bytes: 2_100_000, visible_roles: ['manager', 'board_member', 'co_owner', 'tenant'], created_at: '2024-01-10T09:00:00Z' },
  { id: 'doc-009', original_name: 'Devis_ravalement_Batisoins.pdf', mime_type: 'application/pdf', size_bytes: 640_000, visible_roles: ['manager', 'board_member', 'co_owner'], created_at: '2025-04-02T14:00:00Z' },
  { id: 'doc-010', original_name: 'Rapport_toiture_2024.pdf', mime_type: 'application/pdf', size_bytes: 920_000, visible_roles: ['manager', 'board_member'], created_at: '2024-08-15T11:00:00Z' },
]

// ── Tâches ────────────────────────────────────────────────────────
export const mockTasks: Task[] = [
  { id: 'tsk-001', created_by: 'usr-001', title: 'Réparation fuite toiture — Cage B', description: 'Infiltration d\'eau signalée au dernier étage, cage B. Devis reçu de Couvertures Dupré.', status: 'in_progress', priority: 'urgent', contract_id: undefined, created_at: '2025-04-10T08:00:00Z', updated_at: '2025-04-12T09:30:00Z', document_count: 2 },
  { id: 'tsk-002', created_by: 'usr-001', title: 'Remplacement éclairage cave — LED', description: 'Remplacement des tubes fluorescents par des LED dans les caves A et B.', status: 'open', priority: 'normal', created_at: '2025-03-22T09:00:00Z', updated_at: '2025-03-22T09:00:00Z', document_count: 0 },
  { id: 'tsk-003', created_by: 'usr-001', title: 'Réglage minuterie escalier A', description: 'La minuterie reste allumée en continu. Intervention électricien nécessaire.', status: 'awaiting_vendor', priority: 'high', created_at: '2025-04-05T10:00:00Z', updated_at: '2025-04-08T12:00:00Z', document_count: 1 },
  { id: 'tsk-004', created_by: 'usr-001', title: 'Peinture porte palière Lot 3', description: 'Porte palière à repeindre suite à des dégradations constatées.', status: 'open', priority: 'low', created_at: '2025-02-14T11:00:00Z', updated_at: '2025-02-14T11:00:00Z', document_count: 0 },
  { id: 'tsk-005', created_by: 'usr-001', title: 'Contrôle extincteurs annuel', description: 'Vérification annuelle des extincteurs — cages A et B.', status: 'closed', priority: 'normal', closed_at: '2025-03-15T14:00:00Z', created_at: '2025-02-01T09:00:00Z', updated_at: '2025-03-15T14:00:00Z', document_count: 1 },
  { id: 'tsk-006', created_by: 'usr-001', title: 'Débouchage regards pluviaux', description: 'Regards engorgés signalés lors des dernières pluies.', status: 'closed', priority: 'high', closed_at: '2025-04-08T16:00:00Z', created_at: '2025-03-28T08:00:00Z', updated_at: '2025-04-08T16:00:00Z', document_count: 3 },
  { id: 'tsk-007', created_by: 'usr-001', title: 'Révision chaudière parties communes', description: 'Révision annuelle de la chaudière collective avant l\'hiver.', status: 'cancelled', priority: 'normal', created_at: '2025-01-10T09:00:00Z', updated_at: '2025-01-12T10:00:00Z', document_count: 0 },
]

// ── Contrats ──────────────────────────────────────────────────────
export const mockContracts: VendorContract[] = [
  {
    id: 'ctr-001',
    vendor_name: 'Schindler France',
    vendor_email: 'contrats@schindler.fr',
    vendor_phone: '01 44 67 89 00',
    subject: 'Maintenance ascenseur — contrat Integral',
    contract_type: 'maintenance',
    start_date: '2023-01-01',
    end_date: '2025-12-31',
    amount: 4_200,
    payment_frequency: 'quarterly',
    auto_renewal: true,
    notice_days: 90,
    status: 'active',
    created_at: '2023-01-01T09:00:00Z',
  },
  {
    id: 'ctr-002',
    vendor_name: 'Propreté Plus',
    vendor_email: 'contact@proprete-plus.fr',
    vendor_phone: '01 45 23 11 22',
    subject: 'Nettoyage parties communes — hebdomadaire',
    contract_type: 'service',
    start_date: '2024-01-01',
    end_date: '2026-12-31',
    amount: 7_800,
    payment_frequency: 'monthly',
    auto_renewal: false,
    notice_days: 60,
    status: 'active',
    created_at: '2024-01-02T09:00:00Z',
  },
  {
    id: 'ctr-003',
    vendor_name: 'AXA Immeuble',
    vendor_email: 'sinistres@axa.fr',
    vendor_phone: '01 55 44 33 22',
    subject: 'Assurance multirisque immeuble',
    contract_type: 'insurance',
    start_date: '2025-01-01',
    end_date: '2025-12-31',
    amount: 6_800,
    payment_frequency: 'annual',
    auto_renewal: true,
    notice_days: 60,
    status: 'active',
    created_at: '2025-01-01T09:00:00Z',
  },
  {
    id: 'ctr-004',
    vendor_name: 'Veolia Eau Île-de-France',
    vendor_email: 'clients.idf@veolia.fr',
    vendor_phone: '09 69 32 11 10',
    subject: 'Fourniture eau froide parties communes',
    contract_type: 'service',
    start_date: '2022-06-01',
    end_date: '2022-12-31',
    amount: 3_600,
    payment_frequency: 'biannual',
    auto_renewal: true,
    notice_days: 30,
    status: 'expired',
    created_at: '2022-06-01T09:00:00Z',
  },
]

export const mockReminders: ContractReminder[] = [
  { id: 'rem-001', contract_id: 'ctr-001', reminder_type: 'renewal', reminder_date: '2025-10-03', status: 'pending' },
  { id: 'rem-002', contract_id: 'ctr-003', reminder_type: 'expiration', reminder_date: '2025-11-01', status: 'pending' },
  { id: 'rem-003', contract_id: 'ctr-002', reminder_type: 'termination', reminder_date: '2026-11-01', status: 'pending' },
]

// ── Sinistres ─────────────────────────────────────────────────────
export const mockClaims: Claim[] = [
  { id: 'clm-001', created_by: 'usr-001', title: 'Dégât des eaux — Lot 3 (fuite toiture)', description: 'Infiltration d\'eau dans le salon du lot 3 suite à une détérioration de la toiture. Déclaration AXA N° 2025-04-12345.', claim_type: 'current', declared_at: '2025-04-12', status: 'in_progress', unit_id: 'unt-003', legal_deadline_at: '2025-04-18', days_remaining: 3, is_deadline_approaching: false, is_deadline_missed: false, is_transmitted_late: false, document_count: 2, created_at: '2025-04-12T14:00:00Z', updated_at: '2025-04-13T09:00:00Z' },
  { id: 'clm-002', created_by: 'usr-001', title: 'Fissure façade — mur nord', description: 'Fissuration longitudinale constatée sur le mur nord, hauteur 1er–3ème étage. Expertise mandatée.', claim_type: 'natural_disaster', declared_at: '2025-02-20', published_at: '2025-02-24', status: 'open', legal_deadline_at: '2025-03-06', days_remaining: 1, is_deadline_approaching: true, is_deadline_missed: false, is_transmitted_late: false, document_count: 1, created_at: '2025-02-20T10:00:00Z', updated_at: '2025-02-24T10:00:00Z' },
  { id: 'clm-003', created_by: 'usr-001', title: 'Vol dans les caves (cage A)', description: 'Effraction des caves 1, 3 et 4 signalée le 12/11/2024. Plainte déposée. Sinistre déclaré à l\'assurance.', claim_type: 'theft', declared_at: '2024-11-13', status: 'closed', transmitted_at: '2024-11-18', legal_deadline_at: '2024-11-15', is_deadline_approaching: false, is_deadline_missed: false, is_transmitted_late: true, document_count: 3, created_at: '2024-11-13T09:00:00Z', updated_at: '2024-11-18T18:00:00Z' },
]

// ── Carnet d'entretien ────────────────────────────────────────────
export const mockMaintenanceEntries: MaintenanceEntry[] = [
  { id: 'mnt-001', title: 'Révision annuelle ascenseur', description: 'Contrôle complet par Schindler — rapport conforme, aucune anomalie.', intervention_date: '2025-03-10', cost: 1_050, vendor_name: 'Schindler France', contract_id: 'ctr-001', source: 'automatic', created_at: '2025-03-10T12:00:00Z' },
  { id: 'mnt-002', title: 'Débouchage regards pluviaux', description: 'Intervention suite au signalement de débordement. Regards nettoyés, grilles remplacées.', intervention_date: '2025-04-08', cost: 480, vendor_name: 'Plomberie Duval', source: 'manual', created_at: '2025-04-08T16:00:00Z' },
  { id: 'mnt-003', title: 'Contrôle extincteurs', description: '6 extincteurs vérifiés — 4 requalifiés, 2 remplacés.', intervention_date: '2025-03-15', cost: 340, vendor_name: 'Sécurité Services', source: 'manual', created_at: '2025-03-15T14:00:00Z' },
  { id: 'mnt-004', title: 'Réfection urgente toiture', description: 'Remplacement de 12 m² de zinguerie + pose d\'étanchéité provisoire suite à l\'AGE du 20/09/2024.', intervention_date: '2024-10-15', cost: 8_200, vendor_name: 'Couvertures Dupré', source: 'manual', created_at: '2024-10-15T17:00:00Z' },
  { id: 'mnt-005', title: 'Peinture cage A — parties communes', description: 'Ravalement intérieur cage A : hall, paliers, cage d\'escalier.', intervention_date: '2024-07-20', cost: 3_800, vendor_name: 'Peintres Associés', source: 'manual', created_at: '2024-07-20T10:00:00Z' },
  { id: 'mnt-006', title: 'Révision chaudière collective', description: 'Révision annuelle + nettoyage brûleurs. Rapport de conformité joint.', intervention_date: '2024-11-05', cost: 680, vendor_name: 'Thermique Expert', source: 'automatic', created_at: '2024-11-05T11:00:00Z' },
]

// ── Signalements résidents ────────────────────────────────────────
export const mockReports: Report[] = [
  { id: 'rpt-001', unit_id: 'unt-003', author_id: 'usr-007', author_first_name: 'Claire', author_last_name: 'Renaud', title: 'Tâche d\'humidité au plafond', description: 'Une tâche d\'humidité est apparue au plafond de ma chambre (lot 3). Semble liée à la fuite de toiture.', status: 'in_progress', document_count: 2, created_at: '2025-04-14T09:00:00Z', updated_at: '2025-04-14T09:00:00Z' },
  { id: 'rpt-002', unit_id: 'unt-005', author_id: 'usr-002', author_first_name: 'Jean-Pierre', author_last_name: 'Dupont', title: 'Éclairage couloir 2ème étage défaillant', description: 'La minuterie du couloir du 2ème étage cage B ne fonctionne plus. Couloir dans le noir.', status: 'open', document_count: 0, created_at: '2025-04-05T11:00:00Z', updated_at: '2025-04-05T11:00:00Z' },
  { id: 'rpt-003', unit_id: 'unt-001', author_id: 'usr-003', author_first_name: 'Marie', author_last_name: 'Lefebvre', title: 'Bruit de pompe la nuit', description: 'Depuis 3 semaines, une pompe dans la cave fait du bruit entre 23h et 6h du matin.', status: 'open', document_count: 1, created_at: '2025-03-28T22:00:00Z', updated_at: '2025-03-28T22:00:00Z' },
  { id: 'rpt-004', unit_id: 'unt-002', author_id: 'usr-004', author_first_name: 'Robert', author_last_name: 'Cazeneuve', title: 'Interphone en panne', description: 'L\'interphone de l\'entrée principale ne fonctionne plus depuis une semaine.', status: 'closed', document_count: 0, created_at: '2025-02-10T08:00:00Z', updated_at: '2025-02-15T10:00:00Z' },
]

// ── Consultations ─────────────────────────────────────────────────
export const mockConsultations: Consultation[] = [
  {
    id: 'con-001',
    title: 'Choix du prestataire pour le ravalement',
    description: 'Deux devis ont été reçus pour les travaux de ravalement de façade. Quel prestataire préférez-vous ?',
    status: 'open',
    visible_roles: ['manager', 'board_member', 'co_owner'],
    created_at: '2025-04-15T10:00:00Z',
    updated_at: '2025-04-15T10:00:00Z',
    my_choice_ids: [],
    total_responses: 0,
    choices: [
      { id: 'ch-001', label: 'Bâtisoins – 32 400 € TTC (délai 6 semaines)', order_index: 1, votes: 0, pct: 0 },
      { id: 'ch-002', label: 'Rénov\'Paris – 35 800 € TTC (délai 4 semaines)', order_index: 2, votes: 0, pct: 0 },
      { id: 'ch-003', label: 'Attendre l\'AG pour décision finale', order_index: 3, votes: 0, pct: 0 },
    ],
  },
  {
    id: 'con-002',
    title: 'Horaires d\'utilisation de la salle commune',
    description: 'La salle polyvalente peut être réservée par les résidents. Quelles plages horaires vous conviendraient ?',
    status: 'open',
    visible_roles: ['manager', 'board_member', 'co_owner', 'tenant'],
    created_at: '2025-03-20T14:00:00Z',
    updated_at: '2025-03-20T14:00:00Z',
    my_choice_ids: [],
    total_responses: 0,
    choices: [
      { id: 'ch-004', label: '8h–22h tous les jours', order_index: 1, votes: 0, pct: 0 },
      { id: 'ch-005', label: '9h–21h en semaine, 9h–23h le week-end', order_index: 2, votes: 0, pct: 0 },
      { id: 'ch-006', label: '10h–20h tous les jours', order_index: 3, votes: 0, pct: 0 },
    ],
  },
  {
    id: 'con-003',
    title: 'Installation de bornes de recharge VE',
    description: 'Envisagez-vous d\'équiper votre parking d\'une borne de recharge pour véhicule électrique ?',
    status: 'closed',
    visible_roles: ['manager', 'board_member', 'co_owner'],
    closed_at: '2025-02-28T23:59:00Z',
    created_at: '2025-02-01T09:00:00Z',
    updated_at: '2025-02-28T23:59:00Z',
    my_choice_ids: [],
    total_responses: 5,
    choices: [
      { id: 'ch-007', label: 'Oui, je suis intéressé(e)', order_index: 1, votes: 3, pct: 60 },
      { id: 'ch-008', label: 'Non, pas besoin pour l\'instant', order_index: 2, votes: 2, pct: 40 },
    ],
  },
]
