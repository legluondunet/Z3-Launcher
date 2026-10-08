# Z3-Launcher — Rust / Linux et Windows 0.11.16

Réécriture Rust du launcher C#/WinForms de RadzPrower (Anthony Johns), sous
licence GPL-3.0-or-later pour ce fork. La notice MIT du launcher original
est conservée dans LICENSE-MIT-original. Le launcher est en Rust ; le jeu externe snesrev/zelda3 reste en C
et son extraction utilise Python système. Aucune ROM ou ressource Nintendo
n'est distribuée dans cette archive.

Dépôt officiel : [legluondunet/Z3-Launcher](https://github.com/legluondunet/Z3-Launcher).

## Auteurs

- Launcher original C#/WinForms : Anthony Johns (RadzPrower), 2023.
- Fork et réécriture du launcher en Rust : [legluondunet](https://github.com/legluondunet), 2026.

## Distribuer une AppImage Linux

Exécuter `./build-appimage.sh` sur Linux x86_64 pour produire `dist/Z3-Launcher-0.11.16-x86_64.AppImage`. Voir [APPIMAGE.md](APPIMAGE.md) pour les dépendances de construction, les limites et la validation avant distribution.

## Windows

Le port Windows utilise MSYS2/UCRT64 pour construire le jeu et lance `zelda3.exe` nativement. Voir [WINDOWS.md](WINDOWS.md) pour la compilation du launcher et les dépendances du jeu. Le mode portable, les traductions, les réglages et le remappage sont conservés.

Cette version contient les adaptations Windows, mais la compilation et le lancement Windows restent à valider sur une machine Windows.

## Compiler sous Linux

Sur Manjaro/Arch, avec les outils de développement SDL2 et un Rust stable récent :

```bash
sudo pacman -S --needed rust cargo git base-devel python python-pillow python-yaml sdl2 libxkbcommon wayland libx11 libxcursor libxi libxrandr mesa
./check-linux.sh
cargo build --release
./target/release/z3-launcher
```

Rust 1.85 minimum déclaré ; les dépendances transitives peuvent réclamer une
version stable plus récente. Cargo télécharge les dépendances. Interface
X11/Wayland via eframe 0.31.1, sélecteur de fichiers rfd 0.15.4 et manettes
via sdl2 0.37.0. Le sélecteur nécessite le portail XDG du bureau et son backend.

Conserver le même dossier de travail que dans les versions précédentes pour
retrouver le dépôt, la ROM, les réglages et les sauvegardes. Remplacer les sources
et recompiler ne nécessite pas de réinstaller le jeu.

## Général

- Ajouter la ROM par glisser-déposer ou avec Parcourir… (filtres .sfc/.smc).
- Installer + compiler : vérifier les prérequis/ROM, cloner récursivement le
  dépôt (ou le mettre à jour par pull --ff-only s’il existe), copier la ROM, nettoyer les objets compilés et l’exécutable avec make clean_obj, extraire les ressources et recompiler avec make.
- Mettre à jour : pull Git --ff-only puis compilation. Pas de reset/clean.
- Lancer : exécuter le binaire Linux dans son répertoire de jeu.
- Journal : aperçu dans la fenêtre et fichier complet launcher.log.

La ROM US sans en-tête doit avoir le SHA256
`66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb`.
Déposer une ROM ne déclenche pas automatiquement l'installation.

Les actions tournent dans un thread séparé. Lancer attend la fermeture du jeu ;
les autres actions sont désactivées durant cette période. Pas d'annulation de
compilation pour l'instant. Un clonage interrompu peut laisser un dossier partiel,
qui n'est pas supprimé automatiquement.

## Onglets de réglages et remappage

Les onglets Général, Jeu, Affichage, Son / MSU, Clavier, Manette et Raccourcis sont tous au même niveau. Les six onglets de réglages proposent :

| Page | Contenu |
|---|---|
| Jeu | Sauvegarde automatique, FPS, délai d'image, langue et améliorations du gameplay |
| Affichage | Ratio et variantes, dimensions, plein écran, échelle, rendu, filtrage, Mode 7, sprites, flashs, fichiers ZSPR et shaders |
| Son / MSU | Audio, fréquence, mono/stéréo, tampon, format MSU/OPUZ, volume, reprise et préfixe des pistes |
| Clavier | 12 commandes SNES, préréglages QWERTY/AZERTY/QWERTZ, saisie et capture |
| Manette | 12 commandes SNES, liste des boutons SDL, capture et retour aux valeurs par défaut |
| Raccourcis | Jeu, volume, cheats, rendu, replays et 10 emplacements de sauvegarde/chargement/replay, séparément pour clavier et manette |

L’onglet INI affiche zelda3.ini dans un éditeur intégré en lecture et écriture. Quitter cet onglet enregistre automatiquement le fichier et actualise les formulaires. En cas de valeur invalide ou d’échec d’écriture, l’onglet reste ouvert et le brouillon est conservé pour correction. Fermer le launcher tente également d’enregistrer les changements. La sauvegarde précédente reste disponible dans zelda3.ini.bak.

Chaque modification valide effectuée dans les formulaires est sauvegardée automatiquement, y compris les préréglages, les captures et la langue après import. Il n’y a plus de bouton Enregistrer ni Reload. Les réglages s’appliquent au prochain lancement du jeu.

Les commentaires, paramètres inconnus, sections et fins de ligne sont conservés. Une sauvegarde zelda3.ini.bak précède chaque écriture, suivie d'un remplacement par fichier temporaire. Une valeur temporairement invalide (par exemple une dimension incomplète pendant la saisie) reste en mémoire et ne remplace pas le fichier valide. La sauvegarde est relancée après correction. En cas d'erreur d'écriture, un message apparaît et Réessayer la sauvegarde automatique permet de réessayer ; Annuler les modifications revient au fichier du disque. Les actions du launcher sont bloquées tant que des modifications restent en attente. La fermeture ne demande confirmation que si des modifications n'ont pas pu être sauvegardées.

Capturer attend une touche ou un bouton pendant 15 secondes. Échap annule ;
la valeur précédente reste en place. Affecter toutes les commandes passe de
Haut à R successivement. Les touches Ctrl/Shift/Alt sont combinées automatiquement.
Les modificateurs seuls peuvent être saisis manuellement (ex. Right Shift).
Une virgule seule ne peut pas être affectée via ce format INI.

Pour les manettes, SDL fournit les noms logiques A/B/X/Y, Lb/Rb, L2/R2, etc.
Ils restent ces noms même pour une DualShock. Les gâchettes sont détectées avec
un seuil, les sticks analogiques ne sont pas convertis en boutons. Le jeu conserve
sa gestion native des sticks. Les manettes SDL sont ouvertes pendant la capture ;
le branchement à chaud est géré via les événements SDL. Une affectation par liste
reste possible sans manette connectée. Les captures clavier/manette restent
indépendantes et les captures n'écrivent pas directement sur disque.

### Limites fonctionnelles

Les formulaires remplacent l'édition manuelle, mais toutes les fonctions externes
de l'ancien launcher ne sont pas reproduites : l’import des ROM de langue est maintenant disponible (voir version 0.8). Les fichiers MSU doivent être installés manuellement ;
MSUPath est un préfixe de fichier, pas uniquement un dossier. ZSPR/shaders sont choisis
par chemin, sans téléchargement automatique. La construction Windows n'est pas portée.
Les affectations en doublon ne sont pas détectées ; le jeu peut signaler des conflits.

## Ligne de commande

```bash
cargo run --release -- check
cargo run --release -- setup --rom "/chemin/ma ROM.sfc"
cargo run --release -- update
cargo run --release -- build
cargo run --release -- run
cargo run --release -- status
```

Ajouter --dir "/chemin/dossier de travail" si nécessaire. Défaut :
$XDG_DATA_HOME/Z3-Launcher ou ~/.local/share/Z3-Launcher.
La CLI utilise ce défaut, pas le dossier mémorisé dans la GUI.

## Validation

La version 0.1 a été compilée et utilisée avec succès par l'utilisateur sur Linux.
L'extraction réelle de sa ROM US avec le script du jeu a produit un
zelda3_assets.dat de 683 888 octets, sans erreur (commit jeu
45a149d8b4bb8098f53d1009f76e76e7d8b79bbf).

Le SDK Rust est absent dans l'environnement de préparation : la version 0.4
n'a PAS été compilée, ses tests et sa GUI ne sont PAS exécutés ici. Les API
utilisées ont été comparées aux sources egui 0.31.1 et rust-sdl2 0.37.0.
Les tests sources couvrent la conservation INI, le remappage d'un seul emplacement,
les modificateurs clavier, les dimensions invalides, la sauvegarde INI et
les erreurs/logs de processus. Vérifier localement :

```bash
cargo fmt
cargo test --no-default-features
cargo test
cargo build --release
```

Le noyau CLI peut être compilé sans interface (serde/serde_json restent requis) avec
cargo build --release --no-default-features.

Sources :
- https://github.com/snesrev/zelda3/wiki/1.-Getting-Started
- https://github.com/snesrev/zelda3/wiki/2.-Cloning-and-Setup
- https://github.com/emilk/egui/tree/0.31.1
- https://github.com/Rust-SDL2/rust-sdl2/tree/0.37.0

## Version 0.6 : bilan des dépendances selon la distribution

La cible reste Linux uniquement ; les changements Windows/macOS sont annulés.
Vérifier inspecte toutes les dépendances, sans s'arrêter à la première erreur.
Il affiche dans le journal la distribution détectée, chaque résultat, la liste
complète des dépendances manquantes/inutilisables et un exemple de commande.
Copier le journal permet de récupérer cette commande pour son terminal.
Aucune commande d'installation n'est exécutée par le launcher.

Familles prises en charge : Debian/Ubuntu (y compris Linux Mint et dérivées),
Arch/Manjaro, Fedora/Nobara, openSUSE Leap/Tumbleweed. ID et ID_LIKE sont lus depuis
/etc/os-release, sans exécuter ce fichier. Les autres distributions affichent
le bilan sans inventer de commande. Les systèmes immuables connus et les variantes
RHEL restent hors du périmètre des commandes proposées.

Les commandes ne contiennent que les paquets associés aux dépendances détectées,
avec déduplication. Les modules Pillow/PyYAML sont vérifiés séparément dans le
python3 utilisé par l'application. Une installation système ne corrigera pas
forcément un environnement virtuel Python isolé.

SDL2 est aujourd'hui fourni par sdl2-compat sous Arch et sdl2-compat-devel sous
Fedora récente. Les versions anciennes de ces distributions peuvent encore
utiliser sdl2/SDL2-devel ; la commande est un exemple à adapter si les dépôts
locaux proposent ces anciens noms. openSUSE utilise libSDL2-devel et Debian/Ubuntu
libsdl2-dev.

Tests sources ajoutés : détection des familles/dérivées, exclusion des distributions
immuables et déduplication/limitation des paquets proposés. SDK Rust toujours absent
ici : version 0.6 non compilée ni exécutée dans l'environnement de préparation.

Références paquets :
- https://archlinux.org/packages/extra/x86_64/sdl2-compat/files/
- https://packages.fedoraproject.org/pkgs/sdl2-compat/sdl2-compat-devel/
- https://packages.ubuntu.com/noble/libsdl2-dev

## Version 0.7 : langues de l'interface

General / Général contient le choix Interface language / Langue de l'interface.
English est le défaut, quelle que soit la langue du système. Français, Italiano, Español et Deutsch sont fournis.
Le changement est immédiat et mémorisé dans language.txt du répertoire de
configuration. Le choix est indépendant de la langue du jeu et de zelda3.ini.
Le sélecteur est désactivé pendant les opérations pour conserver la cohérence
linguistique des nouveaux messages du journal.

Les 341 messages du launcher sont extraits dans les fichiers JSON de locales/.
Les cinq catalogues sont embarqués ; ils restent disponibles sans
fichiers externes. Les fichiers externes sont chargés au démarrage : locales/
du répertoire courant, puis locales/ à côté du binaire, puis le dossier utilisateur
~/.config/Z3-Launcher/locales (ou son équivalent XDG). Une traduction absente
ou avec des paramètres erronés utilise automatiquement l'anglais embarqué.

Pour ajouter une langue sans modifier Rust ni recompiler, copier en.json, traduire
les valeurs, changer language.code/name et déposer le JSON dans un de ces dossiers.
Consulter TRANSLATING.md et vérifier avec :

```bash
python3 tools/check-translations.py
```

La vérification Python des deux catalogues a été exécutée : 200 messages chacun,
identifiants et paramètres cohérents. L'outil a aussi été testé sur une traduction
incomplète, un paramètre incorrect et des clés JSON dupliquées. Les tests Rust
sources couvrent le retour à l'anglais et la substitution non récursive des valeurs.
Le SDK Rust étant absent ici, la version 0.7 n'a pas été compilée ni sa GUI exécutée.

Les sorties des programmes externes restent dans leur langue d'origine et les
anciennes entrées du journal ne sont pas réécrites après un changement de langue.
Les polices embarquées couvrent les écritures latines courantes ; d'autres systèmes
d'écriture peuvent nécessiter l'ajout d'une police adaptée avant une traduction
complète de l'affichage. Le script shell check-linux.sh conserve des messages anglais.

Correction 0.7 : le commentaire de module de dependencies.rs est replacé avant les imports (erreur Rust E0753).

## Version 0.8 : import des langues du jeu

Fonction restaurée depuis settingsForm.cs du launcher d'origine. Dans Options >
Jeu, sélectionner une langue autre que US English ouvre le sélecteur de ROM
.sfc/.smc. La ROM US reste la base du jeu ; la ROM choisie sert aux dialogues et
à la police de la langue sélectionnée. Anglais européen (en) et portugais (pt)
sont ajoutés à la liste, comme dans le programme original.

Le launcher appelle util.load_rom avec le mode multilingue du dépôt Zelda 3,
qui vérifie les hashes des versions reconnues, puis compare le code de langue
identifié au code sélectionné. Une ROM US choisie pour le français est donc
refusée. Certaines langues requièrent précisément une ROM patchée prise en charge
par le script, pas n'importe quelle traduction. Le détail est affiché dans le journal.

Après vérification :
1. python3 restool.py --extract-dialogue --rom /chemin/ROM
2. python3 restool.py --languages=<langues dont dialogues ET polices existent>
3. le formulaire adopte la nouvelle langue et sauvegarde automatiquement Language dans zelda3.ini.

L'import tourne en arrière-plan et n'exige pas de recompilation C du jeu.
La sélection est annulable et la langue précédente reste inchangée en cas
 d'échec. Les fichiers de dialogue, police et zelda3_assets.dat sont sauvegardés
avant l'opération puis restaurés si une étape échoue. Fermer le launcher est
bloqué pendant cet import. Le retour à US English n'exige pas de seconde ROM.
Le bouton Réimporter permet de relancer l'opération pour la langue actuelle.
Le sélecteur demande aussi une ROM lors d'un changement vers une langue déjà
extraite ; les autres langues présentes restent incluses dans les ressources.

Les compilations/mises à jour du jeu incluent les langues déjà extraites, pour
ne pas les perdre en régénérant les assets. Le launcher ne copie ni ne modifie
la ROM de traduction fournie ; son chemin est transmis comme argument séparé.
Le choix de langue d'interface reste indépendant de la langue du jeu.
L'éditeur INI avancé peut modifier Language manuellement ; cette voie experte
ne déclenche pas l'extraction automatique.

Validation effectuée avec la ROM US disponible : le vérificateur Python utilisé
par le code accepte son code us et refuse ce même fichier comme code fr ; le
fichier de ressources existant reste inchangé. Aucune ROM française n'est
 disponible ici : extraction française réelle non testée. SDK Rust absent :
compilation et exécution GUI de la version 0.8 non testées dans cet environnement.
Les catalogues anglais/français et toutes leurs références sont vérifiés.

## Version 0.9 : mode portable

Créer un fichier vide nommé portable.txt à côté de l'exécutable, puis relancer :

```bash
touch target/release/portable.txt
./target/release/z3-launcher
```

Ce fichier active le mode portable au démarrage. Son contenu n'a pas d'importance.
Un dossier portant ce nom n'active pas le mode. Le launcher utilise l'emplacement
réel de l'exécutable, indépendamment du répertoire courant du terminal.

| Emplacement à côté de l'exécutable | Contenu |
|---|---|
| Z3-Launcher/zelda3/ | dépôt, ROM US copiée, ressources, INI et sauvegardes du jeu |
| Z3-Launcher/launcher.log | journal du launcher |
| Z3-Launcher/config/language.txt | langue de l'interface |
| Z3-Launcher/config/workspace.txt | préférence de dossier |
| Z3-Launcher/config/locales/ | traductions utilisateur |

En mode portable le dossier de travail est fixé à Z3-Launcher/ : les préférences contenant
un ancien chemin absolu sont ignorées, le champ est désactivé et l'option CLI --dir
ne remplace pas cette destination. On peut donc déplacer l'exécutable, portable.txt
et Z3-Launcher/ ensemble. L'ajout/suppression du marqueur nécessite un redémarrage.
Les chemins externes saisis pour MSU, shaders et sprites doivent être adaptés
après déplacement ; les fichiers externes ne sont pas copiés automatiquement.

Les anciennes données utilisateur ne sont pas déplacées ni copiées automatiquement.
Pour conserver une installation existante, fermer le launcher et copier le dépôt
zelda3/ du dossier de travail précédent dans Z3-Launcher/zelda3/. Les préférences peuvent
être copiées dans Z3-Launcher/config/ ; workspace.txt reste ignoré en mode portable.
Sans portable.txt, les emplacements XDG et les préférences classiques restent utilisés.
Si le dossier de l'exécutable n'est pas accessible en écriture, une erreur est affichée
sans basculer silencieusement vers le répertoire utilisateur.

Tests Rust sources ajoutés pour la détection du marqueur (fichier/dossier) et
l'emplacement des données. SDK Rust absent ici : version 0.9 non compilée ni testée
en exécution dans l'environnement de préparation. Les catalogues sont validés.

## Version 0.11.4 : sélection du journal

Le journal est une zone de texte en lecture seule : sélection sur plusieurs lignes, Ctrl+C et menu contextuel Copier la sélection / Copier le journal. Le clic droit conserve la sélection existante. Le bouton Copier le journal reste disponible.


## Version 0.11.3 : onglets directs et sauvegarde automatique

Suppression de l’onglet Options : ses six pages rejoignent Général dans la barre principale. Sauvegarde automatique à chaque modification valide, avec conservation du dernier fichier valide en cas de saisie incorrecte ou d’échec d’écriture. Tests Rust ajoutés pour les dimensions invalides, les sauvegardes, les captures et la langue importée ; non exécutés ici car Rust est absent.

## Version 0.11.4 : installation avec mise à jour et recompilation complète

Installer + compiler vérifie la ROM et les dépendances, puis met le dépôt existant à jour avec git pull --ff-only et synchronise ses sous-modules. Un dépôt absent est cloné récursivement. Si la mise à jour échoue (réseau, modifications locales incompatibles, branche divergente), l’opération s’arrête avant le nettoyage.

Avant recompilation, make clean_obj supprime les objets C et l’exécutable ; sous Windows, zelda3.res est également supprimé. Le nettoyage complet make clean n’est pas utilisé, car il supprimerait aussi des dialogues et polices importés. Les ressources sont ensuite régénérées avec les langues présentes et le jeu entièrement recompilé. Le fichier INI, les sauvegardes, la ROM et les ressources de langue restent conservés. Si la compilation échoue après nettoyage, l’ancien exécutable n’est plus disponible jusqu’à une compilation réussie. Les boutons de mise à jour et de recompilation conservent leur comportement précédent.

## Version 0.11.5 : actions simplifiées et filtre shader

Les boutons Rebuild et Status sont retirés de l’interface ; leurs commandes CLI restent disponibles. Le sélecteur de shader dans Affichage accepte .glsl, .glslp et .slangp, y compris leurs extensions en majuscules. Le chemin est sauvegardé automatiquement dans Graphics/Shader. L’ajout au filtre ne modifie pas le moteur de shader du jeu : les presets doivent rester compatibles avec son pipeline GLSL.

## Version 0.11.6 : actions principales et infobulles

Le bouton Update and build est retiré. Install and build devient Build Zelda 3 (Compiler Zelda 3), et Launch game devient Play Zelda 3 (Jouer à Zelda 3). La mise à jour est toujours effectuée par la commande de construction ; les commandes CLI restent disponibles.

Des infobulles sur les réglages, les contrôles et les actions expliquent leur fonction au survol. Les descriptions du launcher d’origine ont été reprises et adaptées au comportement actuel. Les 77 textes d’aide sont présents dans les catalogues JSON anglais et français, et peuvent être traduits comme les autres textes. Les catalogues et les références sont vérifiés ; la compilation et le comportement de survol restent à tester localement, Rust étant absent de l’environnement de préparation.

## Version 0.11.7 : édition externe de l’INI et bibliothèque de shaders

Suppression de .slangp dans le sélecteur et refus des chemins de shaders autres que .glsl/.glslp lors de la validation. Suppression des boutons Reload et du texte « Each valid change… ». Éditer le fichier INI est uniquement dans Général et ouvre l’éditeur système, avec rechargement automatique des modifications enregistrées.

Build Zelda 3 vérifie la bibliothèque de shaders issue de https://github.com/libretro/glsl-shaders. Si elle est absente ou incomplète, il télécharge l’archive ZIP master, l’extrait dans une zone temporaire et installe les fichiers dans le dossier de données du launcher sous shaders/. Les fichiers supplémentaires sont conservés lors d’une récupération. Une installation valide est réutilisée, sans nouveau téléchargement systématique. Les shaders sont des données persistantes, non du cache. Aucun shader n’est inclus dans l’archive du launcher. Le sélecteur de shader s’ouvre directement dans ce dossier ; il peut être vide avant le premier Build.

Chemins : ~/.local/share/Z3-Launcher/shaders (ou XDG_DATA_HOME), Z3-Launcher/shaders avec portable.txt, et <AppImage>.home/.local/share/Z3-Launcher/shaders avec le mode .home. Sous Windows : %LOCALAPPDATA%/Z3-Launcher/shaders, ou l’équivalent portable.

Le téléchargement/extraction et les catalogues sont contrôlés avec des fixtures locales. La compilation Rust et l’ouverture d’un éditeur graphique restent à tester sur la machine utilisateur.

## Version 0.11.8 : activation conditionnelle du shader

Le champ shader et son bouton de sélection sont désactivés avec les rendus SDL et SDL-Software. Ils sont disponibles avec OpenGL ou OpenGL ES. Le chemin déjà enregistré est conservé lorsqu’un autre moteur est choisi.

## Version 0.11.9 : thème forêt et or

Thème inspiré de la maquette choisie : fond vert forêt texturé, cadre et contrôles dorés, typographie DejaVu Serif couleur parchemin et bouton Play Zelda 3 mis en avant. L’organisation des onglets, champs, actions et journal reste la même. Les textes Portable mode enabled et Additional translations… sont retirés de l’interface ; le mode portable et les traductions externes restent fonctionnels.

Le fond PNG et la police sont inclus dans assets/ et embarqués dans le binaire, avec la licence de redistribution de la police. L’AppImage ne nécessite pas de dossier d’images externe. La barre de titre native dépend du bureau utilisé. Les catalogues, les références aux fichiers et les API egui sont vérifiés ; la compilation et le rendu réel restent à tester localement.

## Version 0.11.10 : avertissements du thème

Les six épaisseurs de trait passées à Stroke::new sont explicitement typées f32 pour supprimer les avertissements float_literal_f32_fallback des compilateurs récents.

## Version 0.11.11 : onglet INI intégré

Le bouton d’édition externe est remplacé par un onglet INI. Les modifications sont enregistrées au changement d’onglet et les réglages sont actualisés. Un brouillon invalide reste dans l’éditeur sans écraser le fichier précédent.

## Version 0.11.12 : journal et barre d’état

Le journal de Général occupe la hauteur disponible, avec ses boutons de copie et d’effacement en dessous. Son titre est supprimé. Les libellés des onglets utilisent une police plus petite ; leurs boutons ont des bordures dorées arrondies et l’onglet actif est rempli de doré. Les messages d’action et l’indicateur d’activité sont déplacés dans une barre d’état en bas de la fenêtre, visible sur tous les onglets.

## Version 0.11.13 : réglages plus compacts

La police des boutons et des options est réduite de deux points. Le chemin du fichier de configuration n’est affiché que dans l’onglet INI. Dans Gameplay, les réglages Général apparaissent à gauche et les améliorations de gameplay à droite, dans des sections encadrées aux titres dorés.

## Version 0.11.15 : attribution et licence

Le fork est attribué à legluondunet (2026) et distribué sous GPL-3.0-or-later. Voir [LICENSING.md](LICENSING.md) pour la notice, les attributions et la distribution de binaires. La notice MIT originale et la licence de DejaVu sont conservées.

## Version 0.11.15 : Z3-Launcher

Nouveau nom affiché, exécutable z3-launcher (z3-launcher.exe sous Windows), AppImage Z3-Launcher et artefacts GitHub mis à jour. Les répertoires de données et de préférences utilisent désormais Z3-Launcher. Après renommage d’une AppImage, renommer également son éventuel dossier .home pour qu’il corresponde au nom complet du nouveau fichier.

## Version 0.11.16 : nouveaux chemins

Les données utilisent ~/.local/share/Z3-Launcher et les préférences ~/.config/Z3-Launcher sous Linux (ou leurs équivalents XDG). Sous Windows, les données utilisent %LOCALAPPDATA%/Z3-Launcher. Avec portable.txt, le dossier situé près de l’exécutable se nomme Z3-Launcher. Le mode AppImage .home utilise également ces noms. Les anciens dossiers ne sont pas déplacés automatiquement : renommer leurs dossiers pour retrouver les données existantes.
