# LAN Tournament Manager

Plateforme logicielle autonome (self-hosted) pour la gestion, l'arbitrage et le suivi en temps réel de tournois lors d'événements LAN (Local Area Network).

L'application est conçue pour fonctionner intégralement en réseau local fermé sans dépendance à des services tiers externes (absence d'OAuth externe, de bases managées obligatoires ou de CDN tiers requis). Elle intègre un moteur de règles et de barèmes configurable dynamiquement en base de données, permettant de réutiliser la même instance sur divers types de jeux (Duels, Battle Royale, formats par équipes) sans modification de code ni recompilation.

---

## 1. Architecture Technique

### Backend
- **Langage** : Rust (édition 2024).
- **Framework Web** : Axum 0.8 (reposant sur le runtime asynchrone Tokio).
- **Base de données** : MongoDB 7.0 (utilisant le pilote asynchrone officiel `mongodb`).
- **Sécurité et Authentification** :
  - Hachage cryptographique des mots de passe avec `bcrypt` (coût par défaut).
  - Gestion de sessions par jetons signés HMAC-SHA256 (`jsonwebtoken`) encapsulés dans des cookies HTTP sécurisés (`HttpOnly`, `SameSite=Lax`).
  - Contrôle d'accès basé sur les rôles (RBAC) via des extracteurs Axum (`AdminUser`, `CaptainUser`).
- **Diffusion Temps Réel** : WebSockets natifs Axum synchronisés via un canal de diffusion asynchrone (`tokio::sync::broadcast`).
- **Limitation de charge** : Couche de limitation de taille de requête fixée à 5 Mo (`DefaultBodyLimit`) pour la réception des captures d'écran de vérification.

### Frontend
- Architecture statique servie directement par le binaire Rust (`tower-http::services::ServeDir`).
- Interface utilisateur responsive conçue en HTML5, Tailwind CSS et JavaScript standard (sans dépendance lourde de framework client).
- Prise en charge du collage direct depuis le presse-papier (`Ctrl + V`) pour les captures d'écran et compression Canvas côté client avant téléversement.

---

## 2. Fonctionnalités Principales

### Assistant d'Initialisation (First-Run Setup)
- Détection automatique de l'absence de compte administrateur à l'initialisation de la base de données.
- Formulaire dédié au premier démarrage permettant de créer l'administrateur initial.
- Verrouillage automatique de la route d'initialisation dès qu'un administrateur existe dans le système.

### Moteur de Jeux et de Barèmes Dynamiques
- Configuration complète de chaque jeu en base de données :
  - Format de jeu (`duel`, `brawl`).
  - Barème de points par classement (vecteur ordonné de points par position).
  - Valeur par défaut pour les classements hors barème standard.
  - Statistiques secondaires personnalisables (éliminations, dégâts, manches gagnées) avec coefficient multiplicateur de points.
  - Bornes de validation et seuils anti-triche stricts (position maximale, valeur maximale de la statistique secondaire).
  - Consignes et règles spécifiques du jeu (cartes autorisées, outils tiers acceptés).

### Gestion des Équipes et des Capitaines
- Création d'équipes avec génération d'un compte capitaine dédié (identifiant et mot de passe chiffré).
- Suivi du total de points cumulés par équipe.
- Index d'unicité garantissant l'absence de collision entre pseudos d'équipes et comptes d'administration.

### Cycle de Vie des Matchs et Lobbies
- Définition des matchs avec rattachement à un jeu du catalogue.
- Assignation explicite des équipes participantes.
- Transmission des instructions en direct aux équipes convoquées (codes de lobby, serveurs, mots de passe).
- Contrôle d'ouverture et de fermeture des matchs aux soumissions.

### Soumission et Arbitrage des Scores
- Soumission par le capitaine d'équipe avec saisie de son résultat et transmission de la capture d'écran faisant foi.
- Mode de saisie groupée pour les administrateurs permettant d'enregistrer instantanément les scores complets d'un lobby (particulièrement adapté aux Battle Royale / Brawls multi-équipes).
- Interface d'arbitrage pour validation ou rejet manuel des soumissions en attente avec calcul automatique des points selon le barème du jeu.
- Fonction d'ajustement direct des points d'une équipe par les administrateurs pour la gestion des pénalités ou des corrections d'arbitrage.

### Écran Géant et Diffusion en Temps Réel
- Affichage public du classement général, actualisé instantanément dès qu'un score est validé via les événements WebSocket (`LEADERBOARD_UPDATE`, `NEW_SUBMISSION`).
- Affichage public des matchs actuellement ouverts et des jeux associés.

### Outils de Réinitialisation
- Procédure de remise à zéro configurable selon trois périmètres :
  - Réinitialisation des scores uniquement (maintien des équipes, comptes et catalogue de jeux).
  - Réinitialisation des utilisateurs et des équipes (maintien des administrateurs et des jeux).
  - Réinitialisation complète du tournoi (vidage intégral hors compte administrateur initial).

---

## 3. Prérequis Système

Pour compiler et exécuter l'application, les outils suivants sont requis :

- **Rust** (version stable 1.80 ou supérieure, avec `cargo`).
- **Docker** et **Docker Compose** (pour l'exécution locale de l'instance MongoDB).
- Optionnellement : un client MongoDB (`mongosh`, MongoDB Compass) pour l'inspection des données.

---

## 4. Installation et Démarrage Local

### Étape 1 : Cloner le dépôt
```bash
git clone https://github.com/votre-utilisateur/Lan_tournament_manager.git
cd Lan_tournament_manager
```

### Étape 2 : Configurer les variables d'environnement
Créer un fichier `.env` à la racine du projet en se basant sur le modèle `.env.example` :
```dotenv
# Clé secrète obligatoire pour la signature des jetons de session JWT
JWT_SECRET=remplacer_par_une_chaine_aleatoire_tres_longue_et_securisee

# URI de connexion à la base MongoDB (en local avec Docker Compose)
MONGODB_ADDON_URI=mongodb://localhost:27017/lan_tournament

# Port d'écoute du serveur HTTP (optionnel, 8080 par défaut)
PORT=8080
```

### Étape 3 : Démarrer la base de données MongoDB
Lancer le conteneur MongoDB à l'aide de Docker Compose :
```bash
docker compose up -d
```

Pour vérifier que la base est opérationnelle :
```bash
docker ps
```

### Étape 4 : Lancer le serveur d'application
Exécuter le serveur en mode développement :
```bash
cargo run
```

Le serveur écoute par défaut sur `http://localhost:8080`.

Pour une compilation optimisée pour la production :
```bash
cargo build --release
./target/release/Lan_tournament_manager
```

---

## 5. Répertoire des Points de Terminaison d'API (Endpoints)

### Santé et Statut Système
| Méthode | Chemin | Description | Accès |
|---|---|---|---|
| `GET` | `/api/health` | Vérification de l'état du serveur et connectivité MongoDB | Public |
| `GET` | `/ws` | Canal WebSocket pour la diffusion des événements temps réel | Public |

### Authentification et Sessions
| Méthode | Chemin | Description | Accès |
|---|---|---|---|
| `GET` | `/api/auth/setup-status` | Indique si le premier compte administrateur doit être créé | Public |
| `POST` | `/api/auth/setup` | Création du premier compte administrateur | Public (verrouillé après 1er usage) |
| `POST` | `/api/auth/login` | Authentification (Administrateur ou Capitaine) | Public |
| `GET` | `/api/auth/me` | Récupération de l'identité et du rôle de l'utilisateur connecté | Authentifié |
| `POST` | `/api/auth/logout` | Déconnexion et révocation du cookie de session | Authentifié |

### Consultation Publique
| Méthode | Chemin | Description | Accès |
|---|---|---|---|
| `GET` | `/api/leaderboard` | Classement complet des équipes ordonné par points | Public |
| `GET` | `/api/games` | Catalogue complet des jeux et de leurs barèmes | Public |
| `GET` | `/api/matches` | Liste des matchs ouverts aux inscriptions/soumissions | Public |

### Espace Capitaine
| Méthode | Chemin | Description | Accès |
|---|---|---|---|
| `GET` | `/api/captain/matches` | Matchs ouverts assignés à l'équipe du capitaine | Capitaine |
| `POST` | `/api/captain/submissions` | Soumission du résultat d'un match avec capture d'écran | Capitaine |

### Administration Générale
| Méthode | Chemin | Description | Accès |
|---|---|---|---|
| `GET` | `/api/admin/admins` | Liste des administrateurs enregistrés | Administrateur |
| `POST` | `/api/admin/admins` | Création d'un administrateur secondaire | Administrateur |
| `DELETE` | `/api/admin/admins/{id}` | Suppression d'un administrateur (refusée s'il s'agit du dernier) | Administrateur |
| `POST` | `/api/admin/games` | Création d'un jeu avec configuration de barème | Administrateur |
| `DELETE` | `/api/admin/games/{id}` | Suppression d'un jeu du catalogue | Administrateur |
| `POST` | `/api/admin/teams` | Inscription d'une équipe et génération du compte capitaine | Administrateur |
| `DELETE` | `/api/admin/teams/{id}` | Suppression d'une équipe | Administrateur |
| `PATCH` | `/api/admin/teams/{id}/points` | Ajustement direct du total de points d'une équipe | Administrateur |
| `GET` | `/api/admin/matches` | Liste exhaustive de tous les matchs créés | Administrateur |
| `POST` | `/api/admin/matches` | Création d'un match avec assignation d'équipes | Administrateur |
| `PATCH` | `/api/admin/matches/{id}/instructions` | Mise à jour des consignes et codes de lobby en direct | Administrateur |
| `POST` | `/api/admin/matches/{id}/toggle` | Ouverture ou fermeture d'un match aux soumissions | Administrateur |
| `POST` | `/api/admin/matches/{id}/scores` | Saisie groupée directe des scores d'un match | Administrateur |
| `GET` | `/api/admin/submissions` | Liste des soumissions de score en attente de validation | Administrateur |
| `POST` | `/api/admin/submissions/{id}/approve` | Validation d'une soumission et attribution des points | Administrateur |
| `POST` | `/api/admin/submissions/{id}/reject` | Rejet d'une soumission non conforme | Administrateur |
| `POST` | `/api/admin/reset` | Réinitialisation du tournoi selon le périmètre spécifié | Administrateur |

---

## 6. Structure du Projet

```text
Lan_tournament_manager/
├── Cargo.toml            # Déclaration des dépendances et métadonnées du crate
├── Cargo.lock            # Arbre résolu et verrouillé des dépendances
├── .env.example          # Gabarit des variables d'environnement
├── docker-compose.yml    # Service de base de données MongoDB local
├── deploiement.md        # Guide d'exploitation et procédures de déploiement
├── README.md             # Documentation générale du projet
├── src/
│   ├── main.rs           # Point d'entrée, initialisation Tokio, MongoDB et Axum
│   ├── errors.rs         # Centralisation des types d'erreurs et mapping HTTP Axum
│   ├── ws.rs             # Gestionnaire de connexions WebSockets
│   ├── models/           # Définitions des structures de données et des DTOs
│   │   ├── mod.rs
│   │   ├── models.rs     # Modèles MongoDB (Game, Match, Team, Admin, Submission)
│   │   └── dtos.rs       # Payloads de requêtes JSON désérialisées
│   ├── db/               # Couche d'accès aux données MongoDB et indexation
│   │   ├── mod.rs
│   │   ├── admins.rs     # Opérations de persistance relatives aux administrateurs
│   │   ├── games.rs      # Opérations relatives au catalogue de jeux
│   │   ├── matches.rs    # Opérations de gestion des matchs
│   │   ├── submissions.rs# Persistance et cycle de vie des soumissions
│   │   ├── teams.rs      # Gestion des équipes et calculs de points
│   │   └── reset.rs      # Procédures de réinitialisation de la base
│   ├── auth/             # Moteur d'authentification et de sécurité
│   │   ├── mod.rs
│   │   ├── claims.rs     # Structure du jeton de session JWT
│   │   ├── utils.rs      # Hachage bcrypt, génération et validation des cookies
│   │   └── extractors.rs # Gardes d'accès Axum (AdminUser, CaptainUser)
│   └── routes/           # Découpage et handlers de l'API HTTP
│       ├── mod.rs
│       ├── router.rs     # Assemblage centralisé des sous-routeurs Axum
│       ├── auth.rs       # Routes d'authentification et d'initialisation
│       ├── public.rs     # Routes publiques (classement, jeux, santé)
│       ├── captain.rs    # Routes réservées aux capitaines
│       └── admin/        # Modules de l'espace d'administration
│           ├── mod.rs
│           ├── admins.rs
│           ├── games.rs
│           ├── matches.rs
│           ├── reset.rs
│           ├── scores.rs
│           └── teams.rs
└── static/               # Fichiers d'interface statique distribués au client
    ├── index.html        # Vue publique et écran géant de projection
    ├── login.html        # Portail de connexion et premier lancement
    ├── captain.html      # Tableau de bord capitaine et envoi de captures
    ├── admin.html        # Console complète d'administration
    └── js/               # Scripts JavaScript clients
```

---

## 7. Exploitation et Déploiement

Pour les détails relatifs au déploiement en production, à la sauvegarde ou à la migration des données MongoDB (notamment vers un hébergeur cloud tel que Clever Cloud), consulter le guide dédié dans [`deploiement.md`](deploiement.md).
