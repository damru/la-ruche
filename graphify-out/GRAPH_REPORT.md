# Graph Report - frontend  (2026-05-11)

## Corpus Check
- 149 files · ~68,130 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 251 nodes · 177 edges · 98 communities detected
- Extraction: 59% EXTRACTED · 41% INFERRED · 0% AMBIGUOUS · INFERRED: 73 edges (avg confidence: 0.76)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Route View Registry|Route View Registry]]
- [[_COMMUNITY_View Interaction Handlers|View Interaction Handlers]]
- [[_COMMUNITY_Frontend Entry And Theme|Frontend Entry And Theme]]
- [[_COMMUNITY_Access And File Icons|Access And File Icons]]
- [[_COMMUNITY_Admin And Status Icons|Admin And Status Icons]]
- [[_COMMUNITY_Modal Feedback Pattern|Modal Feedback Pattern]]
- [[_COMMUNITY_Tailwind Vite Setup|Tailwind Vite Setup]]
- [[_COMMUNITY_Schedule And Status Icons|Schedule And Status Icons]]
- [[_COMMUNITY_Document Finance Utility Icons|Document Finance Utility Icons]]
- [[_COMMUNITY_Property Location Icons|Property Location Icons]]
- [[_COMMUNITY_API Client Session|API Client Session]]
- [[_COMMUNITY_Confirmation Dialog State|Confirmation Dialog State]]
- [[_COMMUNITY_Resident Charge Status|Resident Charge Status]]
- [[_COMMUNITY_Navigation Arrow Icons|Navigation Arrow Icons]]
- [[_COMMUNITY_Vite Build Config|Vite Build Config]]
- [[_COMMUNITY_Assembly Detail View|Assembly Detail View]]
- [[_COMMUNITY_Theme Preference Store|Theme Preference Store]]
- [[_COMMUNITY_Theme And Loading Icons|Theme And Loading Icons]]
- [[_COMMUNITY_App Bootstrap Shell|App Bootstrap Shell]]
- [[_COMMUNITY_Chip Input Field|Chip Input Field]]
- [[_COMMUNITY_Flexible Layout Group|Flexible Layout Group]]
- [[_COMMUNITY_Auto Close Composable|Auto Close Composable]]
- [[_COMMUNITY_Formatting Helpers|Formatting Helpers]]
- [[_COMMUNITY_Form Feedback Composable|Form Feedback Composable]]
- [[_COMMUNITY_Role Access Composable|Role Access Composable]]
- [[_COMMUNITY_Dashboard Priority Sorting|Dashboard Priority Sorting]]
- [[_COMMUNITY_Maintenance View Screen|Maintenance View Screen]]
- [[_COMMUNITY_File Extension Helpers|File Extension Helpers]]
- [[_COMMUNITY_Energy And Water Icons|Energy And Water Icons]]
- [[_COMMUNITY_Overflow Indicator Icons|Overflow Indicator Icons]]
- [[_COMMUNITY_Info And Message Icons|Info And Message Icons]]
- [[_COMMUNITY_Add And Refresh Icons|Add And Refresh Icons]]
- [[_COMMUNITY_Search And Upload Icons|Search And Upload Icons]]
- [[_COMMUNITY_Frontend ESLint Config|Frontend ESLint Config]]
- [[_COMMUNITY_Vite Type Declarations|Vite Type Declarations]]
- [[_COMMUNITY_API Barrel Export|API Barrel Export]]
- [[_COMMUNITY_Alert Message Component|Alert Message Component]]
- [[_COMMUNITY_Status Badge Component|Status Badge Component]]
- [[_COMMUNITY_Button Action Component|Button Action Component]]
- [[_COMMUNITY_Eyebrow Label Component|Eyebrow Label Component]]
- [[_COMMUNITY_Chip Select Field|Chip Select Field]]
- [[_COMMUNITY_Form Feedback Component|Form Feedback Component]]
- [[_COMMUNITY_Form Input Field|Form Input Field]]
- [[_COMMUNITY_Form Select Field|Form Select Field]]
- [[_COMMUNITY_Form Textarea Field|Form Textarea Field]]
- [[_COMMUNITY_Legal Description Component|Legal Description Component]]
- [[_COMMUNITY_Mock Data Banner|Mock Data Banner]]
- [[_COMMUNITY_Progress Bar Component|Progress Bar Component]]
- [[_COMMUNITY_Section Separator Component|Section Separator Component]]
- [[_COMMUNITY_Tooltip Hint Component|Tooltip Hint Component]]
- [[_COMMUNITY_Card Surface Component|Card Surface Component]]
- [[_COMMUNITY_Card Note Component|Card Note Component]]
- [[_COMMUNITY_Data Table Component|Data Table Component]]
- [[_COMMUNITY_Address Autocomplete Field|Address Autocomplete Field]]
- [[_COMMUNITY_File Dropzone Component|File Dropzone Component]]
- [[_COMMUNITY_KPI Metric Card|KPI Metric Card]]
- [[_COMMUNITY_Legal Summary Card|Legal Summary Card]]
- [[_COMMUNITY_Legal Notice Component|Legal Notice Component]]
- [[_COMMUNITY_List Item Component|List Item Component]]
- [[_COMMUNITY_Meta Block Component|Meta Block Component]]
- [[_COMMUNITY_Meta Item Component|Meta Item Component]]
- [[_COMMUNITY_Meta Line Component|Meta Line Component]]
- [[_COMMUNITY_Modal Dialog Component|Modal Dialog Component]]
- [[_COMMUNITY_Page Loading Component|Page Loading Component]]
- [[_COMMUNITY_Application Navigation|Application Navigation]]
- [[_COMMUNITY_Auth Brand Panel|Auth Brand Panel]]
- [[_COMMUNITY_Confirmation Dialog Component|Confirmation Dialog Component]]
- [[_COMMUNITY_Document Link Action Modal|Document Link Action Modal]]
- [[_COMMUNITY_Linked Documents Modal|Linked Documents Modal]]
- [[_COMMUNITY_Page Error State|Page Error State]]
- [[_COMMUNITY_Reminder Card Component|Reminder Card Component]]
- [[_COMMUNITY_Top Bar Header|Top Bar Header]]
- [[_COMMUNITY_Assembly List View|Assembly List View]]
- [[_COMMUNITY_Invite Acceptance View|Invite Acceptance View]]
- [[_COMMUNITY_Login Authentication View|Login Authentication View]]
- [[_COMMUNITY_Budget Management View|Budget Management View]]
- [[_COMMUNITY_Claims Management View|Claims Management View]]
- [[_COMMUNITY_Contractor Overview Page|Contractor Overview Page]]
- [[_COMMUNITY_Contractor Detail View|Contractor Detail View]]
- [[_COMMUNITY_Contract Form Modal|Contract Form Modal]]
- [[_COMMUNITY_Contract Management View|Contract Management View]]
- [[_COMMUNITY_Document Management View|Document Management View]]
- [[_COMMUNITY_Onboarding Flow View|Onboarding Flow View]]
- [[_COMMUNITY_User Profile View|User Profile View]]
- [[_COMMUNITY_Reports Overview View|Reports Overview View]]
- [[_COMMUNITY_Residence Building View|Residence Building View]]
- [[_COMMUNITY_Resident Charges View|Resident Charges View]]
- [[_COMMUNITY_Task Management View|Task Management View]]
- [[_COMMUNITY_Internationalization Setup|Internationalization Setup]]
- [[_COMMUNITY_Application Layout Shell|Application Layout Shell]]
- [[_COMMUNITY_Mock Domain Data|Mock Domain Data]]
- [[_COMMUNITY_Authentication Store|Authentication Store]]
- [[_COMMUNITY_Auto Import Types|Auto Import Types]]
- [[_COMMUNITY_Component Type Declarations|Component Type Declarations]]
- [[_COMMUNITY_CSS Type Declarations|CSS Type Declarations]]
- [[_COMMUNITY_Domain Type Models|Domain Type Models]]
- [[_COMMUNITY_Forbidden Access View|Forbidden Access View]]
- [[_COMMUNITY_Not Found View|Not Found View]]

## God Nodes (most connected - your core abstractions)
1. `t()` - 12 edges
2. `DDR-006 — Modal Form Feedback Rule` - 9 edges
3. `index.html — App Entry Point` - 8 edges
4. `Frontend CLAUDE.md — Vue/Tailwind Instructions` - 7 edges
5. `toErrorMessage()` - 6 edges
6. `Tailwind CSS v4 — Vite Setup` - 6 edges
7. `submitVote()` - 5 edges
8. `reload()` - 4 edges
9. `DESIGN.md — Design System Tokens Reference` - 4 edges
10. `Lock Icon (Locked/Immutable)` - 4 edges

## Surprising Connections (you probably didn't know these)
- `submitVote()` --calls--> `respond()`  [INFERRED]
  frontend/src/features/consultations/ConsultationsView.vue → frontend/src/composables/useConfirm.ts
- `statusLabel()` --calls--> `t()`  [INFERRED]
  frontend/src/features/residents/UnitChargesView.vue → frontend/src/features/tasks/TaskDetailView.vue
- `forceModalClose()` --calls--> `t()`  [INFERRED]
  frontend/src/features/contractors/ContractorsView.vue → frontend/src/features/tasks/TaskDetailView.vue
- `forceKeyClose()` --calls--> `t()`  [INFERRED]
  frontend/src/features/residence/AllocationKeysView.vue → frontend/src/features/tasks/TaskDetailView.vue
- `unitTypeIcon()` --calls--> `t()`  [INFERRED]
  frontend/src/features/residents/RegistryView.vue → frontend/src/features/tasks/TaskDetailView.vue

## Communities

### Community 0 - "Route View Registry"
Cohesion: 0.07
Nodes (0): 

### Community 1 - "View Interaction Handlers"
Cohesion: 0.09
Nodes (12): forceKeyClose(), reload(), confirmClose(), confirmDelete(), saveConsultation(), submitVote(), forceModalClose(), buildDocumentDeleteMessage() (+4 more)

### Community 2 - "Frontend Entry And Theme"
Cohesion: 0.17
Nodes (15): Atomic Design System (Atoms > Molecules > Organisms), DESIGN.md — Design System Tokens Reference, Frontend CLAUDE.md — Vue/Tailwind Instructions, Root CLAUDE.md — Project-wide Rules, SVG Icons — Atomic Components in src/components/atoms/, #app — Vue Mount Point, Rationale: Dark mode applied before Vue render to prevent flash, Anti-Flash Dark Mode Inline Script (+7 more)

### Community 3 - "Access And File Icons"
Cohesion: 0.24
Nodes (11): Edit Icon, Eye Icon (Visibility On), Eye Off Icon (Visibility Off), File Icon, Folder Icon, Layers Icon (Stacked Layers), Lock Icon (Locked/Immutable), Lock Open Icon (Unlocked) (+3 more)

### Community 4 - "Admin And Status Icons"
Cohesion: 0.2
Nodes (11): Settings Icon, Shield Icon, Trash Icon, User Icon, User Plus Icon, Users Icon, Warning Icon, Wrench Icon (+3 more)

### Community 5 - "Modal Feedback Pattern"
Cohesion: 0.22
Nodes (9): AllocationKeysView.vue — Reference Implementation, CBtn Component, CFormFeedback Component, CModal Component, ContractFormModal.vue — Reference Implementation, ContractorsView.vue — Reference Implementation, DDR-006 — Modal Form Feedback Rule, Rationale: CFormFeedback must be in #footer slot, not modal body (+1 more)

### Community 6 - "Tailwind Vite Setup"
Cohesion: 0.33
Nodes (6): Rationale: Tailwind v4 CSS-first config eliminates postcss.config.js and tailwind.config.js, Tailwind CSS v4 — Vite Setup, @tailwindcss/vite Plugin, Tailwind CSS Docs — Installation/Vite, Tailwind CSS Upgrade Guide, vite.config.ts

### Community 7 - "Schedule And Status Icons"
Cohesion: 0.33
Nodes (6): Agora — Public Building / Assembly Hall, Calendar — Date / Scheduling, Check — Confirmation / Done, Check Circle — Success / Validated State, Clock — Time / Timestamp, Favicon — La Ruche App Logo (Hexagonal Grid)

### Community 8 - "Document Finance Utility Icons"
Cohesion: 0.4
Nodes (6): Archive — Box with Lid, Book — Documentation / Reading, Contract — Signed Document, Copy — Duplicate Content, Currency — Money / Financial Amount, Download — Save to Device

### Community 9 - "Property Location Icons"
Cohesion: 0.4
Nodes (5): Hard Hat Icon (Safety/Construction), Home Icon, Leaf Icon (Nature/Eco), Link Icon (Hyperlink/Chain), Location Icon (Map Pin)

### Community 10 - "API Client Session"
Cohesion: 0.67
Nodes (2): notifyQueue(), tryRefreshSession()

### Community 11 - "Confirmation Dialog State"
Cohesion: 0.5
Nodes (1): respond()

### Community 12 - "Resident Charge Status"
Cohesion: 0.5
Nodes (1): statusLabel()

### Community 13 - "Navigation Arrow Icons"
Cohesion: 0.67
Nodes (4): Arrow Left — Navigate Backward, Arrow Right — Navigate Forward, Chevron Down — Expand / Dropdown, Chevron Right — Collapse / Navigate Forward

### Community 14 - "Vite Build Config"
Cohesion: 0.67
Nodes (0): 

### Community 15 - "Assembly Detail View"
Cohesion: 0.67
Nodes (0): 

### Community 16 - "Theme Preference Store"
Cohesion: 0.67
Nodes (0): 

### Community 17 - "Theme And Loading Icons"
Cohesion: 0.67
Nodes (3): Sparkles Icon, Spinner Icon, Sun Icon

### Community 18 - "App Bootstrap Shell"
Cohesion: 1.0
Nodes (0): 

### Community 19 - "Chip Input Field"
Cohesion: 1.0
Nodes (0): 

### Community 20 - "Flexible Layout Group"
Cohesion: 1.0
Nodes (0): 

### Community 21 - "Auto Close Composable"
Cohesion: 1.0
Nodes (0): 

### Community 22 - "Formatting Helpers"
Cohesion: 1.0
Nodes (0): 

### Community 23 - "Form Feedback Composable"
Cohesion: 1.0
Nodes (0): 

### Community 24 - "Role Access Composable"
Cohesion: 1.0
Nodes (0): 

### Community 25 - "Dashboard Priority Sorting"
Cohesion: 1.0
Nodes (0): 

### Community 26 - "Maintenance View Screen"
Cohesion: 1.0
Nodes (0): 

### Community 27 - "File Extension Helpers"
Cohesion: 1.0
Nodes (0): 

### Community 28 - "Energy And Water Icons"
Cohesion: 1.0
Nodes (2): Bolt — Lightning / Quick Action, Droplet — Water / Fluid / Maintenance

### Community 29 - "Overflow Indicator Icons"
Cohesion: 1.0
Nodes (2): Dot — Status Indicator / Bullet Point, Dots Horizontal — More Options / Overflow Menu

### Community 30 - "Info And Message Icons"
Cohesion: 1.0
Nodes (2): Info Icon, Message Square Icon (Chat/Comment)

### Community 31 - "Add And Refresh Icons"
Cohesion: 1.0
Nodes (2): Plus Icon (Add/Create), Refresh Icon (Reload/Sync)

### Community 32 - "Search And Upload Icons"
Cohesion: 1.0
Nodes (2): Search Icon, Upload Icon

### Community 33 - "Frontend ESLint Config"
Cohesion: 1.0
Nodes (0): 

### Community 34 - "Vite Type Declarations"
Cohesion: 1.0
Nodes (0): 

### Community 35 - "API Barrel Export"
Cohesion: 1.0
Nodes (0): 

### Community 36 - "Alert Message Component"
Cohesion: 1.0
Nodes (0): 

### Community 37 - "Status Badge Component"
Cohesion: 1.0
Nodes (0): 

### Community 38 - "Button Action Component"
Cohesion: 1.0
Nodes (0): 

### Community 39 - "Eyebrow Label Component"
Cohesion: 1.0
Nodes (0): 

### Community 40 - "Chip Select Field"
Cohesion: 1.0
Nodes (0): 

### Community 41 - "Form Feedback Component"
Cohesion: 1.0
Nodes (0): 

### Community 42 - "Form Input Field"
Cohesion: 1.0
Nodes (0): 

### Community 43 - "Form Select Field"
Cohesion: 1.0
Nodes (0): 

### Community 44 - "Form Textarea Field"
Cohesion: 1.0
Nodes (0): 

### Community 45 - "Legal Description Component"
Cohesion: 1.0
Nodes (0): 

### Community 46 - "Mock Data Banner"
Cohesion: 1.0
Nodes (0): 

### Community 47 - "Progress Bar Component"
Cohesion: 1.0
Nodes (0): 

### Community 48 - "Section Separator Component"
Cohesion: 1.0
Nodes (0): 

### Community 49 - "Tooltip Hint Component"
Cohesion: 1.0
Nodes (0): 

### Community 50 - "Card Surface Component"
Cohesion: 1.0
Nodes (0): 

### Community 51 - "Card Note Component"
Cohesion: 1.0
Nodes (0): 

### Community 52 - "Data Table Component"
Cohesion: 1.0
Nodes (0): 

### Community 53 - "Address Autocomplete Field"
Cohesion: 1.0
Nodes (0): 

### Community 54 - "File Dropzone Component"
Cohesion: 1.0
Nodes (0): 

### Community 55 - "KPI Metric Card"
Cohesion: 1.0
Nodes (0): 

### Community 56 - "Legal Summary Card"
Cohesion: 1.0
Nodes (0): 

### Community 57 - "Legal Notice Component"
Cohesion: 1.0
Nodes (0): 

### Community 58 - "List Item Component"
Cohesion: 1.0
Nodes (0): 

### Community 59 - "Meta Block Component"
Cohesion: 1.0
Nodes (0): 

### Community 60 - "Meta Item Component"
Cohesion: 1.0
Nodes (0): 

### Community 61 - "Meta Line Component"
Cohesion: 1.0
Nodes (0): 

### Community 62 - "Modal Dialog Component"
Cohesion: 1.0
Nodes (0): 

### Community 63 - "Page Loading Component"
Cohesion: 1.0
Nodes (0): 

### Community 64 - "Application Navigation"
Cohesion: 1.0
Nodes (0): 

### Community 65 - "Auth Brand Panel"
Cohesion: 1.0
Nodes (0): 

### Community 66 - "Confirmation Dialog Component"
Cohesion: 1.0
Nodes (0): 

### Community 67 - "Document Link Action Modal"
Cohesion: 1.0
Nodes (0): 

### Community 68 - "Linked Documents Modal"
Cohesion: 1.0
Nodes (0): 

### Community 69 - "Page Error State"
Cohesion: 1.0
Nodes (0): 

### Community 70 - "Reminder Card Component"
Cohesion: 1.0
Nodes (0): 

### Community 71 - "Top Bar Header"
Cohesion: 1.0
Nodes (0): 

### Community 72 - "Assembly List View"
Cohesion: 1.0
Nodes (0): 

### Community 73 - "Invite Acceptance View"
Cohesion: 1.0
Nodes (0): 

### Community 74 - "Login Authentication View"
Cohesion: 1.0
Nodes (0): 

### Community 75 - "Budget Management View"
Cohesion: 1.0
Nodes (0): 

### Community 76 - "Claims Management View"
Cohesion: 1.0
Nodes (0): 

### Community 77 - "Contractor Overview Page"
Cohesion: 1.0
Nodes (0): 

### Community 78 - "Contractor Detail View"
Cohesion: 1.0
Nodes (0): 

### Community 79 - "Contract Form Modal"
Cohesion: 1.0
Nodes (0): 

### Community 80 - "Contract Management View"
Cohesion: 1.0
Nodes (0): 

### Community 81 - "Document Management View"
Cohesion: 1.0
Nodes (0): 

### Community 82 - "Onboarding Flow View"
Cohesion: 1.0
Nodes (0): 

### Community 83 - "User Profile View"
Cohesion: 1.0
Nodes (0): 

### Community 84 - "Reports Overview View"
Cohesion: 1.0
Nodes (0): 

### Community 85 - "Residence Building View"
Cohesion: 1.0
Nodes (0): 

### Community 86 - "Resident Charges View"
Cohesion: 1.0
Nodes (0): 

### Community 87 - "Task Management View"
Cohesion: 1.0
Nodes (0): 

### Community 88 - "Internationalization Setup"
Cohesion: 1.0
Nodes (0): 

### Community 89 - "Application Layout Shell"
Cohesion: 1.0
Nodes (0): 

### Community 90 - "Mock Domain Data"
Cohesion: 1.0
Nodes (0): 

### Community 91 - "Authentication Store"
Cohesion: 1.0
Nodes (0): 

### Community 92 - "Auto Import Types"
Cohesion: 1.0
Nodes (0): 

### Community 93 - "Component Type Declarations"
Cohesion: 1.0
Nodes (0): 

### Community 94 - "CSS Type Declarations"
Cohesion: 1.0
Nodes (0): 

### Community 95 - "Domain Type Models"
Cohesion: 1.0
Nodes (0): 

### Community 96 - "Forbidden Access View"
Cohesion: 1.0
Nodes (0): 

### Community 97 - "Not Found View"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **43 isolated node(s):** `Atomic Design System (Atoms > Molecules > Organisms)`, `CModal Component`, `CFormFeedback Component`, `CBtn Component`, `useFormFeedback() Composable` (+38 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `App Bootstrap Shell`** (2 nodes): `App.vue`, `main.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Chip Input Field`** (2 nodes): `onInput()`, `CFormChipInput.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Flexible Layout Group`** (2 nodes): `return()`, `CFlexGroup.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Auto Close Composable`** (2 nodes): `useAutoClose.ts`, `useAutoClose()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Formatting Helpers`** (2 nodes): `useFormatters.ts`, `useFormatters()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Form Feedback Composable`** (2 nodes): `useFormFeedback.ts`, `useFormFeedback()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Role Access Composable`** (2 nodes): `useRole.ts`, `useRole()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dashboard Priority Sorting`** (2 nodes): `priorityOrder()`, `DashboardView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Maintenance View Screen`** (2 nodes): `MaintenanceView.vue`, `t()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `File Extension Helpers`** (2 nodes): `extractFileExtension()`, `files.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Energy And Water Icons`** (2 nodes): `Bolt — Lightning / Quick Action`, `Droplet — Water / Fluid / Maintenance`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Overflow Indicator Icons`** (2 nodes): `Dot — Status Indicator / Bullet Point`, `Dots Horizontal — More Options / Overflow Menu`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Info And Message Icons`** (2 nodes): `Info Icon`, `Message Square Icon (Chat/Comment)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Add And Refresh Icons`** (2 nodes): `Plus Icon (Add/Create)`, `Refresh Icon (Reload/Sync)`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Search And Upload Icons`** (2 nodes): `Search Icon`, `Upload Icon`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Frontend ESLint Config`** (1 nodes): `eslint.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Vite Type Declarations`** (1 nodes): `vite-env.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `API Barrel Export`** (1 nodes): `index.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Alert Message Component`** (1 nodes): `CAlert.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Status Badge Component`** (1 nodes): `CBadge.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Button Action Component`** (1 nodes): `CBtn.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Eyebrow Label Component`** (1 nodes): `CEyebrow.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Chip Select Field`** (1 nodes): `CFormChipSelect.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Form Feedback Component`** (1 nodes): `CFormFeedback.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Form Input Field`** (1 nodes): `CFormInput.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Form Select Field`** (1 nodes): `CFormSelect.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Form Textarea Field`** (1 nodes): `CFormTextarea.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Legal Description Component`** (1 nodes): `CLegalDescription.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Mock Data Banner`** (1 nodes): `CMockBanner.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Progress Bar Component`** (1 nodes): `CProgressBar.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Section Separator Component`** (1 nodes): `CSeparator.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Tooltip Hint Component`** (1 nodes): `CTooltip.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Card Surface Component`** (1 nodes): `CCard.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Card Note Component`** (1 nodes): `CCardNote.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Data Table Component`** (1 nodes): `CDataTable.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Address Autocomplete Field`** (1 nodes): `CFormAddressAutocomplete.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `File Dropzone Component`** (1 nodes): `CFormDropzone.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `KPI Metric Card`** (1 nodes): `CKpiCard.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Legal Summary Card`** (1 nodes): `CLegalCard.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Legal Notice Component`** (1 nodes): `CLegalNotice.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `List Item Component`** (1 nodes): `CListItem.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Meta Block Component`** (1 nodes): `CMeta.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Meta Item Component`** (1 nodes): `CMetaItem.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Meta Line Component`** (1 nodes): `CMetaLine.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Modal Dialog Component`** (1 nodes): `CModal.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Page Loading Component`** (1 nodes): `CPageLoader.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Application Navigation`** (1 nodes): `AppNav.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Auth Brand Panel`** (1 nodes): `AuthBrandPanel.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Confirmation Dialog Component`** (1 nodes): `CConfirmDialog.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Document Link Action Modal`** (1 nodes): `CDocumentLinkActionModal.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Linked Documents Modal`** (1 nodes): `CLinkedDocumentsModal.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Page Error State`** (1 nodes): `CPageError.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Reminder Card Component`** (1 nodes): `CReminderCard.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Top Bar Header`** (1 nodes): `CTopbar.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Assembly List View`** (1 nodes): `AssemblyListView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Invite Acceptance View`** (1 nodes): `AcceptInviteView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Login Authentication View`** (1 nodes): `LoginView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Budget Management View`** (1 nodes): `BudgetView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Claims Management View`** (1 nodes): `ClaimsView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Contractor Overview Page`** (1 nodes): `ContractorPage.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Contractor Detail View`** (1 nodes): `ContractorDetailView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Contract Form Modal`** (1 nodes): `ContractFormModal.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Contract Management View`** (1 nodes): `ContractsView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Document Management View`** (1 nodes): `DocumentsView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Onboarding Flow View`** (1 nodes): `OnboardingView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `User Profile View`** (1 nodes): `ProfileView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Reports Overview View`** (1 nodes): `ReportsView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Residence Building View`** (1 nodes): `BuildingView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Resident Charges View`** (1 nodes): `MyChargesView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Task Management View`** (1 nodes): `TasksView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Internationalization Setup`** (1 nodes): `index.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Application Layout Shell`** (1 nodes): `AppLayout.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Mock Domain Data`** (1 nodes): `data.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Authentication Store`** (1 nodes): `auth.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Auto Import Types`** (1 nodes): `auto-imports.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Component Type Declarations`** (1 nodes): `components.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `CSS Type Declarations`** (1 nodes): `css.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Domain Type Models`** (1 nodes): `domain.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Forbidden Access View`** (1 nodes): `ForbiddenView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Not Found View`** (1 nodes): `NotFoundView.vue`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `t()` connect `View Interaction Handlers` to `Resident Charge Status`?**
  _High betweenness centrality (0.016) - this node is a cross-community bridge._
- **Why does `Frontend CLAUDE.md — Vue/Tailwind Instructions` connect `Frontend Entry And Theme` to `Modal Feedback Pattern`, `Tailwind Vite Setup`?**
  _High betweenness centrality (0.010) - this node is a cross-community bridge._
- **Why does `submitVote()` connect `View Interaction Handlers` to `Confirmation Dialog State`?**
  _High betweenness centrality (0.007) - this node is a cross-community bridge._
- **Are the 11 inferred relationships involving `t()` (e.g. with `reload()` and `saveConsultation()`) actually correct?**
  _`t()` has 11 INFERRED edges - model-reasoned connections that need verification._
- **Are the 5 inferred relationships involving `toErrorMessage()` (e.g. with `reload()` and `saveConsultation()`) actually correct?**
  _`toErrorMessage()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Atomic Design System (Atoms > Molecules > Organisms)`, `CModal Component`, `CFormFeedback Component` to the rest of the system?**
  _43 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Route View Registry` be split into smaller, more focused modules?**
  _Cohesion score 0.07 - nodes in this community are weakly interconnected._