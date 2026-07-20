# AsterQuanta OS — distribution Linux installable (Yocto)

Ceci est un **vrai projet Yocto**, buildable en une commande, produisant une
**image disque bootable réelle** (partitions réelles, noyau Linux réel,
systemd réel, GUI réelle au boot, mise à jour A/B réelle via RAUC, réseau
DTN réel via uD3TN). Rien ici n'est simulé.

## Pourquoi Yocto (et pas autre chose)

Yocto est le bon choix pour ce projet, et je le confirme plutôt que de le
remplacer :

- c'est le standard industriel pour les OS embarqués reproductibles
  (automobile, aérospatial, télécoms — exactement le profil "environnement
  à connectivité intermittente, mission critique" que tu décris) ;
- il produit une image **reproductible bit-à-bit** à partir de recettes
  versionnées (pas un script d'installation ad hoc) ;
- il a un écosystème mature pour tout ce que tu demandes : `meta-rauc`
  (update A/B + recovery), `meta-openembedded` (GTK, Wayland/Cage), gestion
  fine des paquets (`opkg`/`rpm`/`ipk` au choix), `dm-verity`/secure boot
  via `meta-security`.

Alternative honnête si tu changes d'avis plus tard : **Buildroot** est plus
simple à apprendre pour un dev solo (moins de couches, build plus rapide)
mais moins outillé pour l'A/B/reproductibilité industrielle. Recommandation
: rester sur Yocto, mais piloté par **`kas`** (outil de la fondation Yocto)
pour cacher la complexité de configuration — c'est ce que ce projet fait.

## Ce que tu ne dois PAS faire à la main

- Ne pas cloner poky/meta-openembedded/meta-rauc toi-même : `kas-asterquanta.yml`
  le fait pour toi.
- Ne pas éditer `local.conf`/`bblayers.conf` à la main : `kas` les génère.
- Ne pas installer de dépendances sur la machine finale : tout est **dans
  l'image** (noyau + rootfs + services + GUI), l'utilisateur flashe une
  image et démarre, point.

## Prérequis pour builder (sur ta machine ou en CI, pas dans ce chat)

- Linux x86_64 (Ubuntu 22.04/24.04 recommandé), 100+ Go d'espace disque,
  8+ Go de RAM, `git`, `python3`, `docker` (kas peut builder en conteneur).
- Installer kas : `pip3 install kas`
- Builder : `kas build kas-asterquanta.yml`
- Tester sans matériel : `runqemu asterquanta-image asterquanta-qemux86-64`
- Premier build : compte 2 à 5 heures selon la machine (c'est normal, c'est
  un vrai noyau + une vraie distribution qui compilent).

## 1. Architecture finale — couches logicielles

```
┌───────────────────────────────────────────────────────────────────┐
│ L6  Interface graphique      aqm-ui (GTK4)  sur  Cage (kiosk Wayland)│
├───────────────────────────────────────────────────────────────────┤
│ L5  Espace utilisateur AQM   aqm-supervisor · aqm-shell (aqmctl)    │
│                              aqm-dtnd (uD3TN) · aqm-recoveryd       │
├───────────────────────────────────────────────────────────────────┤
│ L4  Services système         systemd (init, units, journald, logind)│
│                              RAUC (update/rollback A/B)              │
│                              opkg/rpm (gestionnaire de paquets)      │
├───────────────────────────────────────────────────────────────────┤
│ L3  Userland minimal         BusyBox/coreutils, glibc, bash, util-  │
│                              linux, dbus, PAM (utilisateurs locaux) │
├───────────────────────────────────────────────────────────────────┤
│ L2  Bootloader               GRUB (x86_64) / U-Boot (ARM), env A/B  │
├───────────────────────────────────────────────────────────────────┤
│ L1  Noyau Linux              kernel standard (branche poky, config  │
│                              defconfig + fragments AQM)              │
├───────────────────────────────────────────────────────────────────┤
│ L0  Matériel / QEMU          x86_64 (qemux86-64) en v1, portable    │
│                              vers ARM (Raspberry Pi, i.MX) en v2    │
└───────────────────────────────────────────────────────────────────┘
```

## 2. Services à créer (systemd units réels)

| Service            | Rôle                                             | Techno         |
|--------------------|--------------------------------------------------|----------------|
| `aqm-supervisor`   | supervise les autres services via systemd/dbus, calcule le score de santé | Rust (std) |
| `aqm-dtnd`         | nœud DTN (bundle protocol), file de priorité, contact réseau intermittent | uD3TN (C, upstream) |
| `aqm-recoveryd`    | snapshots BTRFS, bascule recovery, mode sûr      | Rust + btrfs-progs |
| `aqm-ui`           | interface graphique kiosque au démarrage          | GTK4 + Cage |
| `aqm-shell` (`aqmctl`) | CLI d'administration (status, services, update, dtn, logs) | Rust (std) |
| `rauc`             | update/rollback A/B (réel, upstream, pas custom) | RAUC (C, upstream) |
| `systemd-journald` | logs structurés persistants                      | systemd (upstream) |

## 3. Structure du projet (réelle, sur disque dans ce dossier)

```
asterquanta-os-yocto/
├── kas-asterquanta.yml              # point d'entrée unique du build
├── README.md
└── meta-asterquanta/
    ├── conf/
    │   ├── layer.conf
    │   ├── distro/asterquanta.conf       # distro: systemd, wayland, rauc
    │   └── machine/asterquanta-qemux86-64.conf
    ├── wic/asterquanta-ab.wks.in         # partitions réelles A/B + data
    ├── recipes-core/
    │   ├── images/asterquanta-image.bb   # image finale (rootfs + GUI + services)
    │   └── rauc/files/system.conf        # config slots A/B RAUC
    └── recipes-aqm/
        ├── aqm-supervisor/ (recette + source Rust)
        ├── aqm-shell/      (recette + source Rust, binaire aqmctl)
        ├── aqm-ui/         (recette + source GTK4)
        ├── aqm-recovery/   (recette + script recovery)
        └── aqm-dtnd/       (recette wrappant uD3TN upstream)
```

## 4. Dépendances intégrées dans l'image (rien à installer par l'utilisateur)

Tout est compilé et embarqué dans l'image finale par Yocto : glibc,
BusyBox/coreutils, systemd, dbus, PAM, GTK4 + ses libs, Cage/wlroots,
RAUC + openssl (vérification de signature), btrfs-progs, uD3TN + ses
dépendances réseau. L'utilisateur flashe l'image `.wic`/`.img` sur un
disque/une clé et démarre — zéro installation manuelle de dépendance.

## 5. Flux de boot (réel)

1. Firmware (BIOS/UEFI ou U-Boot) → GRUB.
2. GRUB lit sa variable d'environnement `aqm_active_slot` (A ou B, gérée
   par RAUC) et boot le noyau + initramfs du slot actif.
3. Noyau Linux monte le rootfs du slot actif (lecture seule + overlay
   pour `/etc`, `/var` persistants sur la partition `data`).
4. `systemd` (PID 1) démarre les targets : `sysinit` → `basic` →
   `multi-user` → `graphical`.
5. Services système démarrent dans l'ordre de dépendance déclaré par
   leurs `.service` : `aqm-supervisor` (After=multi-user), `aqm-dtnd`,
   `rauc.service`, `aqm-recoveryd`.
6. `aqm-ui.service` (`WantedBy=graphical.target`) lance `cage -- aqm-ui`
   automatiquement sur le tty graphique — **aucune action utilisateur
   requise**, l'interface s'affiche seule.
7. `aqm-supervisor` publie un premier rapport de santé dans le journal
   et via son socket UNIX, consommé par `aqm-ui`.

## 6. Flux d'installation

- **v1 (recommandé pour un dev solo)** : image `.wic` flashée directement
  sur le support de destination (`dd`/`bmaptool`) — pas d'installeur
  interactif, l'image EST le système installé (modèle "appliance", standard
  en embarqué).
- **v2 (si un installeur interactif est nécessaire)** : ajouter une image
  "live installer" minimaliste (`asterquanta-installer-image.bb`, recette
  séparée) qui partitionne le disque cible et copie le rootfs — pattern
  classique Yocto, à ajouter une fois le système central stable.

## 7. Flux de mise à jour (A/B réel via RAUC)

1. `aqmctl update install <bundle.raucb>` (bundle signé, `.raucb`).
2. RAUC vérifie la signature (clé publique embarquée dans l'image) —
   rejet si signature invalide.
3. RAUC écrit l'image sur le slot **inactif** (B si A est actif).
4. RAUC bascule le flag de boot vers le slot B et redémarre.
5. Au reboot, un service de validation (`aqm-supervisor` healthcheck)
   doit confirmer "boot sain" dans un délai fixé (`RAUC mark good`).
6. Si le healthcheck échoue ou si le nombre de tentatives de boot est
   dépassé, GRUB/RAUC **rebascule automatiquement sur le slot A** —
   rollback réel, pas simulé.

## 8. Flux de recovery

1. `aqm-recoveryd` prend des snapshots BTRFS périodiques et avant chaque
   update de `/etc` et `/home` (persistants, séparés des slots A/B).
2. En cas de démarrage en échec répété : GRUB boot un **noyau/rootfs de
   recovery** minimal (troisième partition dédiée, indépendante de A/B)
   avec `aqmctl` en mode texte pour restaurer un snapshot ou relancer un
   rollback RAUC manuel.
3. Mode sûr (`aqmctl safe-mode on`) : `systemctl isolate aqm-safe.target`,
   qui ne démarre que les services critiques (`aqm-supervisor`, `sshd`,
   `aqm-shell`), sans `aqm-ui`.

## 9. Sécurité

- Bundles RAUC signés (openssl, clé privée hors image, clé publique
  embarquée) — refus d'installer un paquet non signé.
- `dm-verity` optionnel sur le rootfs (via `meta-security`) pour intégrité
  au runtime — activable en v2 sans changer l'architecture.
- Utilisateurs/permissions réels via PAM + groupes Unix standards (pas de
  modèle custom) : `operator` (sudo restreint), `aqm-service` (compte
  système sans shell pour les daemons).
- Logs signés/horodatés via `journald` avec rotation persistante sur la
  partition `data`.

## 10. Couche réseau DTN

`aqm-dtnd` = wrapper Yocto autour de **uD3TN** (implémentation open-source
du Bundle Protocol, utilisée dans de vrais projets spatiaux/IoT à
connectivité intermittente — exactement le cas d'usage décrit). Pas de
protocole DTN réinventé : on s'appuie sur un vrai standard (RFC 9171)
avec une vraie implémentation maintenue. `aqmctl dtn status|send|discover`
pilote `aqm-dtnd` via son AAP (Application Agent Protocol) socket, réel.

## 11. Stack technique recommandée

| Domaine | Choix | Pourquoi |
|---|---|---|
| Build system | Yocto piloté par `kas` | reproductible, industriel, cache la complexité bitbake |
| Init | systemd | gestion de services/dépendances/logs standard et robuste |
| Update A/B | RAUC | mature, signé, rollback automatique, bien intégré Yocto |
| GUI | GTK4 + Cage (Wayland kiosk) | léger, buildable Yocto, pas de bureau complet inutile |
| Services AQM | Rust (std, peu de deps) | binaires petits, sûrs mémoire, faciles à recipe Yocto |
| DTN | uD3TN | implémentation réelle du Bundle Protocol, pas de réinvention |
| Filesystem persistant | BTRFS (snapshots) | recovery réel via snapshots natifs |
| Paquets applicatifs | opkg (ipk) | standard embarqué, léger |
| Cible v1 | qemux86-64 | testable sans matériel réel avant portage carte cible |

## 12. Contenu minimal de la v1 (scope réaliste solo dev)

- Boot QEMU x86_64 fonctionnel avec GRUB + noyau standard.
- `aqm-supervisor` + `aqm-shell` opérationnels (status réel des services
  systemd).
- `aqm-ui` : un seul écran (statut système + liste services), lancé
  automatiquement par Cage.
- RAUC configuré avec 2 slots + 1 bundle de test signé, rollback testé
  manuellement en QEMU.
- `aqm-dtnd` démarré, `aqmctl dtn status` fonctionnel entre deux instances
  QEMU sur le même réseau virtuel (test DTN réel local avant vrai lien
  intermittent).
- Recovery : snapshot BTRFS manuel + restauration testée.
- Pas encore en v1 : installeur graphique, dm-verity, portage matériel
  réel, DTN longue distance.

## 13. Plan de build concret (ordre d'exécution)

1. `kas-asterquanta.yml` + squelette `meta-asterquanta` (ce dépôt) → build
   une image de base sans services AQM, valider le boot QEMU.
2. Ajouter `aqm-supervisor` + `aqm-shell`, rebuild, valider `aqmctl status`
   en SSH sur l'image QEMU.
3. Ajouter `aqm-ui` + Cage, valider l'affichage automatique au boot.
4. Intégrer RAUC + `wic` A/B, valider un update + rollback en QEMU.
5. Intégrer `aqm-dtnd` (uD3TN), valider un échange de bundle entre deux
   instances QEMU.
6. Intégrer `aqm-recoveryd` + snapshots BTRFS + target `aqm-safe.target`.
7. Durcissement sécurité (signature obligatoire, `dm-verity` optionnel).
8. Portage vers une cible matérielle réelle (Raspberry Pi 4 via
   `meta-raspberrypi`, ou carte x86 embarquée).

## 14. Fichiers à créer en premier

Déjà créés dans ce dépôt, dans cet ordre de priorité :
1. `kas-asterquanta.yml`
2. `meta-asterquanta/conf/layer.conf`
3. `meta-asterquanta/conf/distro/asterquanta.conf`
4. `meta-asterquanta/conf/machine/asterquanta-qemux86-64.conf`
5. `meta-asterquanta/recipes-core/images/asterquanta-image.bb`
6. `meta-asterquanta/wic/asterquanta-ab.wks.in`
7. `meta-asterquanta/recipes-aqm/aqm-supervisor/*`
8. `meta-asterquanta/recipes-aqm/aqm-shell/*`
9. `meta-asterquanta/recipes-aqm/aqm-ui/*`
10. `meta-asterquanta/recipes-aqm/aqm-dtnd/*`
11. `meta-asterquanta/recipes-core/rauc/files/system.conf`

## 15. Design final — une seule vue

```
                        ┌─────────────────────────┐
                        │      aqm-ui (GTK4)       │  <- auto-lancé au boot
                        │  écran statut + services │     par Cage (Wayland)
                        └────────────┬────────────┘
                                     │ socket UNIX / D-Bus
                        ┌────────────▼────────────┐
                        │      aqm-shell           │  <- aqmctl (CLI + API interne)
                        │ status/services/update/  │
                        │ dtn/logs/recovery         │
                        └───┬────────┬────────┬────┘
              ┌─────────────┴────────┴─────────────┐
              │                                    │
              │ systemd (PID 1)                    │
              │ journald, logind, networkd, timed  │
              │                                    │
              └────────────────────────────────────┘
                                  │
                                  │ systemd units
                                  │
              ┌───────────────────▼───────────────────┐
              │ aqm-supervisor (Rust)                 │  <- supervise les autres services
              │ aqm-dtnd (uD3TN)                      │     via systemd/dbus, calcule le
              │ aqm-recoveryd (Rust)                  │     score de santé
              │ rauc (C)                              │
              └───────────────────────────────────────┘
                                  │
                                  │ appels système / bibliothèques
                                  │
              ┌───────────────────▼───────────────────┐
              │ Noyau Linux                           │
              │ glibc, btrfs-progs, openssl, ...      │
              └───────────────────────────────────────┘
```
```

## Contribution

Nous accueillons les contributions à ce projet ! Si vous souhaitez contribuer, veuillez suivre les directives ci-dessous :

### Comment contribuer

1.  **Fork** le dépôt sur GitHub.
2.  **Clonez** votre fork localement : `git clone https://github.com/votre-utilisateur/asterquanta-os-yocto.git`
3.  Créez une **nouvelle branche** pour vos modifications : `git checkout -b feature/votre-fonctionnalite` ou `bugfix/correction-bug`.
4.  Effectuez vos modifications et **testez-les** soigneusement.
5.  **Commitez** vos modifications avec un message clair et descriptif : `git commit -m 
feat: Ajout d'une nouvelle fonctionnalité`.
6.  **Pushez** vos modifications vers votre fork sur GitHub : `git push origin feature/votre-fonctionnalite`.
7.  Ouvrez une **Pull Request** (PR) depuis votre fork vers la branche `main` du dépôt original.

### Directives de contribution

-   Assurez-vous que votre code respecte les conventions de style existantes.
-   Documentez clairement toutes les nouvelles fonctionnalités ou modifications.
-   Ajoutez des tests unitaires si applicable.
-   Soyez respectueux et constructif dans vos interactions.

Merci de votre intérêt pour le projet AsterQuanta OS !
