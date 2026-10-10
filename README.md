# AionPartyMeter

Application Windows x64 en français pour les statistiques du **groupe entier** dans Aion 2. Capture passive via Npcap, moteur Rust et overlay Tauri. Dérivé d’A2Tools, sous GPL-3.0-only.

## Lancer

1. Extraire tout `AionPartyMeter-0.1.12-Windows-x64.zip`, puis ouvrir **DEMARRER.exe**. Le lanceur vérifie les nouvelles versions stables sur GitHub, puis les dépendances. Les DLL Visual C++ requises et l’installateur WebView2 hors ligne sont fournis ; aucun outil de développement n’est nécessaire. Lire `LIRE-MOI.html` dans le pack ou `docs/INSTALLATION.md`.
2. Installer [Npcap](https://npcap.com/#download) si le diagnostic le signale absent. Le chargement prend en charge son dossier standard `System32\Npcap` et le mode compatible WinPcap. Le pilote n’est pas embarqué ni installé automatiquement.
3. Cliquer **Démarrer**. Si Npcap refuse l’accès, fermer l’application et la lancer en administrateur.
4. Ouvrir **Groupe** dans la barre de commandes : la liste des joueurs découverts est actualisée en direct. Cliquer **+** sur un coéquipier ou saisir son pseudo exact puis Entrée, sans quitter le groupe. Si la liste réseau est décodée, les membres sont ajoutés automatiquement. Une identité non encore reçue reste signalée, même sans dégâts.
5. Combattre et ouvrir **Dégâts**, **Soins** ou **Dégâts reçus**. Cliquer un membre pour ses statistiques, compétences et courbes.

Version 0.1.6 : compact à trois statistiques simultanées (dégâts, soins, reçus), totaux du groupe et taux par seconde. Actualisation par défaut 10 Hz, réglable à 4 ou 20 Hz, animations discrètes désactivables. État du groupe courant et identités en attente visibles, retrait immédiat et mode solo explicite. Vue complète conservée : onglet **Toutes les stats** avec dix colonnes triables, totaux du groupe et trois courbes simultanées (DPS, HPS et dégâts reçus par seconde). La vue s’agrandit à son ouverture ; les graphiques peuvent être masqués. Fonctionnalités conservées : mode compact mémorisé, mises à jour du tableau sans recréer ses lignes, détails chargés à la demande et ajout immédiat des coéquipiers découverts dans Groupe. Le panneau reste actualisé pendant la saisie ; un court arrêt/redémarrage de capture sur la même connexion conserve les identités. Le client peut encore omettre sa liste de groupe en monde ouvert : la sélection explicite est alors nécessaire. Correctif précédent : attaques directes des monstres désormais décodées pour les dégâts reçus, y compris avant une attaque du groupe. Les premières identités sont conservées avant combat ; le panneau **Groupe** permet la sélection manuelle par pseudo exact si la liste réseau manque en monde ouvert. Les joueurs extérieurs restent exclus et les identités absentes affichent « — ». Voir la procédure de diagnostic dans `docs/INSTALLATION.md`.

La capture ne démarre pas automatiquement et aucun paquet n’est envoyé au jeu. Il n’y a aucune lecture de mémoire, injection, automatisation du jeu, connexion à un compte ni upload.

**Compatibilité Global / Europe : la validation sur un combat réel en groupe reste à faire.** Les tests hors ligne vérifient le moteur et les formats de paquets ; ils ne prouvent pas que la version actuelle de ton client expose tous les événements nécessaires. L’overlay convient au jeu en fenêtre ou sans bordure ; le plein écran exclusif peut le masquer.

## Statistiques

- Une ligne par membre réel du groupe, et un résumé collectif. Les autres joueurs ne deviennent pas des lignes.
- Même durée pour le DPS et le HPS de tous : premier événement hostile jusqu’à la fin du combat ; minimum d’une seconde.
- Dégâts directs et périodiques, soins prodigués et reçus, dégâts reçus, morts observées, compétences et impacts. Les impacts multiples ne sont pas des lancements de compétence.
- Les soins sont les valeurs rapportées par le protocole. L’overheal et les soins effectifs ne sont pas estimés.
- Les invocations sont attribuées seulement lorsque le propriétaire est confirmé. Les identités de personnage ne sont pas déduites de sa classe seule.
- Les membres sortis restent dans le combat courant, avec la mention « a quitté le groupe ». Leurs actions après leur sortie ne sont pas ajoutées.
- Le filtre cible agit sur les dégâts ; les soins et dégâts reçus restent ceux du groupe pendant le combat. L’historique conserve les dégâts de **toutes** les cibles, indépendamment de ce filtre.
- Fin au décès de la cible suivie, au décès d’un boss combattu, au changement de zone détecté, à la déconnexion, à la fermeture du jeu, à l’arrêt manuel ou après dix secondes sans événement pertinent. Une téléportation interne pendant un combat n’est pas considérée automatiquement comme un changement de zone.

## Overlay et historique

Le bouton **▤** ou Réglages → **Mode compact** garde le temps de combat et les totaux collectifs. **Combat** affiche un classement coloré par classe avec total, taux/s et contribution ; les onglets passent des dégâts aux soins ou aux dégâts reçus. **Bandeau**, **Cartes** et **3 panneaux** permettent d’afficher les trois statistiques ensemble. Le choix est mémorisé et permet de revenir à la taille complète. Taille minimale : 320 × 180 en compact, 420 × 420 sinon. Déplacer la fenêtre par son titre ; redimensionner par ses bords. Opacité, interface réseau et maintien au-dessus du jeu sont réglables. Le verrouillage laisse les clics traverser : **icône AionPartyMeter près de l’horloge → Afficher et déverrouiller** pour retrouver les contrôles.

L’historique conserve les 200 derniers combats dans le dossier local de l’application. Le diagnostic permet d’ouvrir ce dossier. Les exports JSON comprennent compétences et courbes ; les CSV donnent un tableau par membre. Les sauvegardes sont atomiques. Un historique illisible est conservé sans écrasement et une erreur est affichée.

## Enregistrement et replay

Dans **Diagnostic**, démarrer un enregistrement réseau manuel avant de former le groupe ou de changer de zone. Les identités envoyées avant l’enregistrement ne peuvent pas être reconstituées après coup.

**Importer** accepte :

- un rapport JSON exporté, pour le consulter ;
- une capture `*.apm.jsonl`, pour la décoder hors ligne avec le même moteur ;
- le scénario `examples/groupe-synthetique.apm.jsonl`, pour vérifier l’interface. Il est explicitement marqué synthétique et n’est pas ajouté à l’historique réel.

Les captures brutes peuvent contenir des noms et messages du jeu. Elles restent sur ce PC. Le replay est accéléré et utilise les horodatages enregistrés, sans s’appuyer sur la date de relecture.

## Développer

Prérequis : Node 18+, Rust stable MSVC (édition 2024), Visual Studio Build Tools avec C++, Windows SDK. Npcap est nécessaire à la capture, pas à la compilation ni aux tests hors ligne.

```powershell
./dev.ps1 -Mode dev
./dev.ps1 -Mode test
./dev.ps1 -Mode build
./dev.ps1 -Mode replay -Capture ./examples/groupe-synthetique.apm.jsonl
```

`build` crée `src-tauri/target/release/AionPartyMeter.exe`. Le script réutilise la toolchain locale installée dans le dossier de travail de cette livraison si `cargo` n’est pas dans le PATH. Sur un autre PC, installer Rust normalement.

L’analyse technique et le rapport de vérification se trouvent dans `docs/ANALYSE.md` et `docs/VALIDATION.md`.

## Origine et licence

Moteur de protocole : [taengu/A2Tools-DPS-Meter](https://github.com/taengu/A2Tools-DPS-Meter), commit `82e53c1008ac4c2974446cc703703f473bbbed81`. Le fichier `LICENSE` est conservé. Les modifications, l’interface et les sources distribuées restent GPL-3.0-only. Les polices embarquées conservent leurs licences OFL.

Inspiration fonctionnelle : [jeojabon/JaMeter](https://github.com/jeojabon/JaMeter). Son dépôt ne publie pas son moteur ; aucun code ni visuel propriétaire de JaMeter n’a été repris. Npcap, Aion 2 et les données du jeu appartiennent à leurs auteurs respectifs.


## Version 0.1.7 — choix d’affichage et feedback visuel

Sélecteur **Affichage du meter** dans la barre du groupe : Bandeau (pseudo et trois totaux), Cartes (classe, totaux et taux), 3 panneaux (classement indépendant par statistique), Développé (onglets, tableau complet et graphiques). Détails et courbes au clic. Le choix est mémorisé localement ; le changement conserve la position de fenêtre et adapte sa taille.

La barre d’état distingue capture active, recherche du jeu, replay et rapport historique, avec activité des paquets, état du combat et pertes détectées. Survoler la capture ou le groupe pour les compteurs, la connexion et les identités confirmées/en attente ; cliquer pour le diagnostic. Aucun changement au moteur automatique, aux règles de groupe, à la capture ou au calcul des statistiques.

## Version 0.1.10 — Combat +, trois statistiques ensemble

Choisir **Combat +** dans le sélecteur d’affichage. Même style sombre et barres par classe que Combat, avec trois colonnes permanentes par membre : **Dégâts**, **Soins** et **Tank** (dégâts reçus observés). Chaque colonne affiche le total et le taux/s ; le résumé du groupe affiche les trois totaux et taux simultanément. Cliquer un titre de colonne trie les lignes et ajuste les barres sans masquer les autres statistiques. Les contributions précises restent au survol. Format initial 400 × 280 pour quatre membres, largeur minimale 320 pixels, détails et graphiques au clic. Le choix est mémorisé. Capture, calculs et détection automatique restent inchangés.

## Version 0.1.9 — affichage Combat

**Combat** reprend la référence fournie : fenêtre sombre, barres par classe, total, DPS/HPS et contribution par membre. Format de départ 360 × 230 pour quatre membres. Onglets Dégâts / Soins / Reçus, détails et courbes au clic, filtre cible et état de capture en bas. Le nouveau mode est adopté au premier lancement ; un choix explicite Cartes ou 3 panneaux est conservé. Les choix suivants restent mémorisés. Le moteur et la détection automatique ne changent pas. La vie de la cible et le ping ne sont pas exposés par le snapshot : aucune valeur n’est inventée.

## Version 0.1.8 — mises à jour GitHub

Le dépôt officiel est [Flwrian/FlowDPSMeter](https://github.com/Flwrian/FlowDPSMeter). **DEMARRER.exe** consulte la dernière release stable au lancement et installe une version plus récente avant d’ouvrir l’overlay. Sans Internet, en cas de vérification impossible ou après annulation, la version installée s’ouvre. Si le meter est déjà ouvert, aucun remplacement n’a lieu : le fermer puis relancer DEMARRER.exe.

Installer ce pack une première fois : les anciens lanceurs 0.1.7 et précédents ne savent pas se mettre à jour. Le petit ZIP Update sert au lanceur ; pour une nouvelle installation, télécharger le ZIP Windows complet. Les paramètres, historique et captures restent locaux. Aucun compte GitHub ni token n’est requis pour utiliser les mises à jour du dépôt public.

La configuration `update.json` est préremplie. `DEMARRER.exe --updates` permet de désactiver la vérification. `DEMARRER.exe --skip-update` ouvre sans vérification pour ce lancement. L’ouverture directe de `AionPartyMeter.exe` contourne le lanceur et sa vérification. Les mises à jour nécessitent un dossier accessible en écriture.

Pour publier les prochaines versions, suivre [le guide de release](packaging/windows/README.md). Le workflow prépare un brouillon avec le pack complet, le ZIP Update et les sources GPL correspondantes. Un simple push de sources ne déclenche pas une mise à jour chez les utilisateurs.
