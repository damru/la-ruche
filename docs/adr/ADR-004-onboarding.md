# ADR-004 — Onboarding First-Run Public

## Contexte

Au premier lancement de l'application, la base de données est vide (aucun utilisateur, aucune copropriété). Le flux de login existant exige qu'un utilisateur et un `user_building` existent déjà en base pour délivrer un JWT valide. Aucun mécanisme d'initialisation n'était prévu.

La cible est un self-hosting par un syndic bénévole, non-technique, qui déploie l'application via Docker Compose.

## Décision

Implémenter un onboarding first-run **public et minimal** accessible directement depuis l'interface web :

- **`GET /api/v1/onboarding/status`** : détecte si l'instance est vide (`COUNT(user_account) = 0`) et retourne `needs_onboarding: boolean`.
- **`POST /api/v1/onboarding/bootstrap`** : crée atomiquement (transaction PostgreSQL) le premier compte manager + la première copropriété + la liaison `user_building`, puis retourne une session authentifiée (cookies `access_token` + `refresh_token`).
- **Guard frontend** : au premier chargement, le router appelle `GET /onboarding/status` et redirige vers `/onboarding` si l'instance n'est pas initialisée. La route est bloquée dès qu'un utilisateur existe.
- **Guard backend** : le handler `bootstrap` revérifie le compteur d'utilisateurs à l'intérieur de la transaction. Si `COUNT > 0`, il retourne 409 Conflict.

### Périmètre de ce lot

- Inclus : compte manager (email, nom, prénom, mot de passe) + copropriété (nom, adresse, code postal, ville, tantièmes totaux).
- Exclu pour ce lot : lots, clés de répartition, invitations pendant l'onboarding (à traiter en lot 2).

### Requêtes SQL sans macro

Les requêtes du module `api/onboarding.rs` utilisent `sqlx::query` (runtime) plutôt que les macros `sqlx::query!` (compile-time) car l'ajout de nouvelles requêtes au cache offline sqlx (`cargo sqlx prepare`) nécessite une base de données live accessible au moment de la CI. Cette décision sera révisée lorsque la CI aura accès à une base de test persistante.

## Alternatives rejetées

| Option | Raison du rejet |
|---|---|
| Seed via script SQL / entrypoint Docker | Nécessite un accès SSH ou CLI ; non utilisable par un syndic bénévole sans compétences DevOps |
| Bootstrap token dans `.env` | Complexité supplémentaire pour un chemin unique ; distribuer un token est une surface d'attaque inutile |
| Inscription publique (`POST /auth/register`) | Requiert rate-limiting, CAPTCHA, vérification email — hors scope MVP self-hosted |

## Conséquences

- `auth.needsOnboarding` dans le store Pinia est initialisé à `null` et vérifié **une seule fois par session** dans le guard du router.
- Le guard est `async` (le router Vue Router supporte les guards async).
- La route `/onboarding` est publique mais bloquée après initialisation (redirection vers `/login` ou `/dashboard`).
- La contrainte UNIQUE sur `user_account.email` garantit l'idempotence en cas de race condition.
- Lot 2 à prévoir : wizard post-connexion pour créer les lots et les clés de répartition.
