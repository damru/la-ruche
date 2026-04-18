# ADR-002 — Authentification

**Date :** 2026  
**Statut :** Accepté

---

## Contexte

Besoin d'une auth simple, sans admin technique, supportant passkeys et
mot de passe. Gestion des rôles par copropriété. Invitations par lien.

## Décisions

### Stratégie générale
Auth intégrée dans le binaire Rust. Pas de service externe (Keycloak rejeté
car trop lourd pour du self-hosted sans compétences DevOps).

### Mots de passe — Argon2id
- Standard recommandé par OWASP (2024)
- Résistant aux attaques GPU/ASIC
- Crate : `argon2`
- Paramètres : mémoire 64MB, itérations 3, parallélisme 1 (ajustable via config)

### Passkeys — webauthn-rs
- Implémentation WebAuthn Level 2
- Enregistrement après première connexion par mot de passe
- Credential stockée en JSONB dans `user_account.passkey_credential`
- Pas de dépendance à un authenticator spécifique (Touch ID, Windows Hello, clé FIDO2)
- **Statut implémentation** : `auth/passkey.rs` — `build_webauthn()` opérationnel ; 4 handlers de cérémonie (`register_start/finish`, `login_start/finish`) retournent `501 NOT_IMPLEMENTED`. Pas de roadmap d'activation définie.

### Sessions — JWT httpOnly
- Access token : JWT signé HS256, durée 15 minutes
- Refresh token : opaque (UUID hashé), stocké en DB, durée 30 jours
- Transport : cookie `httpOnly` + `Secure` + `SameSite=Strict`
- Pas de localStorage → protection XSS

### Rôles
- Claim `roles` dans le JWT : tableau des rôles pour la copropriété courante
- Vérification dans le middleware Axum via extracteur `AuthUser`
- Les rôles sont re-chargés depuis la DB au refresh du token (pas au access token)

### Tokens invitation / reset / prestataire
- UUID v4 généré côté backend
- Stocké hashé (SHA-256) en DB — jamais en clair
- Envoyé en clair dans le lien email ou URL prestataire
- TTL configurable (défaut : invitation 7j, reset 1h, prestataire 30j)

## Alternatives rejetées

| Alternative | Raison |
|---|---|
| Keycloak | JVM, complexité ops, overkill pour ce scope |
| Authentik | Service externe Python, contraire au self-hosted simple |
| Logto | Dépendance externe, moins de contrôle |
| Sessions serveur | Scalabilité horizontale plus complexe (sticky sessions ou Redis) |
| localStorage | Vulnérable XSS |

## Conséquences

- `JWT_SECRET` doit être une variable d'environnement forte (min 256 bits)
- Rotation du secret invalide tous les access tokens → refresh automatique côté client
- Le frontend doit gérer le refresh transparent (intercepteur HTTP)
- Pas de SSO inter-applications (hors scope)
