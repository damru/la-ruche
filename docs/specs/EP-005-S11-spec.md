# Spécification technique — EP-005-S11

**Story :** Visualiser les documents PDF générés (PV d'AG, relances d'appels de fonds, tout courrier réglementaire envoyé par voie postale et/ou électronique en pièce jointe) directement dans l'Application sans téléchargement préalable, dans le respect des droits d'accès par rôle.

**Épic :** EP-005 — Documents
**Réf. PRD :** US-DOC-003 (consultation des documents)
**Réf. LEGAL :** §3.4 (PV d'AG), §4.7 (relances / mise en demeure)
**Stories liées :** EP-003-S05, EP-013-S09 (PV d'AG), EP-004-S11 (relances d'appels de fonds), EP-002-S06/S07 (états datés), EP-021-S02 (courrier automatique, futur consommateur)
**Hors scope :** l'annotation et la signature dans le visualiseur, la génération des PDF eux-mêmes (portée par les modules producteurs), l'envoi postal/électronique des pièces jointes (EP-021-S02 et epic notifications), et le rendu des formats non PDF (images, bureautique) — le téléchargement existant (S05) reste la voie de repli.

---

## 1. Quoi (résumé fonctionnel)

Aujourd'hui, tout document PDF passe par le téléchargement sécurisé d'EP-005-S05 : l'utilisateur doit quitter l'Application pour lire le fichier. Les documents à valeur juridique ou comptable produits ou archivés par l'Application — PV d'AG (projet et signé), relances d'appels de fonds / mises en demeure, états datés, tout courrier réglementaire destiné à l'envoi postal et/ou électronique en pièce jointe — sont précisément ceux que le syndic et les copropriétaires doivent pouvoir relire vite et avec contexte.

Cible :
- Un visualiseur PDF intégré (rendu dans l'Application, sans téléchargement préalable), réutilisable par toutes les vues qui listent des documents PDF.
- Aucun contournement des droits d'accès : la visualisation emprunte exactement le même chemin de contrôle que le téléchargement (EP-005-S04 / S05). Un utilisateur autorisé à consulter peut visualiser ; un utilisateur non autorisé n'obtient ni le rendu ni le fichier.
- Une dégradation propre : si le rendu inline échoue (fichier volumineux, navigateur sans lecteur PDF intégré, mobile), le bouton Télécharger reste toujours disponible.

---

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
Téléchargement sécurisé (S05), disposition attachment | module documents du backend (endpoint de téléchargement) |
Contrôles de rôle sur la consultation/téléchargement (S04) | même module — garde existante à réutiliser à l'identique |
Listes de documents côté vues (download/retrait) | molecule CDocumentList (EP-020-S16, planifiée) et blocs équivalents des vues AG, charges, sinistres, contrats |

Les points d'ancrage exacts (handlers, composants) seront confirmés à l'implémentation ; cette spec fixe le contrat fonctionnel et les règles, pas les noms de fichiers.

---

## 3. Choix de conception

- Pas de nouvelle table, pas de nouveau flux de stockage : les PDF générés sont déjà des documents au sens du module (S02/S06/S07). Le viewer est une couche de présentation.
- Pas de logique métier dans les handlers (ADR-003) : la seule évolution backend est la variante « inline » du téléchargement sécurisé — mêmes gardes de permission, même document, seule la valeur de Content-Disposition change.
- Rendu natif navigateur (iframe/object sur l'URL sécurisée courte durée) en première intention : zéro dépendance, rendu correct sur desktop et mobile modernes. pdf.js (Apache-2.0, compatible AGPL-3.0) uniquement si un besoin réel de rendu uniforme apparaît — à justifier dans une story de suivi, pas ici.
- Le viewer ne reçoit jamais d'URL publique : il consomme le même accès signé/autorisé que le téléchargement, à TTL court, sans exposition du bucket (principe S05 conservé).

---

## 4. Backend

Étendre le téléchargement sécurisé existant avec une variante de rendu inline :
- Mêmes paramètres d'identification du document, mêmes contrôles de rôle et de visibilité que S04/S05 — aucune duplication de règle.
- Content-Disposition: inline (au lieu de attachment), avec nom de fichier encodé (RFC 5987) pour l'affichage d'onglet et l'impression.
- Content-Type: application/pdf uniquement ; tout autre type de document refuse la variante inline et retombe sur le comportement attachment existant.
- Pas de nouvelle route métier : la variante est une option de la route de téléchargement déjà exposée au frontend.

---

## 5. Frontend

- Nouveau composant molecule de visualisation (ex. CDocumentViewer) : rendu inline dans une modale ou un panneau, header avec titre du document, actions Télécharger et Fermer, état de chargement et état d'erreur avec repli explicite vers le téléchargement.
- Intégration prioritaire sur les vues qui portent des documents générés :
  - fiche AG : projet de PV et PV signé (EP-003-S05, EP-013-S09),
  - relances d'appels de fonds (EP-004-S11) : courriers de relance / mise en demeure,
  - états datés (EP-002-S06/S07),
  - puis généralisation progressive aux autres listes de documents (via CDocumentList une fois EP-020-S16 livrée).
- Styles uniquement via les tokens CSS var(--...) ; chaînes dans fr.json (CONTRIBUTING).

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Lien signé expiré pendant la consultation | Message clair + régénération de l'accès à la demande ; jamais d'URL longue durée |
| PDF volumineux / connexion lente | État de chargement ; pas de blocage de la vue hôte ; repli téléchargement |
| Navigateur sans rendu PDF intégré | Détection d'échec de rendu → repli automatique sur le bouton Télécharger |
| Document supprimé (soft-delete S08) | Même comportement que le téléchargement : non consultable, métadonnée conservée |
| Droits différenciés par rôle | Le viewer hérite strictement du verdict d'accès du téléchargement ; aucun chemin de visualisation séparé |

---

## 7. Critères d'acceptation

1. Un utilisateur autorisé peut visualiser un PDF généré (PV d'AG, relance, état daté) directement dans l'Application, sans téléchargement préalable.
2. Un utilisateur sans droit de consultation sur le document n'obtient aucun rendu ni fichier (même réponse refusée que pour le téléchargement).
3. La variante inline ne s'applique qu'aux documents PDF ; les autres types conservent le comportement de téléchargement existant.
4. Le bouton Télécharger reste disponible depuis le viewer.
5. Aucune URL publique du bucket n'est exposée ; l'accès reste signé/autorisé à TTL court.

---

## 8. Plan de tests

- Backend (e2e) : accès inline autorisé 200 avec Content-Disposition inline ; accès refusé pour un rôle sans droit (même code d'erreur que le téléchargement) ; type non-PDF retombe en attachment ; nom de fichier encodé correct.
- Frontend : lint + typecheck ; vérification manuelle desktop/mobile du rendu, du repli téléchargement et de l'état d'erreur.

---

## 9. Ordre d'implémentation

1. Variante inline du téléchargement sécurisé (backend) + tests e2e.
2. Composant de visualisation frontend + intégration fiche AG (PV).
3. Intégration relances (EP-004-S11) et états datés (EP-002-S06/S07).
4. Mise à jour de docs/EPICS.md (statut de la story et État code actuel de l'epic) et docs/DESIGN.md si nouveau composant catalogué.
