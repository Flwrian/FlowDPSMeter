# Analyse et décisions techniques

## Présentation Combat — 0.1.9

Le composant `src/combat-meter.ts` consomme le même snapshot que les autres dispositions. Il conserve une ligne DOM par clé de membre, met à jour les textes et les largeurs de barres et trie une copie des lignes. Les symboles SVG sont originaux et les noms reçus sont insérés comme texte. Le taux des dégâts reçus utilise la durée commune avec un minimum d’une seconde ; les DPS/HPS sont repris du snapshot. Une identité non confirmée affiche des valeurs inconnues. Les anciens membres restent distingués ; aucune attribution ou composition n’est modifiée par l’affichage.

Les barres expriment le montant relativement au meilleur membre de la statistique ; les pourcentages utilisent son total collectif. La transition de largeur dure 140 ms et respecte l’option Animations ainsi que la réduction des mouvements du système. Les valeurs numériques ne sont pas interpolées. Le filtre cible appelle la commande existante, avec les mêmes restrictions que la vue développée ; la carte ne simule ni vie de cible ni ping.

Une préférence locale marque l’adoption du nouveau mode au premier démarrage, en conservant les choix existants Cartes/3 panneaux. Les commandes de compact et géométrie adaptent uniquement la présentation. La barre de titre garde les gestes natifs et ignore les boutons lors d’un déplacement. Les préférences de capture et de groupe ne sont pas réécrites par cette migration.

## Ce que JaMeter publie

Le dépôt [JaMeter](https://github.com/jeojabon/JaMeter) contient un README et des images, pas le moteur de décodage. Le README annonce une capture passive Npcap, un overlay de DPS et contribution, des détails de compétences, des onglets de dégâts reçus et soins reçus ainsi que des fonctions de partage. Ces informations décrivent le produit ; elles ne permettent pas de connaître son langage, ses structures internes, ses opcodes ou ses formules exactes.

## Ce qui a été lu dans A2Tools

L’implémentation examine le commit `82e53c1008ac4c2974446cc703703f473bbbed81` de [taengu/A2Tools-DPS-Meter](https://github.com/taengu/A2Tools-DPS-Meter).

La chaîne du moteur ouvert comprend extraction TCP, accumulation du flux, délimitation des messages par longueur varint, décompression des blocs LZ4 `FF FF`, analyse des enregistrements et suivi des entités. Le protocole comprend notamment dégâts `04 38`, effets périodiques `05 38`, roster `02 97`, identités et liens d’invocations de la famille `36`. Les formats peuvent évoluer avec les patches.

Le parseur distingue les effets de dégâts périodiques `02/0A` des soins `01/09/0B`. La version amont conserve les soins par soigneur et compétence, mais perd le destinataire à cette étape. Les soins sur alliés traversent aussi les enregistrements de type dégâts et sont reclassés selon les entités. Les suffixes de vol de vie constituent un autre cas. Les restaurations de mana sont exclues à partir de la table de compétences.

## Adaptations AionPartyMeter

- `party.rs` : modèle `CombatEvent`, participation au groupe au moment de l’événement, durée commune, attribution confirmée des invocations, totaux collectifs et détails par membre. Les identités obtenues tardivement peuvent rattacher les événements encore en attente.
- `pipeline.rs` : reconstruction TCP par séquence avant le framing. Retransmissions et chevauchements ne créent pas d’événements supplémentaires. Une lacune non résolue pendant deux secondes déclenche une resynchronisation signalée comme partielle. Les connexions sont séparées par adresses et ports.
- `DataStorage` : file d’observation optionnelle, placée avant les politiques d’agrégation par boss/mannequin amont. Les soins exposent désormais leur cible. Les identités utilisées pour le groupe proviennent des sources explicites, pas de l’ancienne liaison par classe.
- `desktop.rs` : capture locale, sélection de connexion, historique atomique, exports et contrôles de fenêtre. Les candidats réseau sont isolés jusqu’à la détection du flux ; la capture s’appuie sur la présence de la fenêtre Aion 2 et sur les signatures amont.
- Interface TypeScript : tableau du groupe, onglets de métriques, résumé, détails, historique, diagnostic et replay. Les données restent calculées par Rust.

Les modules de compte, uploads, partage, serveur de streaming, présence Discord, installation automatique Npcap et updater amont ont été retirés de l’application. La CSP interdit les connexions externes et les dépendances HTTP de ces modules sont absentes du manifeste.

## Limites explicites

### Correctif groupe 0.1.1

Un enregistrement réel de 544 segments sur environ 27 secondes a reproduit le défaut : dégâts locaux décodés, aucune liste `02 97` valide et aucune identité du coéquipier attendu. Aucune perte TCP n’a été mesurée dans ce fichier. Cela établit l’absence d’informations suffisantes dans l’enregistrement ; cela ne prouve pas que le serveur ne les envoie jamais.

Deux défauts ont été corrigés : les candidats de connexion jetaient les identités après trois secondes sans combat, et le moteur perdait les membres après un changement de zone. La sonde reconnaît maintenant une liste valide même comprimée/fragmentée avant combat, et conserve au maximum 60 secondes, 2 048 segments et 1 Mo de données par candidat. Toute éviction est signalée comme partielle une fois le flux sélectionné. Les 64 candidats maximum sont isolés ; seuls les octets de la connexion reconnue sont envoyés au moteur.

Le panneau Groupe affiche le compteur des listes effectivement décodées, les identités observées et une composition manuelle explicite par pseudo exact. Un joueur à proximité n’est jamais membre par déduction de sa classe ou de sa cible. Sans identité correspondante, les valeurs restent inconnues. Les changements manuels sont sauvegardés localement et inclus comme configuration explicite dans les replays, distincte d’une liste décodée du réseau. Une nouvelle capture démarrée avant la formation du groupe reste nécessaire pour adapter et confirmer son format actuel en monde ouvert.

- La compatibilité réelle avec le client Global / Europe actuel n’a pas été validée.
- Le réseau capturé est celui exposé localement par Npcap et le moteur amont, principalement TCP/IPv4. Certains VPN, trafic IPv6 ou futures protections de protocole peuvent exiger une adaptation.
- Un membre connu dans le roster, mais sans identité réseau confirmée, reste affiché sans attribution inventée. Les inconnus ne sont pas intégrés aux totaux et une information apparaît.
- Les événements en attente sont limités à 10 000, ceux d’un combat à 150 000. Le dépassement est signalé comme partiel, comme une saturation du canal de capture.
- Les données visibles dépendent des événements reçus par ton client : proximité, début de capture, déconnexions et pertes peuvent réduire leur couverture.
- Les soins effectifs, overheal, nombre de lancements et nDPS ne sont pas dérivés de données insuffisantes.

La lecture passive ne constitue pas une approbation par l’éditeur du jeu. L’application n’injecte rien et ne modifie aucun paquet ni fichier du client.

## Correction des identités de propriétaire (0.1.2)

La capture suivante a montré le format `04 8D <cible varint> <champ u32> <propriétaire varint> <serveur u16> <longueur u8> <pseudo UTF-8>`. Le décodeur lisait le pseudo par `append_nickname`, mais `authoritative_names` ne le fournissait pas à PartyEngine. La confirmation exige maintenant que le propriétaire commence directement après le champ fixe et soit immédiatement suivi du serveur et du pseudo validés. Un résultat de la recherche amont dans les 128 octets suivants reste non confirmé. La confirmation ne rattache pas automatiquement ce joueur au groupe et ne modifie pas le propriétaire d’une invocation non confirmée.

La composition humaine explicite est nécessaire dans la capture étudiée, qui ne comporte aucun roster valide. La sélection peut initialiser un ancien replay ; les sélections intégrées au fichier prennent ensuite priorité. L’outil CLI accepte `--party=pseudo1,pseudo2`, `--all-fights` et `--diagnostic`.

## Dégâts reçus (0.1.3)

Le rejet des identifiants de compétences NPC `1_000_000..=9_999_999` dans `parsing_damage_inner` provenait du périmètre DPS sortant de l’amont. Les compétences NPC utilisent le même layout de valeur et de plotter ; le signal normalisé doit atteindre PartyEngine pour être attribué à sa cible, indépendamment de l’existence d’une cible déjà attaquée. La liste de DoT des joueurs s’applique encore aux sources non NPC ; un effet dommageable explicite venant d’un monstre connu, hors invocation, vers un joueur connu est conservé pour les dégâts reçus. Aucun dégât n’est déduit d’une baisse de HP : celle-ci ne permet pas à elle seule de distinguer une attaque, une variation du maximum ou un autre effet.

## Fluidité et découverte (0.1.4)

`dashboard(detailMember)` utilise la projection résumée de PartyEngine par défaut. Les compétences et les points ne sont pas agrégés pour cette projection. Une demande de détail garde les tableaux du membre choisi ; les exports/historique utilisent toujours `snapshot`. Le cache est borné à un snapshot et ne dépend pas d’un timer : sa durée et ses taux sont réactualisés sans refaire les sommes. Les identités identiques ne provoquent pas une nouvelle liaison à chaque paquet. La recherche dans les événements pour déterminer si une cible a été combattue est limitée aux décès. Les mises à jour de lignes du DOM sont faites par clé et texte ; une ligne n’est pas recréée sur chaque tick. Les gestes de fenêtre sont regroupés par frame, avec au plus une commande de transformation en vol et conservation de la position finale.

Le panneau Groupe conserve les champs ouverts et actualise séparément la liste des identités confirmées. `party_add` prend la composition active et ajoute le pseudo explicitement choisi ; il ne transforme jamais une identité observée à proximité en preuve d’appartenance. Il conserve les autres membres, attend la liaison exacte si elle manque et enregistre le changement de sélection dans un replay en cours. Le stockage des pseudonymes choisis persiste entre lancements ; les identifiants d’entité temporaires ne persistent pas sur disque. Une reprise de capture en moins de 30 secondes dans la même session conserve le stockage courant, redémarre le TCP/framing et le combat ; `Pipeline::feed` efface ce contexte si le tuple de connexion change. Une déconnexion prolongée repart avec un stockage neuf.

Le compact est un réglage rétrocompatible. Il réduit la taille minimale à 320 × 180 pixels logiques ; la taille complète précédente est conservée séparément pour le retour. La géométrie native et les calculs des gestes utilisent les mêmes minimums en pixels physiques après conversion DPI. Les réglages d’opacité/interface ne remplacent plus la composition et la géométrie à partir d’un ancien snapshot UI.

## Tableau complet et graphiques (0.1.5)

La projection du moteur a trois modes : Summary (totaux), Timeline (totaux et intervalles par seconde, sans compétences) et Full (archives, exports et détail ouvert). Le cache unique dépend du mode et du statut de capture partielle. Les événements, identités, compositions et filtres continuent à l’invalider. `dashboard(overview, detailMember)` retourne un `groupTimeline` séparé, somme des points des membres, avant de retirer les séries individuelles inutiles de sa réponse. Les compétences ne sont calculées que pour une demande de détail ; les séries restent complètes dans l’historique et les exports.

Le tableau utilise des lignes DOM identifiées par clé et dix colonnes triables ; les identités non confirmées gardent leurs huit cellules numériques inconnues et se placent en fin de tri numérique. Le footer additionne les membres et utilise la même durée pour DPS/HPS. Les membres sortis conservent leur participation antérieure. Les totaux restent observés, avec les avertissements d’incomplétude.

Les trois SVG dessinent des taux par intervalles sur un axe de temps commun. Des intervalles entiers de capture sont regroupés jusqu’à 160 buckets ; le dernier intervalle peut être partiel. L’intégrale taux × durée reproduit la somme des montants observés. Les événements exactement à la borne finale sont conservés dans le dernier intervalle. Aucun lissage ne crée d’impacts ou de soins intermédiaires. Les graphes n’utilisent pas d’identifiant de gradient commun qui pourrait créer une collision entre SVG. Le survol expose les bornes, le taux et le montant de chaque intervalle.

Le filtre cible est appliqué par le moteur avant de produire le tableau et les points DPS ; les soins et dégâts reçus restent ceux du groupe. Les rapports importés et archivés fournissent leurs propres séries, sans mélanger le combat en direct. Les courbes sont redessinées environ une fois par seconde, avec actualisation immédiate aux changements de combat, de filtre et à la première disponibilité des données. Les lignes continuent à quatre mises à jour par seconde. La vue complète s’agrandit sur demande et conserve déplacement, redimensionnement, compact et verrouillage.

## Compact simultané et groupe courant (0.1.6)

Le compact est un composant distinct du tableau développé : une ligne DOM stable par membre, trois cellules toujours visibles (dégâts, soins, reçus) avec montants et taux/s. Les valeurs sont formatées à partir du snapshot, sans tween numérique. Les nouvelles hausses de montants peuvent déclencher une mise en évidence de 180 ms ; les barres transitent en 120 ms. La préférence locale et `prefers-reduced-motion` désactivent ces effets. Les identités non confirmées gardent trois valeurs inconnues et se placent en fin de tri.

Les commandes dashboard sont sérialisées, avec une seule requête en vol. La boucle vise 100 ms par défaut, configurable 250/50 ms ; les détails imposent au moins 250 ms et `document.hidden` impose 1000 ms. Un délai minimal de 8 ms évite une boucle serrée de rattrapage quand le travail dépasse l’intervalle. La capture native et le moteur restent indépendants de cette fréquence. Les graphes conservent leur fréquence inférieure et les projections résumées restent utilisées en compact.

`PartyEngine::party_members()` projette uniquement les membres actuellement présents dans la sélection, avec confirmation d’identité. Ce champ séparé du snapshot évite d’afficher l’ancien roster d’un combat figé dans les contrôles de composition. `party_remove` lit la sélection courante sous le verrou du pipeline et passe par le même enregistrement atomique/routage replay que l’ajout. Retirer un membre marque son départ dans le combat courant ; ses événements ultérieurs sont exclus sans effacer ses montants précédents. Le mode Solo correspond à une liste explicite vide, avec personnage local/invocations confirmées conservés par le moteur. Auto restaure le roster décodé courant. Les joueurs proches ne sont pas ajoutés automatiquement.

Les messages de démarrage ne présentent plus le groupe comme un prérequis au fonctionnement solo. « Vous seul suivi » indique l’état du meter quand aucun roster réseau n’a été reçu, sans affirmer que le personnage n’est pas groupé dans le jeu. Le statut des identités, la liste suivie, Auto/Solo, ajout et retrait sont accessibles sans quitter le groupe. Ces améliorations ne prétendent pas reconstituer des octets d’identité ou de roster absents des captures.


## 0.1.7 — présentation seulement

`visual-layout.ts` réutilise le snapshot et les cellules existantes : bandeau et cartes partagent les lignes stables ; les trois panneaux gardent trois listes stables indépendamment triées. Aucun événement n’est ajouté et les montants sont ceux du moteur. Les nouveaux contrôles d’affichage n’appellent que les commandes de compacité et de géométrie déjà disponibles, avec conversion DPI.

`capture-feedback.ts` interprète les diagnostics et identités déjà exposés. L’activité nécessite une augmentation observée du compteur après établissement d’une référence ; elle est réinitialisée lors d’un changement de flux/mode ou d’une remise à zéro. Les erreurs, pertes, archives et replay sont distincts. Aucun nouvel état de détection ne remplace celui du moteur. Les fichiers Rust de production et de test sont identiques octet par octet à ceux présents avant cette intervention.

## 0.1.8 — mise à jour du lanceur

Le nouveau client HTTP est limité au lanceur .NET Framework. La CSP de l’overlay et le moteur Rust restent inchangés. L’API publique GitHub `/repos/Flwrian/FlowDPSMeter/releases/latest` fournit la release stable ; le tag doit être `vX.Y.Z` ou `X.Y.Z`, supérieur à la version installée. Le nom de l’asset est exact : `AionPartyMeter-X.Y.Z-Update-Windows-x64.zip`. Le téléchargement est limité à HTTPS et aux hôtes GitHub autorisés, sans compte ni token.

Le SHA256 de l’asset fourni par GitHub est vérifié avant extraction, puis un manifeste de hashes vérifie chaque fichier. L’archive est bornée en taille et nombre de fichiers ; chemins absolus, traversées, doublons et fichiers de données locales sont refusés. Seuls les binaires, DLL Visual C++, guides et licences autorisés sont remplacés. La version du binaire doit correspondre au tag de release. Le manifeste garantit l’intégrité interne, pas une signature éditeur indépendante : la confiance repose sur HTTPS et le contrôle du dépôt GitHub.

La préparation ne touche pas à l’installation. Un assistant temporaire attend la sortie du lanceur, prend le verrou d’installation, sauvegarde les anciens fichiers et écrit un journal durable avant remplacement atomique fichier par fichier. En cas d’échec, il restaure l’ancienne version. Après interruption de processus, le lancement suivant reprend la restauration avant d’ouvrir l’overlay. Les captures, réglages, historique et la configuration du dépôt ne sont jamais dans la liste de remplacement. Le meter déjà ouvert bloque l’installation ; aucun processus n’est terminé.

Le workflow GitHub compile Windows, vérifie frontend et lanceur, vérifie les signatures Microsoft et produit le pack complet, le ZIP Update et les sources. Un tag prépare un brouillon avec ces trois assets via le token limité au dépôt, sans compte ni token sur le PC utilisateur. Il ne publie pas de release stable automatiquement : la publication du brouillon est le point de diffusion. Le pack complet local réutilise les dépendances Microsoft de la distribution précédente après vérification des SHA256. Npcap reste téléchargé auprès de son éditeur après clic et acceptation de licence.
