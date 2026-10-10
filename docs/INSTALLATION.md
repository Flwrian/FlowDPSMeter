# Démarrer AionPartyMeter

## Nouvel affichage Combat — 0.1.9

Fermer le meter depuis son icône près de l’horloge → Quitter, extraire tout le pack **0.1.9**, puis ouvrir **DEMARRER.exe**. Le groupe en jeu peut rester formé. Réglages, captures et historique sont conservés. La mise à jour automatique utilise les releases stables publiées sur GitHub ; un brouillon n’est pas proposé au téléchargement.

- **Combat** reprend la disposition de la référence : fond sombre, barres colorées par classe, pseudo, total en petit, taux/s en gras et contribution à droite. Format initial **360 × 230** pour quatre membres ; les grands groupes défilent. Une barre remplie représente le meilleur montant du classement, tandis que le pourcentage représente la part du total collectif.
- Cliquer **Dégâts**, **Soins** ou **Reçus** pour changer le classement. Les soins restent ceux rapportés par le protocole, avec overheal inconnu. Cliquer un membre ouvre ses compétences et ses graphiques.
- La cible en haut filtre les dégâts pendant le combat. **Toutes les cibles** restaure le total. Les soins et dégâts reçus restent ceux du groupe. Le total affiché est celui observé par le meter ; la vie du boss et le ping ne sont pas disponibles dans cette vue.
- La barre du bas ouvre **Groupe** depuis le pseudo ou le compteur d’identités confirmées ; cliquer l’état de capture ouvre le diagnostic. Le bouton **▶ / ■** démarre ou arrête la capture. Les identités en attente affichent **—** et **?** ; **↪** marque les anciens membres conservés dans le combat. **PARTIEL** signale les pertes détectées ; **TEST** signale les données synthétiques.
- Le sélecteur en bas à droite permet de retrouver **Bandeau**, **Cartes**, **3 panneaux** ou **Développé**. Dans Développé, **Toutes les stats** ouvre le tableau complet et les trois courbes du groupe. Les trois statistiques restent simultanément visibles dans les autres dispositions compactes.
- Glisser le titre pour déplacer la fenêtre ; tirer les bords pour la redimensionner. Réglages d’opacité, maintien au-dessus du jeu et verrouillage traversant les clics restent accessibles depuis les icônes du titre. Pour déverrouiller, utiliser l’icône près de l’horloge → **Afficher et déverrouiller**.

Le mode Combat est adopté une fois au premier lancement de cette version si tu utilisais le Bandeau ou l’affichage par défaut. Un choix existant Cartes ou 3 panneaux est conservé. Ensuite, ton choix de disposition est respecté. Cette refonte ne modifie pas la capture, les calculs ou la découverte du groupe.

## Installation du pack

1. **Extraire tout le ZIP** avec « Extraire tout… ». Ne pas lancer l’application depuis l’intérieur du ZIP et ne pas déplacer l’exécutable seul.
2. Ouvrir **DEMARRER.exe**. Aucun Node.js, Rust ou Visual Studio n’est nécessaire pour utiliser le pack.
3. Si WebView2 manque, cliquer **Installer WebView2 (fourni)**. Son installateur Microsoft x64 complet est inclus ; cette étape fonctionne hors ligne.
4. Si Npcap manque, cliquer **Télécharger et installer Npcap**. Une connexion Internet est nécessaire pour ce premier téléchargement depuis `npcap.com`. Accepter la licence du pilote et l’invite administrateur de Windows. Les options par défaut conviennent ; le mode compatible WinPcap n’est pas obligatoire. Redémarrer si l’installateur le demande.
5. Cliquer **Ouvrir le meter**. Si Npcap a été installé avec un accès réservé aux administrateurs, utiliser **Ouvrir en administrateur**.

Une fois les deux dépendances présentes, **DEMARRER.exe** vérifie GitHub puis ouvre le meter. Le pilote et WebView2 restent installés sur le PC ; les fichiers Visual C++ nécessaires restent dans le dossier du pack. Internet n’est plus nécessaire au lancement du meter.

Npcap gratuit n’autorise normalement pas sa redistribution avec une autre application : il est donc téléchargé directement auprès de son éditeur au premier lancement. Le pack n’est pas totalement hors ligne sur un PC dépourvu de Npcap. [Licence Npcap](https://npcap.com/oem/redist)

Configuration : **Windows 10/11 x64**, avec les composants Windows habituels (.NET Framework pour le lanceur et Universal CRT). Les DLL natives Visual C++ requises sont incluses. Les dépendances Microsoft gardent leurs propres licences ; le meter et son lanceur sont GPL-3.0-only.

## En jeu

1. Lancer Aion 2 en fenêtre ou sans bordure.
2. Cliquer **Démarrer** dans le meter.
3. Former le groupe, entrer dans une instance ou se téléporter pour recevoir les identités des joueurs.
4. Combattre puis choisir **Dégâts**, **Soins** ou **Dégâts reçus**. Cliquer une ligne pour ouvrir les détails du membre.

La vue conserve tous les membres identifiés, même sans dégâts. « Identité en attente » signifie que ses statistiques restent inconnues. Les soins affichés sont ceux rapportés par le protocole ; l’overheal est inconnu.

## Si tu ne vois que ton personnage en monde ouvert (version 0.1.8)

1. Fermer l’ancien meter et ouvrir **DEMARRER.exe** du nouveau pack. Les réglages et l’historique existants sont conservés.
2. Démarrer la capture **avant de former le groupe**. La connexion peut maintenant être reconnue sur une liste de groupe, sans attendre le combat ; les premières identités ne sont plus jetées après trois secondes d’attente.
3. Dissoudre puis reformer le groupe et combattre à proximité du coéquipier. Les autres membres n’ont pas besoin d’installer le meter.
4. Ouvrir **Groupe**. Le compteur indique les listes de groupe effectivement décodées. S’il reste à zéro, saisir les pseudos exacts des coéquipiers, un par ligne, puis **Appliquer la composition**. Ton personnage reste inclus. Les joueurs observés à proximité ne sont jamais ajoutés automatiquement.
5. Si un membre affiche « — », son identité réseau n’a pas encore été reçue : cette version ne lui invente aucun dégât. Reformer le groupe ou se déplacer pour renouveler les informations peut aider. Pour diagnostiquer un format inconnu, enregistrer les paquets avant de reformer le groupe, puis arrêter l’enregistrement et transmettre son chemin.

La composition manuelle est sauvegardée. La mettre à jour à chaque changement de groupe ou choisir **Détection automatique** pour revenir à la liste reçue en jeu. Son activation ne récupère pas les attaques d’un joueur qui était exclu au moment de leur capture. Les sélections sont enregistrées dans les nouveaux replays pour conserver la même attribution.

Le correctif garde la composition après un changement de zone. La reconnaissance automatique du groupe sur ton client reste à valider avec une capture contenant sa formation ; le premier enregistrement reçu contenait les combats mais aucune liste décodable ni identité du coéquipier attendu.

**Global / Europe : un combat réel avec plusieurs joueurs et un soigneur reste nécessaire pour confirmer la compatibilité du protocole actuel.** Le scénario fourni vérifie le fonctionnement hors ligne, sans prouver cette compatibilité.

## Déplacer et ajuster la fenêtre

- **Déplacement :** maintenir le bouton gauche sur la barre de titre, puis glisser.
- **Taille :** glisser un bord ou un coin, notamment la poignée visible en bas à droite.
- **Recentrage :** roue dentée → **Recentrer la fenêtre et rétablir sa taille**.
- **Opacité et premier plan :** roue dentée → réglages → **Enregistrer les réglages**.
- **Verrouillage :** icône de verrouillage dans le titre. Les clics passent alors vers le jeu.
- **Déverrouillage :** clic droit sur l’icône AionPartyMeter près de l’horloge → **Afficher et déverrouiller**. Si l’icône est masquée, ouvrir les icônes cachées de Windows.

La position et la taille sont conservées à la fermeture normale. La capture démarre uniquement sur demande.

## Historique, exports et replay

- **Historique :** bouton horloge dans le titre, jusqu’à 200 combats locaux.
- **Exporter :** JSON complet ou tableau CSV ; le dossier de destination est indiqué après export.
- **Dossier local :** Diagnostic → Dossier local. Par défaut : `%LOCALAPPDATA%\local.aionpartymeter.app`.
- **Vérification hors ligne :** Importer → `exemples\groupe-synthetique.apm.jsonl`. Le scénario est marqué synthétique et ne rejoint pas l’historique réel.
- **Capture réelle :** Diagnostic → Enregistrer les paquets, après démarrage de la capture et avant de former le groupe. Importer ensuite le fichier `.apm.jsonl` pour le rejouer.

## Si rien n’apparaît

Ouvrir Diagnostic : vérifier Npcap, les permissions, la connexion détectée et les compteurs. Si Npcap est présent mais refuse la capture, relancer en administrateur. Si les identités manquent, reformer le groupe ou changer de zone. Essayer l’interface réseau utilisée par le jeu dans les réglages, puis arrêter et redémarrer la capture.

Une perte réseau ou une identité inconnue est signalée dans les informations sous le tableau. IPv6, certains VPN et les modifications du protocole peuvent nécessiter une adaptation du moteur.

Si une fenêtre disparaît après un changement d’écran, l’ancienne position hors écran n’est pas réutilisée au lancement. Le bouton de recentrage permet aussi de rétablir sa taille.

## Contenu

`DEMARRER.exe` est le lanceur ; `AionPartyMeter.exe` est l’overlay ; `vcruntime140.dll` et `vcruntime140_1.dll` sont les dépendances Visual C++ ; `dependances` contient l’installateur WebView2 ; `exemples` contient les données synthétiques ; `licences` contient les notices. Les sources sont distribuées séparément dans `AionPartyMeter-0.1.9-Sources.zip`.

Microsoft documente [la distribution de WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution) et [la distribution des DLL Visual C++](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files).

## Correctif 0.1.2 — identité des coéquipiers en monde ouvert

Les identités explicites contenues dans les paquets de butin/propriétaire sont désormais transmises au moteur de groupe, quand l’id, le serveur et le pseudo se suivent dans le format validé. Les anciennes recherches approximatives ne peuvent toujours pas attribuer les dégâts d’un membre. Une identité de butin ne prouve pas que son propriétaire est membre du groupe : utiliser Groupe pour sélectionner les coéquipiers si aucune liste n’est reçue.

Avec cette sélection active, Importer permet aussi de rejouer un ancien enregistrement ne contenant pas de composition manuelle. Une composition enregistrée dans le fichier prend priorité ; les scénarios synthétiques sont indépendants de ce réglage. Un rapport réel corrigé est livré séparément pour consulter les deux membres sans refaire le combat.

## Dégâts reçus — correctif 0.1.3

Fermer l’ancien meter puis ouvrir DEMARRER.exe de la version 0.1.3. La sélection de groupe est conservée. Démarrer la capture et choisir Dégâts reçus. Il n’est pas nécessaire de reformer le groupe si ses membres ont déjà une identité confirmée. Les attaques directes de monstres ne sont plus rejetées par leur code de compétence ; leurs dégâts sont attribués à la victime du groupe, avec détails et courbe. Les effets périodiques explicitement dommageables d’un monstre connu sur un joueur connu sont aussi conservés, sans dépendre de la liste de compétences périodiques des joueurs.

Le nouvel enregistrement réel a permis de vérifier deux attaques sur Yokosoo : 518 et 677, soit 1 195 dégâts reçus. Son identité étant absente de ce fichier court, le replay a utilisé les identités des captures antérieures du même flux. En direct, le meter doit d’abord identifier les personnages ; démarrer la capture puis se téléporter si une identité reste inconnue. Pour un replay autonome, enregistrer avant l’arrivée des identités. Le contrôle visuel du nouveau binaire en combat reste à effectuer.

## Fluidité, mode compact et ajout rapide — 0.1.4

Quitter l’ancien meter depuis son icône près de l’horloge, puis ouvrir DEMARRER.exe du nouveau pack. Les réglages et la composition mémorisée sont conservés.

- **▤ dans la barre de titre** ou Réglages → Mode compact : temps commun, taux et total collectifs, une ligne par membre avec pseudo/taux/total. Les onglets Dégâts, Soins et Dégâts reçus restent disponibles. Le mode est mémorisé ; un second clic restaure la taille complète. Déplacement, redimensionnement, opacité et verrouillage restent accessibles.
- **Groupe → + Ajouter** : ajout immédiat du membre découvert, sans devoir appliquer une seconde fois. Les autres membres et le combat courant sont conservés. Le panneau reçoit les nouvelles identités en direct sans effacer une saisie en cours.
- **Groupe → Ajout rapide** : pseudo exact puis Ajouter ou Entrée. Si l’identité réseau manque, le membre est visible avec « — » jusqu’à réception d’une liaison fiable. Une classe ou la proximité ne suffisent pas à attribuer ses dégâts.
- La découverte passive ne provoque aucun nouvel envoi du serveur. Si une identité a été envoyée avant le démarrage du meter, combattre ensemble avec ramassage de butin ou sortir puis revenir dans la portée d’affichage peut fournir une nouvelle identité. Il n’est pas nécessaire de quitter le groupe. Les captures Global/Europe reçues ne contiennent toujours pas de liste de groupe décodable : la sélection des coéquipiers reste explicite dans ce cas.
- Dans la même session du meter, arrêter puis redémarrer la capture en moins de 30 secondes conserve les identités de la même connexion, tout en démarrant un nouveau combat. Un autre flux ou une déconnexion prolongée repart sans anciennes liaisons. Fermer entièrement l’application ne conserve pas ces identifiants temporaires.
- Modifier la composition complète permet de retirer un membre, de passer en solo ou de rétablir la détection automatique. Seuls les membres sélectionnés ou confirmés par une liste réseau sont comptés. Les actions antérieures à un ajout manuel ne sont pas récupérées rétroactivement.

Les détails des compétences et des courbes sont calculés à leur ouverture ; l’historique et les exports restent complets. Les contrôles visuels fournis utilisent un scénario synthétique et ne constituent pas une preuve en combat réel.

## Tableau complet et graphiques — 0.1.5

Quitter l’ancien meter depuis l’icône près de l’horloge → Quitter, puis lancer **DEMARRER.exe** du pack 0.1.5 extrait. Les réglages, le groupe sélectionné et l’historique existants sont conservés.

- Cliquer **Toutes les stats** dans les onglets : une ligne par membre avec classe, dégâts, DPS, contribution, soins prodigués, HPS, soins reçus, dégâts reçus et morts observées. Les membres sans dégâts et les anciens membres restent visibles. Les identités non reçues affichent « — ».
- Cliquer un en-tête pour trier ; un second clic inverse le tri. Cliquer le pseudo pour ouvrir les compétences et la courbe individuelle. La ligne **Total observé** additionne les membres ; une capture partielle reste signalée.
- Trois courbes du groupe sont affichées ensemble : DPS, HPS et dégâts reçus par seconde. Leur durée commune est celle du combat. Survoler un intervalle pour voir son taux et son montant. Les intervalles sans événement compté valent zéro ; les avertissements de pertes ou d’identités manquantes s’appliquent aussi aux graphiques.
- Les points proviennent d’intervalles de capture d’une seconde, regroupés jusqu’à 160 intervalles pour les combats longs. Les courbes indiquent des taux par intervalle, tandis que les DPS/HPS du tableau sont les moyennes du combat. Les soins restent rapportés par le protocole ; l’overheal est inconnu.
- Le filtre cible agit sur les dégâts du tableau et de la courbe DPS. Les soins et dégâts reçus restent ceux du groupe. L’historique et les exports conservent toutes les cibles.
- La vue complète agrandit une petite fenêtre à 1120 × 780 et la recentre. **Agrandir la vue** permet de retrouver cette taille. La barre de titre et les bords permettent toujours de déplacer et redimensionner la fenêtre. Sur une petite fenêtre, faire défiler le tableau horizontalement.
- **Masquer les graphiques** garde le tableau seul ; **Afficher les graphiques** les rétablit. Le choix est mémorisé. Le bouton **▤** revient au mode compact ; les onglets Dégâts / Soins / Dégâts reçus restent disponibles. Cliquer Toutes les stats pour rouvrir le tableau complet.
- Les rapports de l’historique et les JSON importés peuvent être consultés dans cette vue. Un ancien rapport sans série temporelle affiche « Aucune série temporelle enregistrée ».

Les lignes sont actualisées quatre fois par seconde sans reconstruire le tableau. Les courbes sont dessinées au plus une fois par seconde pendant le combat ; les détails des compétences sont calculés à la demande.

## Compact simultané, réactivité et groupe — 0.1.6

Quitter l’ancien meter par l’icône près de l’horloge → Quitter, puis ouvrir **DEMARRER.exe** du pack 0.1.6 extrait. Aucun changement de groupe en jeu n’est nécessaire. Les réglages et l’historique existants sont conservés.

- **▤** dans le titre ou Réglages → Mode compact : pseudo, dégâts, soins prodigués et dégâts reçus restent visibles ensemble. Chaque cellule indique le total et le taux/s ; les totaux du groupe restent en bas. Cliquer le titre d’une colonne pour classer les membres selon cette statistique. Cliquer un membre pour ses détails.
- La fenêtre compacte passe à environ **400 × 300** pour quatre membres. Elle reste redimensionnable, avec un minimum de 320 × 180 ; un grand groupe ou une petite hauteur fait défiler les lignes sans masquer les totaux. Glisser le titre déplace la fenêtre. Le second clic sur ▤ revient à la vue développée avec les onglets et les graphiques.
- **Réglages → Actualisation** : 10 mises à jour/s par défaut, 4 pour réduire le travail ou 20 pour une actualisation plus fréquente. Les valeurs dépendent toujours des événements reçus et du temps de calcul ; aucune interpolation n’ajoute de dégâts. Les détails de compétences sont limités à 4/s et la fenêtre masquée à 1/s. La capture continue indépendamment de l’affichage.
- **Animations discrètes** : légère mise en évidence des nouveaux montants et transition des barres. Désactivable dans Réglages ; la préférence Windows de réduction des mouvements est respectée.
- La barre du groupe indique la sélection courante et le nombre d’identités confirmées. « Vous seul suivi » signifie que le meter ne suit actuellement que ton personnage, sans prétendre que ton personnage n’est pas en groupe dans le jeu. Une identité non reçue reste « — ».
- **Gérer le groupe +** ouvre les membres suivis, leur état d’identification et les joueurs découverts. Ajouter un coéquipier en un clic ou par pseudo exact ; **Retirer ×** enlève uniquement son suivi. Les coéquipiers n’ont rien à installer. La sélection est mémorisée.
- **Solo** suit ton personnage et ses invocations confirmées. **Auto** utilise les listes de groupe reconnues. En leur absence, ajouter explicitement les coéquipiers : une identité de joueur à proximité n’est pas une preuve qu’il appartient à ton groupe.
- Entrées, retraits et passage en solo conservent la participation antérieure des membres au combat courant ; leurs événements suivants sont exclus après retrait. L’état « Membres suivis maintenant » est indépendant du dernier combat et des rapports historiques ouverts.

Les captures Global/Europe reçues ne contiennent toujours pas de liste de groupe décodable. Une identité peut aussi manquer si elle a été envoyée avant le démarrage de la capture. Combattre ensemble, ramasser du butin ou se rapprocher peut la faire réapparaître ; quitter puis recréer le groupe n’est pas requis. Les inconnues et pertes restent signalées.


## Affichages et indicateurs — 0.1.8

Quitter l’ancien meter depuis son icône près de l’horloge → Quitter. Extraire le pack 0.1.8 puis ouvrir **DEMARRER.exe**. Le groupe en jeu n’a pas besoin d’être reformé. Les données et réglages existants sont conservés ; la détection automatique est identique à la version 0.1.6.

Dans la barre du groupe, choisir **Bandeau**, **Cartes**, **3 panneaux** ou **Développé** :

- **Bandeau** : le format le plus petit, pseudo + dégâts, soins et dégâts reçus. Le taux/s et le montant exact restent au survol ; cliquer un membre ouvre ses détails et courbes. Environ 400 × 256 pour quatre membres.
- **Cartes** : une tuile par membre avec classe/état et trois totaux + taux/s. Deux colonnes, ou une seule à petite largeur. Environ 460 × 324 pour quatre membres.
- **3 panneaux** : dégâts, soins et reçus côte à côte, chacun avec total collectif, taux/s et classement indépendant. Cliquer un membre dans Soins ouvre directement les compétences et la courbe de soins. Environ 780 × 344 pour quatre membres. Les panneaux s’empilent quand la largeur est réduite.
- **Développé** : retrouve les onglets et **Toutes les stats** pour le tableau complet et les trois graphiques. Le bouton ▤ revient à l’affichage compact choisi.

Le choix est mémorisé. Glisser la barre de titre pour déplacer la fenêtre ; les bords restent redimensionnables, ainsi que l’opacité, le verrouillage et les animations dans Réglages. Les grands groupes défilent dans les listes ; les résumés collectifs restent visibles.

**Feedback de détection** : capture active/recherche du jeu, combat/attente/terminé, petite lumière d’activité liée à une hausse du compteur de paquets et badge **Partiel** lors de pertes détectées. L’absence de nouveaux paquets ne signifie pas que la capture est arrêtée. L’indication de délai mesure ce que l’affichage a observé, pas un horodatage précis du dernier paquet capturé.

Survoler l’indicateur de capture pour la connexion, les paquets traités, les pertes et les erreurs ; cliquer pour ouvrir Diagnostic. Survoler la barre du groupe pour les noms et le nombre d’identités confirmées/en attente. Une identité absente reste « — » ; elle ne déclenche pas à elle seule le badge de pertes. Replay et historique sont explicitement identifiés, même si une capture continue en arrière-plan.

Cette version modifie uniquement la présentation et les préférences d’affichage. Les indications des anciennes sections sur les fichiers de test sont historiques ; la composition automatique actuelle reste celle de la version qui fonctionne sur ton client.

## Mises à jour automatiques — à partir de 0.1.8

1. Pour activer cette fonction depuis 0.1.7 ou une version précédente, fermer le meter puis extraire le nouveau pack complet 0.1.8 dans un dossier normal accessible en écriture. Ouvrir son **DEMARRER.exe**. Cette installation initiale est manuelle ; les suivantes peuvent être automatiques.
2. Au lancement, le lanceur consulte les releases stables de **Flwrian/FlowDPSMeter**. Si une version plus récente existe avec son ZIP Update valide, il la télécharge, vérifie son SHA256 et celui de chaque fichier, remplace l’application et se relance. La capture démarre toujours sur demande.
3. Garder le dossier du pack dans Documents ou un autre emplacement accessible en écriture. Les réglages, les captures et les 200 combats locaux sont conservés. Les installateurs de dépendances ne sont pas retéléchargés à chaque version.
4. Sans connexion ou si le téléchargement échoue, la version installée s’ouvre. **Ouvrir sans mise à jour** permet d’annuler la recherche ou le téléchargement. Une installation interrompue est restaurée au lancement suivant avant ouverture.
5. Si le meter est déjà ouvert, fermer sa fenêtre avant de relancer le lanceur. Aucun processus du jeu ou du meter n’est fermé par le système de mises à jour.

Le fichier `update.json` situé à côté de DEMARRER.exe contient `repository: "Flwrian/FlowDPSMeter"` et `enabled: true`. Pour les modifier dans une fenêtre, exécuter `DEMARRER.exe --updates`. `DEMARRER.exe --skip-update` ignore la vérification une seule fois. Le résultat est journalisé dans `%LOCALAPPDATA%\AionPartyMeter\update-last-result.txt`.

Télécharger le ZIP **Windows-x64** pour une première installation. Le ZIP **Update-Windows-x64** est destiné au lanceur et ne contient pas l’installateur WebView2. Les sources seules ou une release brouillon/préversion ne déclenchent pas l’installation automatique. Aucun compte utilisateur, token GitHub ou donnée de combat n’est envoyé par le lanceur ; il contacte GitHub pour la version et les fichiers de la release.
