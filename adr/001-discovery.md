ADR : Service de découverte d’API (équivalent Google Discovery) pour France-Nuage
=================================================================================

Contexte et Enjeux
------------------

France-Nuage souhaite mettre en place un service de **catalogue des APIs** similaire à Google API Discovery. Ce service de découverte permettra de **recenser, maintenir et exposer les métadonnées** des différentes API proposées, afin que développeurs et outils puissent en obtenir la description de façon automatisée. L’objectif est de fournir un point centralisé où chaque API est décrite de façon assez riche pour en **générer du code client** et des intégrations, notamment : des définitions d’interfaces gRPC (`.proto`) et du code pour un provider Terraform correspondant.

Ce besoin s’inscrit dans une optique de **standardisation et d’automatisation** : en enregistrant les détails des endpoints, schémas de données et opérations pour chaque service, on évite la duplication d’effort (écriture manuelle de specs différentes pour gRPC, REST, Terraform…). De plus, en exposant ces métadonnées via une API RESTful de découverte, on facilite la **découverte dynamique** des services (à l’instar du catalogue d’API Google[dev.to](https://dev.to/schttrj/accessing-the-google-api-discovery-api-and-its-associated-discovery-documents-48aj#:~:text=%22kind%22%3A%20%22discovery,Shortener%20API%20lets%20you%20create)). Le système doit également s’intégrer dans l’écosystème France-Nuage (technologies open-source, standards ouverts) et respecter des bonnes pratiques de persistance et de conception d’API.

Options Envisagées
------------------

Plusieurs approches ont été étudiées pour construire ce service :

*   **Utiliser un format standard existant (OpenAPI/Swagger)** : La spécification OpenAPI est largement utilisée pour décrire les API REST et bénéficie d’un large outillage. Cependant, OpenAPI ne correspond pas exactement au format Google Discovery souhaité et n’intègre pas nativement la génération de `.proto`. De plus, l’objectif est de **reproduire le comportement de Google Discovery** pour compatibilité et éventuellement réutiliser certains outils Google (par ex. convertisseurs ou clients existants). OpenAPI a donc été écarté en faveur du format Google Discovery, plus adapté ici.
    
*   **Stocker les métadonnées sous forme de documents JSON** (par ex. dans un document store ou directement dans une colonne JSONB PostgreSQL) vs **modèle relationnel** : Le format natif des documents de découverte est JSON, ce qui plaide pour une solution document. Toutefois, la mise à jour partielle de ces documents et la garantie de cohérence référentielle (par ex. qu’une méthode référence bien un schéma existant) seraient plus complexes. Un **modèle relationnel structuré** facilite les jointures, la validation des références, et des requêtes efficaces pour reconstruire les documents de découverte. PostgreSQL prenant en charge JSON, on aurait pu stocker directement les documents, mais cela compliquerait la génération ciblée de `.proto` ou de code Terraform (il faudrait alors parser le JSON côté applicatif). Le choix s’est donc porté sur une **base relationnelle** bien structurée.
    
*   **Ne pas implémenter de service de découverte interne** : On aurait pu envisager de simplement fournir des fichiers `.proto` et des modules Terraform manuellement pour chaque API, sans passer par un service de découverte. Cette option est moins flexible (mise à jour manuelle fastidieuse, pas de découverte dynamique) et contraire à l’objectif de centralisation. De plus, sans base de données unifiée, la cohérence entre différentes représentations (REST, gRPC, Terraform) serait difficile à maintenir.
    

Après analyse, **la création d’un service dédié, inspiré de Google Discovery, stockant les métadonnées en base relationnelle** a été retenue. Cette solution offre un bon compromis entre standardisation (format de sortie identique à Google) et efficacité interne (modèle SQL robuste, génération automatisée des artefacts).

Décision d’Architecture Adoptée
-------------------------------

Nous décidons de mettre en place un **service de découverte d’API** custom, avec les caractéristiques suivantes :

*   **Base de données PostgreSQL** pour stocker toutes les définitions d’API. Le schéma SQL est soigneusement conçu : on utilise des _schemas_ PostgreSQL pour organiser les tables par domaine, des tables de référence (lookup tables) pour les petites énumérations (plutôt que le type ENUM natif), et on emploie le type `TIMESTAMPTZ` pour tous les horodatages afin de gérer proprement les fuseaux horaires.
    
*   **Modèle de données riche** couvrant : APIs (services) et versions, ressources (groupements de endpoints), méthodes (endpoints individuels avec verbe HTTP), schémas de données (messages JSON), champs (propriétés des schémas), paramètres (query/path de méthodes), et opérations (typologies d’appels CRUD). Ce modèle détaillé permet de reconstruire un document de découverte complet pour chaque API, et sert de source de vérité pour générer d’autres artefacts.
    
*   **API RESTful de consultation (lecture)** conforme à l’API Google Discovery v1. En particulier, on expose `GET /discovery/v1/apis` (liste des APIs enregistrées) et `GET /discovery/v1/apis/{api}/{version}/rest` (description détaillée d’une API donnée en JSON Discovery). Toute application conçue pour consommer l’API Google Discovery pourra consommer France-Nuage Discovery de la même manière. Le format de réponse (`discovery#directoryList`, `discovery#restDescription`, etc.) est respecté.
    
*   **API RESTful d’administration (écriture)** permettant d’enregistrer de nouvelles API, de mettre à jour leurs métadonnées ou d’en supprimer. Par cohérence, ces endpoints d’écriture manipulent des structures JSON similaires à celles retournées en lecture (par exemple, envoyer un objet JSON décrivant une méthode avec sa structure, équivalent à son representation dans le document de découverte). On définit des endpoints clairs (voir section dédiée) pour créer/mettre à jour : API, ressources, méthodes, schémas, etc. Ceci permet l’automatisation de l’enregistrement des services (par exemple, un pipeline CI/CD pourrait appeler ces endpoints pour publier la définition d’une API dès son développement terminé).
    
*   **Génération automatique de fichiers .proto (gRPC)** à partir des métadonnées stockées. Pour chaque API enregistrée, le service pourra produire un fichier `.proto` décrivant des services gRPC équivalents aux endpoints REST (par exemple, un service gRPC par ressource avec des RPC `Create`, `Get`, `List`, etc.) ainsi que les messages (schémas de données) correspondants. L’objectif est de fournir une interface gRPC _optionnelle mais disponible_ pour chaque API, afin d’exploiter les avantages de gRPC (typage strict, performances HTTP/2, streaming, etc.) en parallèle de REST.
    
*   **Génération automatique d’un provider Terraform** pour les APIs enregistrées. Grâce aux informations sur les ressources (types d’objet, champs, opérations CRUD), on peut générer le code source d’un provider Terraform France-Nuage. Chaque type de ressource API devient une ressource Terraform (avec ses champs mappés, ses opérations implémentées via les appels HTTP appropriés). Cela permet aux utilisateurs d’interagir avec les services France-Nuage via Infrastructure as Code sans effort manuel de développement de provider.
    

Ci-dessous, nous détaillons d’abord le **modèle relationnel SQL** retenu, puis les **API REST exposées** (lecture et écriture), et enfin la **transformation des données** vers les fichiers `.proto` et la configuration Terraform. Nous concluons par les **justifications techniques** de ces choix d’architecture.

Modèle de Données Relationnel (PostgreSQL)
------------------------------------------

### Organisation par schémas PostgreSQL

La base de données est PostgreSQL 15+. Deux schémas logiques principaux sont utilisés pour regrouper les tables :

*   Schéma **`discovery`** : contient les tables principales stockant les **définitions d’API** (API, resource, method, parameter, message, field, operation, etc.). On y regroupe les entités métiers décrivant les APIs.
    
*   Schéma **`ref`** (référentiel) : contient les tables d’**énumérations et listes de référence** utilisées dans le modèle (par exemple les types de données, les types d’opération, etc.). Cela évite d’utiliser des types ENUM SQL tout en gardant la liste des valeurs autorisées normalisée dans la base.
    

Ce découpage via les _schemas_ PostgreSQL permet de **structurer logiquement** la base. Par exemple, on pourra accorder des privilèges différents sur `ref` (valeurs quasi constantes) et `discovery` (données métier modifiables). Il sera également plus lisible de voir les tables regroupées par usage.

### Tables de référence (énumérations)

Plusieurs petites tables listent les valeurs possibles de certains champs structurants ; elles jouent le rôle d’**énumérations extensibles**. Les principales tables de référence incluent :

*   `ref.http_method` : liste des méthodes HTTP autorisées pour les endpoints. Elle contient par exemple les entrées `GET`, `POST`, `PUT`, `PATCH`, `DELETE` (chaque ligne avec un identifiant et le nom du verbe). Toutes les méthodes de l’API référenceront l’une de ces valeurs.
    
*   `ref.data_type` : liste des types de données possibles pour les champs et paramètres, alignée sur les types JSON Schema utilisés dans Discovery. Exemples d’entrées : `string`, `integer`, `number`, `boolean`, `object`, `array`. Cela sert pour indiquer le type de chaque paramètre ou champ de schéma. D’autres types plus spécifiques peuvent y figurer (ex : `int64`, `double`, qui peuvent être traités comme alias ou via le champ `format`).
    
*   `ref.param_location` : valeurs possibles pour la localisation d’un paramètre de méthode. Par exemple : `path` (paramètre d’URL path templating), `query` (paramètre de requête), `body` (paramètre correspond à le corps JSON de la requête). Les paramètres de type `body` ne contiendront pas de type simple mais référenceront un schéma de message.
    
*   `ref.operation_type` : liste des types d’opérations haut niveau pour les ressources, utilisées pour la génération Terraform. On y définit notamment `CREATE`, `READ`, `UPDATE`, `DELETE`, `LIST` correspondant aux actions CRUD classiques. Chaque ressource API pourra associer ses méthodes à ces catégories (par ex. la méthode X est l’opération `CREATE` pour la ressource Y).
    
*   `ref.api_label` (et possiblement `ref.api_feature`) : liste pré-définie des étiquettes ou statuts qu’une API peut avoir. Google Discovery utilise des labels comme `limited_availability` ou `deprecated`[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=match%20at%20L378%20%60labels,For%20example%2C%20REST). On peut reprendre ces valeurs dans `ref.api_label`. De même, `ref.api_feature` pourrait lister d’éventuelles features supportées (ex : `mediaUpload` dans Google, etc.) si on souhaite modéliser l’attribut `features[]` du document de découverte.
    

Chacune de ces tables a une clé primaire (souvent un code texte court) et éventuellement une description. Elles sont peu volumineuses et _statiques_ ou mises à jour rarement. **Pourquoi ne pas utiliser ENUM SQL directement ?** Parce qu’**ajouter une valeur dans un ENUM PostgreSQL nécessite une migration DDL** et peut impacter le stockage, alors qu’insérer une ligne dans une table de référence est trivial. Comme le note un expert, _« les types ENUM nécessitent une intervention DBA pour évoluer, alors que l’ajout d’une entrée dans une table de référence reste une opération de données standard »_[sitepoint.com](https://www.sitepoint.com/community/t/using-enum-vs-check-constraint-vs-lookup-tables/6704#:~:text=enums%20and%20check%20constraints%20require,like%20any%20other%20user%20data). Cette souplesse est cruciale pour un système amené à évoluer.

### Tables principales du schéma `discovery`

Nous décrivons ci-dessous les tables principales qui stockent les métadonnées des APIs. Le **diagramme relationnel** ci-après donne un aperçu des entités et relations :

【】_(Schéma simplifié des tables principales : API, Resource, Method, Parameter, Message, Field, Operation et leurs relations)_【】

_(Chaque rectangle représente une table, les flèches illustrent les clés étrangères. Par exemple, Resource → API indique qu’une ressource référence une API. Les tables de référence ne sont pas développées ici.)_

#### Table `discovery.api`

Cette table représente une **API** publiée (généralement correspondant à un service ou un produit, avec une version). Chaque enregistrement correspond typiquement à un couple _{nom d’API, version}_.

**Champs principaux** :

*   `id` (PK interne, serial),
    
*   `name` (nom court de l’API, ex: `"urlshortener"`),
    
*   `version` (ex: `"v1"`),
    
*   `title` (titre lisible, ex: `"Google URL Shortener API"`),
    
*   `description`,
    
*   `revision` (chaine de version interne de la définition, optionnelle),
    
*   `documentation_link` (URL vers la doc externe),
    
*   `protocol` (typiquement `"rest"` pour indiquer le type d’API décrite),
    
*   `root_url`, `service_path`, `base_path`, `base_url` (informations d’URL de base comme dans Google Discovery[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=%5D%2C%20,%5B)[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=,string)),
    
*   `batch_path` (chemin pour les appels batch si applicable),
    
*   `created_at`, `updated_at` (timestamps d’audit en `timestamptz`).
    

Des champs JSON supplémentaires comme `icons` (URL des icônes 16x16 et 32x32) et une liste de `labels` peuvent être stockés soit sous forme de JSONB ou via des tables associatives (`api_label` reliant API et `ref.api_label`). Ici on peut choisir une implémentation simple : par exemple, avoir `api.preferred` (booléen indiquant si c’est la version préférée) et stocker d’autres labels spécifiques via une table jointe.

**Contraintes** : `(name, version)` est unique (on ne peut enregistrer deux fois la même API/version). La clé primaire interne `id` est utilisée par les FKs des tables liées.

#### Table `discovery.resource`

Table des **ressources** REST d’une API, permettant de structurer hiérarchiquement les méthodes. Une ressource correspond généralement à un **groupe logique de endpoints** (souvent associé à un type d’objet manipulé). Par exemple, une API “Library” pourrait avoir une ressource “books” pour regrouper les méthodes liées aux livres. Dans les documents Discovery, les ressources sont représentées comme des objets englobant des méthodes et éventuellement des sous-ressources.

**Champs** :

*   `id` (PK),
    
*   `api_id` (FK vers `api.id` – la ressource appartient à quelle API),
    
*   `parent_resource_id` (FK auto-référent vers `resource.id` – permet des sous-ressources imbriquées ; null si ressource au premier niveau),
    
*   `name` (nom de la ressource, qui servira de clé dans le JSON et généralement de segment d’URL – ex: `"projects"`, `"instances"`),
    
*   `description`,
    
*   `deprecated` (booléen, si la ressource est marquée obsolète).
    

Chaque ressource _peut avoir des sous-ressources_. Par exemple, pour une API cloud on pourrait avoir la ressource “projects” contenant la sous-ressource “instances”. Ce modèle récursif est supporté via `parent_resource_id`. Le nom de la ressource est utilisé pour construire les chemins d’URL des méthodes, mais la **structure finale de l’URL** est définie au niveau de chaque méthode (voir plus loin). On notera que **la table Resource ne stocke pas directement de chemin** : le chemin complet d’un appel est reconstruit en combinant les segments des ressources parentes et le `path` défini dans Method.

#### Table `discovery.method`

Table des **méthodes** HTTP, c’est-à-dire des endpoints concrets. Chaque enregistrement représente un appel REST distinct (combinaison d’une URL relative et d’un verbe HTTP, avec éventuellement des paramètres). Dans le document de découverte JSON, les méthodes sont listées soit à la racine (pour celles qui ne sont pas dans une ressource) soit à l’intérieur d’une ressource.

**Champs** :

*   `id` (PK),
    
*   `api_id` (FK vers API – redondant si `resource_id` est présent, mais pratique pour méthodes sans ressource parente),
    
*   `resource_id` (FK vers Resource, null si la méthode est déclarée au niveau racine de l’API),
    
*   `name` (nom de la méthode tel qu’utilisé dans le JSON Discovery comme clé. Exemples : `"list"`, `"get"`, `"delete"`, ou un nom personnalisé comme `"send"`. Ce n’est pas forcément unique globalement, mais unique au sein de la ressource donnée),
    
*   `http_method` (FK vers `ref.http_method`, par ex. `GET`),
    
*   `path` (chemin URL relatif **par rapport au `basePath` de l’API**, incluant éventuellement des paramètres entre accolades. Exemples : `"projects/{projectId}/instances/{instanceId}"` pour une méthode avec deux paramètres de chemin, ou `"books"` pour un simple endpoint collection),
    
*   `description`,
    
*   `deprecated` (booléen).
    

De plus, les champs suivants caractérisent les **données d’entrée/sortie** de la méthode :

*   `request_schema_id` (FK vers `discovery.message` décrivant le schéma JSON du corps de requête, null si pas de corps JSON attendu ou si seuls des paramètres simples sont attendus),
    
*   `response_schema_id` (FK vers `discovery.message` pour le schéma de la réponse renvoyée par cette méthode, null si la réponse est vide ou un simple code 204).
    

La méthode peut aussi indiquer des détails propres à Google :

*   `scopes` (liste des identifiants OAuth2 scopes requis). Dans le modèle relationnel, on peut avoir une table d’association `method_scope` (méthode N – N scopes) reliant vers une table `api_scope` qui stocke les scopes disponibles pour l’API (avec leur description). Ceci permet de peupler la section `auth.oauth2.scopes` du document. Pour simplifier, on peut omettre le détail dans un premier temps ou stocker les scopes en tableau de texte.
    
*   Indicateurs booléens `supports_media_upload`, `supports_media_download`, et objet `media_upload` (avec MIME types acceptés, taille max, etc.) si l’API supporte l’upload de fichiers. Ces champs ne sont pas forcément utiles pour France-Nuage dans l’immédiat, mais le modèle peut les prévoir (colonnes booléennes et éventuellement un JSONB pour `media_upload` config).
    
*   `parameter_order` (ordre des paramètres dans l’appel, principalement pertinent pour générer certaines librairies clients). On peut déduire l’ordre des paramètres de chemin depuis le champ `path` et l’ordre des paramètres obligatoires, mais on peut aussi stocker explicitement un array de noms de paramètres dans l’ordre. Par simplification, on peut ne pas stocker `parameter_order` et considérer qu’il correspond aux paramètres de chemin dans l’ordre où ils apparaissent.
    

**Relations** : La méthode va référencer zéro ou un schéma de requête et de réponse (via FK sur `message.id`), et elle aura une liste de paramètres associée (relation 1-N vers la table Parameter décrite ci-après). Si `resource_id` est présent, la méthode sera incluse dans la ressource correspondante lors de la sortie JSON (sous l’objet `resources.{resourceName}.methods.{methodName}`) ; sinon, elle apparaîtra au niveau racine de l’objet JSON (`methods.{methodName}`).

**Remarques** : La combinaison `(api_id, http_method, path)` pourrait être contrainte unique pour éviter d’enregistrer deux méthodes identiques, bien que techniquement deux méthodes différentes pourraient partager la même path avec des verbes différents. Ici, `(api, http_method, path)` unique est raisonnable (on n’aurait pas deux méthodes GET avec le même chemin par ex). Le nom de méthode n’a besoin d’être unique qu’au sein d’une même ressource ou de l’API root, ce qui est naturellement assuré par la hiérarchie JSON.

#### Table `discovery.parameter`

Cette table décrit les **paramètres** d’une méthode API. Il s’agit principalement des paramètres de **requête (query)** ou de **chemin (path)**. (Les paramètres d’en-tête HTTP ne sont pas vraiment couverts par Google Discovery et ne sont pas prévus ici, hormis les aspects auth que nous modélisons à part.) Un paramètre correspond à la fois à la définition de ses contraintes (type, obligatoire ou non, etc.) et à son positionnement (query string ou segment d’URL).

**Champs** :

*   `id` (PK),
    
*   `method_id` (FK vers Method – le paramètre est spécifique à une méthode),
    
*   `api_id` (FK vers API – si on veut supporter des **paramètres globaux** communs à toutes les méthodes de l’API, comme `key` ou `prettyPrint` chez Google, on pourrait stocker method\_id = null et juste api\_id non null. Ces paramètres globaux seraient inclus dans chaque appel, typiquement en query. Google liste les “parameters” globaux séparément. Notre modèle le permet en rendant method\_id optionnel),
    
*   `name` (nom du paramètre tel qu’il apparaît dans la requête, ex: `"projectId"`, `"pageSize"`, `"filter"`),
    
*   `location` (FK vers `ref.param_location` – ex: `path` ou `query`),
    
*   `type` (FK vers `ref.data_type` – type de base du paramètre, ex: `string`, `integer`),
    
*   `format` (texte optionnel précisant le format du type si besoin, ex: `"int64"`, `"date-time"`),
    
*   `description`,
    
*   `required` (booléen – si le param est obligatoire pour appeler l’API),
    
*   `repeated` (booléen – si ce param peut être fourni plusieurs fois, ex: paramètre multi-valeurs array),
    
*   `deprecated` (booléen – si le param est obsolète),
    
*   `default_value` (valeur par défaut sous forme texte, si applicable),
    
*   `pattern` (regex que la valeur doit respecter, si applicable),
    
*   `minimum` et `maximum` (bornes numériques, stockées en texte ou nombre selon type).
    

En outre, pour les paramètres plus complexes :

*   `$ref_schema_id` (FK vers `discovery.message` si le paramètre est de type objet défini par un schéma réutilisable). Par exemple, un paramètre de requête qui est un objet JSON complet (rare, mais Google peut avoir des structures plus complexes en query). Dans la plupart des cas, les paramètres en query/path sont des types primitifs. On peut donc laisser ce champ null dans la majorité des cas. S’il est renseigné, il indique que ce param suit la structure d’un schéma défini ailleurs (on attend alors un JSON conforme).
    
*   `items_type` / `items_ref_schema_id` (si le paramètre est de type array ou `repeated=true`, on peut préciser le type des éléments. Google Discovery utilise généralement `repeated=true` couplé à `type` pour signifier un tableau, mais on peut aussi gérer l’objet `items`. Dans notre modèle, on peut interpréter `repeated=true` + `type=X` comme “liste de X”. Si X = `object` ou un type complexe, alors `$ref_schema_id` indique le schéma des éléments. Alternativement, on pourrait avoir des champs dédiés pour le type d’items. Par simplification : `repeated` vrai implique que `type` est le type élémentaire ou null et `$ref_schema_id` le schéma des éléments le cas échéant.)
    

La **clé étrangère** `(method_id)` signifie que les paramètres sont spécifiques à une méthode (excepté les potentiels globaux comme discuté). Pour un paramètre de chemin, `required` sera généralement true (puisqu’il doit être présent dans l’URL), et son `name` correspondra à ce qui est dans `{ }` dans le champ `path` de Method. Il faudra cohérence entre `method.path` et la liste des paramètres `location=path` associés. On peut imaginer une contrainte ou au moins une validation applicative pour vérifier que chaque `{param}` dans le path a bien une entrée correspondante dans Parameter.

Les **paramètres globaux** (api\_id non null, method\_id null) seraient inclus dans la section `"parameters": {...}` au niveau racine du document JSON (Google y met par ex `fields`, `key`, etc.). Ils peuvent avoir une liste d’annotations `required` pour indiquer quelles méthodes les exigent[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=%60parameters.%28key%29.enumDescriptions,value%20in%20the%20enum%20array), mais pour simplifier on peut supposer qu’ils sont facultatifs sauf mention contraire.

Enfin, certains paramètres peuvent avoir des **valeurs énumérées**. Si un paramètre a un ensemble fini de valeurs possibles (ex: param `sortOrder` qui n’accepte que `"asc"` ou `"desc"`), le document Discovery fournit un tableau `"enum": [...]` et `"enumDescriptions": [...]`. Dans notre modèle, on peut gérer cela via une table associée `parameter_enum` : chaque ligne contiendrait `parameter_id` + `value` + `description` + `deprecated` (bool éventuel). Ainsi, si `parameter_enum` contient des entrées pour un paramètre donné, on pourra générer les champs enum appropriés dans le JSON.

#### Table `discovery.message` (schémas de données)

Cette table stocke les **schémas de données JSON** utilisés par l’API – équivalent des **messages** ou objets échangés. Dans le document Discovery, on trouve ces schémas dans la section `"schemas": { ... }`. Chaque schéma a un identifiant unique dans le contexte de l’API, et décrit soit un objet structuré, soit un type primitif (parfois paramétré), soit un tableau, etc.

**Champs** :

*   `id` (PK),
    
*   `api_id` (FK vers API – le schéma est défini dans le cadre d’une API),
    
*   `name` (identifiant du schéma, unique par API, ex: `"Book"`, `"ListBooksResponse"`, `"Empty"`),
    
*   `type` (FK vers `ref.data_type` – souvent ce sera `object` pour les schémas complexes, mais il peut y avoir des schémas simples de type `string`, `integer` etc. Un schéma type primitif avec enum peut représenter un type énuméré par exemple),
    
*   `description`,
    
*   `required` (booléen – ce champ peut être utilisé pour indiquer si ce schéma, lorsqu’il est utilisé comme propriété quelque part, est requis. Google inclut `required` dans la définition du schéma mais cela semble redondant – on pourrait l’utiliser pour noter qu’un schéma simple doit obligatoirement avoir une valeur… Ce champ est rarement utilisé et pourrait être omis),
    
*   `deprecated` (booléen – si ce schéma est obsolète),
    
*   `format` / `pattern` / `minimum` / `maximum` / `default_value` – mêmes sémantiques que pour Parameter/Field, utiles surtout si le schéma est un type primitif avec des contraintes, par exemple un schéma `"ImageFormat"` de type string avec un pattern ou une enum).
    
*   `repeatable` (booléen – comme pour Parameter, on pourrait marquer un schéma comme étant un tableau d’un autre type. Toutefois, Google représente les tableaux différemment : via un type `"array"` avec champ `items`. Nous pouvons modéliser les tableaux en créant un schéma de type `array` et reliant à `items_schema_id`, voir ci-dessous).
    
*   `items_schema_id` (FK vers un autre Message si ce schéma est un array d’objets – applicable si `type` = array ou si on choisit d’indiquer repeated autrement),
    
*   `items_type` (FK vers ref.data\_type si schéma array de types primitifs).
    

En pratique, deux approches coexistent pour représenter un tableau :

1.  soit on crée un schéma avec `type = array` et on précise la nature des items via `items_type` ou `items_schema_id`,
    
2.  soit on utilise `type` = X et `repeated=true` dans la propriété parente. Google Discovery utilise plutôt la notion de repeated sur les propriétés/paramètres, donc côté _schéma top-level_ on pourrait ne pas créer de schéma “array” séparé, sauf si on veut l’exposer dans “schemas”. Par simplicité, on peut adopter la règle : _les tableaux ne sont jamais des schémas nommés indépendants_ – c’est toujours le champ ou paramètre qui sera marqué repeated, sauf peut-être pour les réponses de liste (ex: un schéma `ListBooksResponse` contiendra un champ `books` qui est repeated Book). Donc on peut se passer de stocker `type=array` au niveau message, et plutôt gérer repeated au niveau Field/Parameter. **Nous supposerons ainsi que `message.type` est généralement `object` ou un primitif.** On définira les tableaux via `Field.repeated` ou `Parameter.repeated`.
    

La table Message va contenir toutes les structures utiles, par exemple :

*   Schémas de ressources principales (ex: `Book` avec type=object et des fields comme title, author… définis dans Field),
    
*   Schémas de requête et réponse des méthodes (ex: `ListBooksResponse` avec fields `books` (array de Book) et `nextPageToken`).
    
*   Schémas utilitaires (ex: un schéma vide `Empty` pour méthodes delete sans retour, ou des schémas de types énumérés).
    

Les relations : un Message de type object aura une liste de champs associés dans la table Field. Un Message peut référencer d’autres Message (via fields $ref ou item) ou être référencé par Parameter, Method (request/response) ou Field (sous-objet).

#### Table `discovery.field`

Table des **champs** internes d’un schéma (les propriétés si le schéma est un objet JSON). Chaque entrée correspond à une propriété nommée de l’objet représenté par un Message de type `object`.

**Champs** :

*   `id` (PK),
    
*   `message_id` (FK vers Message – le schéma dont c’est un champ, ce message doit avoir type=object),
    
*   `name` (nom de la propriété JSON, ex: `"id"`, `"title"`, `"items"` etc.),
    
*   `type` (FK vers `ref.data_type` – type de base de la propriété, ex: string, boolean, object, integer…),
    
*   `schema_id` (FK vers Message – si le champ est un objet complexe ou une référence vers un autre schéma. Correspond à `$ref` dans la spec. Si ce champ `schema_id` est non null, il indique que la propriété est de type complexe défini ailleurs, et le champ `type` peut alors être object ou null),
    
*   `description`,
    
*   `required` (booléen – indique si cette propriété est requise dans l’objet. Dans JSON Schema on met la liste des required dans le parent, mais ici on l’attache au champ pour simplicité),
    
*   `deprecated` (booléen – si la propriété est obsolète),
    
*   `repeated` (booléen – si cette propriété est un tableau de valeurs du type donné. Si true, alors soit `type` indique le type élémentaire, soit `schema_id` indique le schéma des éléments si c’est un tableau d’objets complexes),
    
*   `format`, `pattern`, `minimum`, `maximum`, `default_value` – similaires aux attributs de Parameter pour contraindre la valeur si pertinent (principalement pour des types primitifs numériques ou string avec format).
    

Comme pour Parameter, on peut aussi avoir des valeurs énumérées pour un champ (si un champ string n’accepte que certaines valeurs). Dans ce cas, on utiliserait une table `field_enum` avec `field_id`, `value`, `description`, `deprecated`. Cela permettra de générer les tableaux `"enum"` et `"enumDescriptions"` dans le JSON du schéma[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=match%20at%20L410%20%60parameters.%28key%29.enum,value%20in%20the%20enum%20array).

**Exemple** : Supposons un schéma `Book` (dans Message) avec champs : `id` (string), `title` (string), `author` (string), `status` (string avec enum possible {"AVAILABLE","BORROWED"}). On aurait dans Field des entrées pour chacun. Le champ `status` aurait `type = string` et on créerait deux lignes dans `field_enum` associées avec values "AVAILABLE"/"BORROWED". Le champ `id` pourrait être marqué `required=true` si toujours fourni, etc.

Les **contraintes** : `(message_id, name)` est unique (pas deux propriétés du même objet avec le même nom). L’ordre des champs n’est pas explicitement stocké – on présume que l’ordre n’a pas d’importance en JSON (et dans Discovery les propriétés ne sont pas ordonnées). Toutefois, pour générer du code (proto par ex), on donnera un numéro aux champs, peut-être dans l’ordre d’insertion ou alphabétique.

#### Table `discovery.operation`

Cette table relie les méthodes à des **catégories d’opérations** CRUD pour les besoins de Terraform. Elle sert à indiquer quelle méthode correspond à la création d’une ressource, laquelle à sa suppression, etc., de manière explicite.

**Champs** :

*   `id` (PK),
    
*   `resource_id` (FK vers Resource – la ressource concernée par l’opération),
    
*   `operation_type` (FK vers `ref.operation_type` – ex: CREATE, READ, UPDATE, DELETE, LIST),
    
*   `method_id` (FK vers Method – la méthode qui implémente cette opération pour cette ressource).
    

Par exemple, pour la ressource `books` de l’API Library, on aurait potentiellement cinq lignes dans Operation :

*   (resource=books, type=CREATE, method=POST /books),
    
*   (resource=books, type=READ, method=GET /books/{id}),
    
*   (resource=books, type=UPDATE, method=PUT /books/{id}),
    
*   (resource=books, type=DELETE, method=DELETE /books/{id}),
    
*   (resource=books, type=LIST, method=GET /books).
    

Si l’API ne propose pas certaines opérations (par ex. pas de mise à jour), on n’aura pas d’entrée correspondante. Cette table permet au générateur Terraform de savoir quelles URLs appeler pour chaque action sur la ressource. **Remarque** : dans certains cas, plusieurs methods pourraient correspondre (ex: il peut exister deux variantes de mise à jour, PUT et PATCH). On ne gère qu’une par type, préférentiellement la plus RESTful (ex: on choisirait PATCH si les deux existent). Ces décisions seront prises lors de l’enregistrement (un administrateur renseignera l’opération principale).

### Schéma SQL complet

Ci-dessous, nous présentons un **extrait de schéma SQL** illustrant la création des principales tables et contraintes décrites. (Les colonnes de méta-informations comme `created_at`/`updated_at` en timestamptz sont incluses sur les tables majeures).

```sql 
-- Schéma de référence
CREATE SCHEMA ref;
CREATE TABLE ref.http_method (
    code VARCHAR(10) PRIMARY KEY,
    description TEXT
);
INSERT INTO ref.http_method(code) VALUES ('GET'),('POST'),('PUT'),('PATCH'),('DELETE');

CREATE TABLE ref.data_type (
    code VARCHAR(20) PRIMARY KEY,
    description TEXT
);
INSERT INTO ref.data_type(code) VALUES 
    ('string'),('integer'),('number'),('boolean'),('object'),('array');

CREATE TABLE ref.param_location (
    code VARCHAR(10) PRIMARY KEY,
    description TEXT
);
INSERT INTO ref.param_location(code) VALUES ('query'),('path'),('body');

CREATE TABLE ref.operation_type (
    code VARCHAR(10) PRIMARY KEY,
    description TEXT
);
INSERT INTO ref.operation_type(code) VALUES ('CREATE'),('READ'),('UPDATE'),('DELETE'),('LIST');

CREATE TABLE ref.api_label (
    code VARCHAR(30) PRIMARY KEY,
    description TEXT
);
INSERT INTO ref.api_label(code) VALUES ('limited_availability'),('deprecated'),('stable');

-- Schéma principal
CREATE SCHEMA discovery;
CREATE TABLE discovery.api (
    id            SERIAL PRIMARY KEY,
    name          VARCHAR(50) NOT NULL,
    version       VARCHAR(20) NOT NULL,
    title         TEXT,
    description   TEXT,
    revision      VARCHAR(20),
    documentation_link TEXT,
    protocol      VARCHAR(10) NOT NULL DEFAULT 'rest',
    root_url      TEXT,
    base_path     TEXT,
    service_path  TEXT,
    base_url      TEXT,
    batch_path    TEXT,
    preferred     BOOLEAN DEFAULT FALSE,
    created_at    TIMESTAMPTZ DEFAULT NOW(),
    updated_at    TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(name, version)
);

CREATE TABLE discovery.resource (
    id                SERIAL PRIMARY KEY,
    api_id            INT NOT NULL REFERENCES discovery.api(id) ON DELETE CASCADE,
    parent_resource_id INT REFERENCES discovery.resource(id) ON DELETE CASCADE,
    name              VARCHAR(50) NOT NULL,
    description       TEXT,
    deprecated        BOOLEAN DEFAULT FALSE,
    UNIQUE(api_id, parent_resource_id, name)
);

CREATE TABLE discovery.message (
    id           SERIAL PRIMARY KEY,
    api_id       INT NOT NULL REFERENCES discovery.api(id) ON DELETE CASCADE,
    name         VARCHAR(100) NOT NULL,
    type_code    VARCHAR(20) NOT NULL REFERENCES ref.data_type(code),
    description  TEXT,
    deprecated   BOOLEAN DEFAULT FALSE,
    format       TEXT,
    pattern      TEXT,
    minimum      TEXT,
    maximum      TEXT,
    default_value TEXT,
    -- If this message is an array type, reference item type/schema (optional)
    items_type_code    VARCHAR(20) REFERENCES ref.data_type(code),
    items_schema_id    INT REFERENCES discovery.message(id),
    UNIQUE(api_id, name)
);

CREATE TABLE discovery.field (
    id          SERIAL PRIMARY KEY,
    message_id  INT NOT NULL REFERENCES discovery.message(id) ON DELETE CASCADE,
    name        VARCHAR(50) NOT NULL,
    type_code   VARCHAR(20) REFERENCES ref.data_type(code),
    schema_id   INT REFERENCES discovery.message(id),
    description TEXT,
    required    BOOLEAN DEFAULT FALSE,
    deprecated  BOOLEAN DEFAULT FALSE,
    repeated    BOOLEAN DEFAULT FALSE,
    format      TEXT,
    pattern     TEXT,
    minimum     TEXT,
    maximum     TEXT,
    default_value TEXT,
    UNIQUE(message_id, name)
);

CREATE TABLE discovery.method (
    id           SERIAL PRIMARY KEY,
    api_id       INT NOT NULL REFERENCES discovery.api(id) ON DELETE CASCADE,
    resource_id  INT REFERENCES discovery.resource(id) ON DELETE CASCADE,
    name         VARCHAR(50) NOT NULL,
    http_method_code VARCHAR(10) NOT NULL REFERENCES ref.http_method(code),
    path         TEXT NOT NULL,
    description  TEXT,
    deprecated   BOOLEAN DEFAULT FALSE,
    request_schema_id  INT REFERENCES discovery.message(id),
    response_schema_id INT REFERENCES discovery.message(id),
    -- ensure method path is unique per HTTP verb within API:
    UNIQUE(api_id, http_method_code, path)
);

CREATE TABLE discovery.parameter (
    id           SERIAL PRIMARY KEY,
    method_id    INT REFERENCES discovery.method(id) ON DELETE CASCADE,
    api_id       INT REFERENCES discovery.api(id) ON DELETE CASCADE,
    name         VARCHAR(50) NOT NULL,
    location_code VARCHAR(10) NOT NULL REFERENCES ref.param_location(code),
    type_code    VARCHAR(20) REFERENCES ref.data_type(code),
    schema_id    INT REFERENCES discovery.message(id),
    description  TEXT,
    required     BOOLEAN DEFAULT FALSE,
    repeated     BOOLEAN DEFAULT FALSE,
    deprecated   BOOLEAN DEFAULT FALSE,
    format       TEXT,
    pattern      TEXT,
    minimum      TEXT,
    maximum      TEXT,
    default_value TEXT,
    UNIQUE(method_id, name)
    -- Note: for global params, method_id is null and api_id + name could be unique.
);

CREATE TABLE discovery.parameter_enum (
    parameter_id INT NOT NULL REFERENCES discovery.parameter(id) ON DELETE CASCADE,
    value        TEXT NOT NULL,
    description  TEXT,
    deprecated   BOOLEAN DEFAULT FALSE,
    PRIMARY KEY(parameter_id, value)
);

CREATE TABLE discovery.field_enum (
    field_id    INT NOT NULL REFERENCES discovery.field(id) ON DELETE CASCADE,
    value       TEXT NOT NULL,
    description TEXT,
    deprecated  BOOLEAN DEFAULT FALSE,
    PRIMARY KEY(field_id, value)
);

CREATE TABLE discovery.operation (
    id            SERIAL PRIMARY KEY,
    resource_id   INT NOT NULL REFERENCES discovery.resource(id) ON DELETE CASCADE,
    operation_code VARCHAR(10) NOT NULL REFERENCES ref.operation_type(code),
    method_id     INT NOT NULL REFERENCES discovery.method(id),
    UNIQUE(resource_id, operation_code)
);

```

_(Code SQL indicatif – certaines contraintes (CHECK, triggers de mise à jour de `updated_at`, etc.) sont omises pour concision.)_

Ce schéma relationnel assure la **cohérence référentielle** : par exemple, si on supprime une API, toutes ses ressources, méthodes, paramètres associées seront cascades supprimées, évitant les données orphelines. Les tables de référence (`ref.*`) garantissent l’intégrité des valeurs pour les colonnes clés (types, verbes, etc.). L’utilisation de `TIMESTAMPTZ` pour les colonnes de dates (`created_at`, `updated_at`) garantit qu’on stocke les timestamps avec fuseau horaire (typiquement UTC). C’est une bonne pratique car _« sans aucun doute TIMESTAMPTZ est indispensable pour le stockage du temps dans Postgres : en sauvegardant date, heure _et_ fuseau, on n’a plus à se soucier des conversions ou du contexte serveur/client »_[crunchydata.com](https://www.crunchydata.com/blog/working-with-time-in-postgres#:~:text=,any%20of%20those%20crazy%20calculations).

Le modèle peut sembler complexe, mais il reflète fidèlement les composantes d’un document de découverte. Il permet de **composer facilement les réponses JSON** via des jointures appropriées ou une couche applicative qui navigue ces relations.

API REST de Consultation (Lecture) – Compatible Google Discovery
----------------------------------------------------------------

Pour la partie lecture, le service expose les mêmes endpoints que l’API Google Discovery v1[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=The%20Discovery%20document%20focuses%20on,based%20discovery%20documents), à savoir principalement :

*   **GET `/discovery/v1/apis`** – Retourne la **liste des APIs** disponibles (toutes versions confondues) au format annuaire. Cette route correspond à la méthode `discovery.apis.list` de Google[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=The%20Discovery%20document%20focuses%20on,based%20discovery%20documents). La réponse est un JSON de type `discovery#directoryList` contenant un tableau `items` où chaque entrée décrit une API.
    

Chaque **item** comprend typiquement : `id` (concaténation nom:version), `name`, `version`, `title`, `description`, `discoveryRestUrl` (URL pour obtenir le document détaillé), éventuellement `icons` (URLs d’icônes), et un champ `preferred` pour marquer la version préférée si plusieurs versions existent[dev.to](https://dev.to/schttrj/accessing-the-google-api-discovery-api-and-its-associated-discovery-documents-48aj#:~:text=%22kind%22%3A%20%22discovery,discoveryRestUrl)[dev.to](https://dev.to/schttrj/accessing-the-google-api-discovery-api-and-its-associated-discovery-documents-48aj#:~:text=,true).

_Exemple de réponse_ (simplifiée) pour une API fictive France-Nuage :

```json
{
  "kind": "discovery#directoryList",
  "discoveryVersion": "v1",
  "items": [
    {
      "kind": "discovery#directoryItem",
      "id": "library:v1",
      "name": "library",
      "version": "v1",
      "title": "Service de gestion de bibliothèque",
      "description": "API pour gérer les livres et auteurs dans une bibliothèque.",
      "discoveryRestUrl": "https://discovery.france-nuage.fr/discovery/v1/apis/library/v1/rest",
      "documentationLink": "https://docs.france-nuage.fr/library/v1",
      "preferred": true
    },
    {
      "kind": "discovery#directoryItem",
      "id": "storage:v2",
      "name": "storage",
      "version": "v2",
      "title": "Service de stockage d\u2019objets",
      "description": "API pour stocker et récupérer des objets.",
      "discoveryRestUrl": "https://discovery.france-nuage.fr/discovery/v1/apis/storage/v2/rest",
      "preferred": false
    }
  ]
}
```

Ici on voit deux APIs (library v1 et storage v2) listées. Notre service construira cette réponse en listant toutes les entrées de la table `api`, en injectant le lien Discovery approprié. Le champ `preferred` pourra être déterminé par un attribut dans la table `api` (par exemple on marque la version la plus récente ou stable).

Des **paramètres de filtrage** pourront être supportés comme chez Google : `?name=library` pour ne retourner que ce nom d’API, `?preferred=true` pour n’avoir que les versions préférées. Il suffit de filtrer la requête SQL derrière.

*   **GET `/discovery/v1/apis/{apiName}/{version}/rest`** – Retourne le **document de découverte complet** pour l’API demandée (au format JSON, Discovery document). C’est la méthode principale pour obtenir la description de toutes les ressources, méthodes et schémas d’une API spécifique. Le résultat a la structure type `discovery#restDescription`[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=%7B%20%22kind%22%3A%20%22discovery,string).
    

La réponse inclut beaucoup d’informations. Notre service va reconstruire ce JSON à partir des tables relationnelles. Pour une API donnée, il faut assembler :

*   **Propriétés de haut niveau de l’API** : `name`, `version`, `title`, `description`, `documentationLink`, `revision`, `protocol` (toujours "rest"), `baseUrl`, `basePath`, `rootUrl`, `servicePath`, `batchPath`, `parameters` (les éventuels paramètres globaux), `auth` (scopes OAuth2), `features` (le cas échéant), `labels` (ex: "limited\_availability").
    
*   **La liste des schémas** (`schemas`) définis : c’est un objet dont chaque clé est le nom d’un schéma et la valeur est la définition JSON Schema correspondante. On y met tous les enregistrements de `discovery.message` pour cette API. Par exemple, si l’API Library a des schémas `Book`, `Author`, `ListBooksResponse`, chacun sera une entrée avec ses champs comme `id, type, properties, required, etc.`. On générera la section `schemas` à partir de la table Message et de ses Fields. Pour chaque schéma de type `object`, on aura une sous-section `properties` contenant chaque champ (et ses éventuels sous-propriétés récursivement si on a modélisé inline, mais dans notre cas on fait plutôt les $ref). Les champs enum seront traduits en `enum` et `enumDescriptions` dans la JSON[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=match%20at%20L410%20%60parameters.%28key%29.enum,value%20in%20the%20enum%20array).
    
*   **La liste des méthodes** : structurée d’abord par ressources. Notre service doit restituer un objet JSON où les ressources imbriquées apparaissent. Concrètement :
    
    *   S’il y a des méthodes attachées à l’API sans ressource (resource\_id null), elles seront listées dans `"methods": { "<name>": { ... } }` au niveau racine.
        
    *   Pour chaque ressource de niveau 1, on crée une entrée dans `"resources": { "<resourceName>": { ... } }`. À l’intérieur, on aura éventuellement `"methods": { ... }` pour ses méthodes directes, et `"resources": { ... }` pour ses sous-ressources.
        
    *   Ce nesting se reconstruit en parcourant la table Resource de manière hiérarchique, et en insérant les méthodes de chaque ressource.
        
    *   Chaque méthode est rendue avec ses attributs : `id` (on peut concaténer api.name + "." + (resource names) + "." + method.name pour reproduire un identifiant unique style "library.books.list"), `path`, `httpMethod`, `description`, `parameters` (les paramètres spécifiques de cette méthode, combinant path et query params), `parameterOrder` (on peut fournir le tableau des paramètres de chemin par ex), `request` (s’il y a un schéma de requête, on met `"$ref": "SchemaName"`), `response` (idem), `scopes` (si défini), etc.[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=,%7B%20%28key%29%3A)[developers.google.com](https://developers.google.com/discovery/v1/reference/apis#resource#:~:text=%7D%2C%20,string).
        

Cela donne un JSON potentiellement volumineux. **Exemple simplifié** pour l’API `library:v1` avec juste une ressource `books` :

```json 
{
  "kind": "discovery#restDescription",
  "discoveryVersion": "v1",
  "id": "library:v1",
  "name": "library",
  "version": "v1",
  "revision": "0",
  "title": "Service de gestion de bibliothèque",
  "description": "API pour gérer les livres et auteurs dans une bibliothèque.",
  "protocol": "rest",
  "basePath": "/library/v1/",
  "baseUrl": "https://api.france-nuage.fr/library/v1/",
  "rootUrl": "https://api.france-nuage.fr/",
  "servicePath": "library/v1/",
  "labels": ["limited_availability"],
  "schemas": {
    "Book": {
      "id": "Book",
      "type": "object",
      "description": "Un livre de la biblioth\u00e8que",
      "properties": {
        "id":   { "type": "string", "description": "Identifiant du livre" },
        "title":{ "type": "string", "description": "Titre du livre", "required": true },
        "author":{ "type": "string", "description": "Auteur du livre" },
        "status":{ 
          "type": "string", 
          "description": "Disponibilit\u00e9", 
          "enum": ["AVAILABLE", "BORROWED"], 
          "enumDescriptions": ["Disponible", "Emprunt\u00e9"] 
        }
      }
    },
    "ListBooksResponse": {
      "id": "ListBooksResponse",
      "type": "object",
      "properties": {
        "books": {
          "type": "array",
          "items": { "$ref": "Book" }
        },
        "nextPageToken": { "type": "string" }
      }
    }
  },
  "resources": {
    "books": {
      "methods": {
        "list": {
          "id": "library.books.list",
          "path": "books",
          "httpMethod": "GET",
          "description": "Liste tous les livres",
          "parameters": {
            "pageSize": {
              "type": "integer", "format": "int32", "description": "Nombre max de livres \u00e0 retourner", "minimum": "1", "maximum": "100"
            },
            "pageToken": {
              "type": "string", "description": "Jeton de continuation pour paginer"
            }
          },
          "parameterOrder": ["pageSize", "pageToken"],
          "response": { "$ref": "ListBooksResponse" }
        },
        "get": {
          "id": "library.books.get",
          "path": "books/{bookId}",
          "httpMethod": "GET",
          "description": "R\u00e9cup\u00e8re un livre via son ID",
          "parameters": {
            "bookId": { "type": "string", "description": "Identifiant du livre", "required": true, "location": "path" }
          },
          "parameterOrder": ["bookId"],
          "response": { "$ref": "Book" }
        },
        "create": {
          "id": "library.books.create",
          "path": "books",
          "httpMethod": "POST",
          "description": "Cr\u00e9e un nouveau livre",
          "request": { "$ref": "Book" },
          "response": { "$ref": "Book" }
        },
        "delete": {
          "id": "library.books.delete",
          "path": "books/{bookId}",
          "httpMethod": "DELETE",
          "description": "Supprime un livre",
          "parameters": {
            "bookId": { "type": "string", "description": "Identifiant du livre", "required": true, "location": "path" }
          },
          "parameterOrder": ["bookId"]
        }
      }
    }
  }
}
```

Dans cet exemple, on voit comment tout est structuré : l’objet `books` dans `resources` contient à son tour un objet `methods` avec quatre méthodes CRUD (`list`, `get`, `create`, `delete`). Chaque méthode reprend l’info du modèle relationnel : `path`, `httpMethod` correspondent aux colonnes, les paramètres sont listés (ici `pageSize`, `pageToken` pour list – tous deux location query par défaut; et `bookId` pour get/delete avec location path). Les schémas `Book` et `ListBooksResponse` définis plus haut sont référencés via `$ref`.

Notre service génère exactement ce format. À noter qu’il faut inclure aussi dans la réponse l’objet `"auth"` si l’API requiert des scopes OAuth2. Par exemple :

```json
"auth": {
  "oauth2": {
    "scopes": {
      "https://auth.example.com/library.read": {
         "description": "Voir les livres"
      },
      "https://auth.example.com/library.write": {
         "description": "Modifier les livres"
      }
    }
  }
}
```

Ces infos proviendraient d’une table `api_scope` ou similaire et des associations method<->scope. Si France-Nuage utilise un système OAuth2 commun, on peut documenter les scopes ici pour info aux intégrateurs.

**Compatibilité Google Discovery** : En respectant scrupuleusement les clés (`kind`, `id`, `name`, etc.) et la structure, on s’assure que n’importe quel outil client qui consomme habituellement `.../discovery/v1/apis/.../rest` pourra consommer les notres. Par exemple, Google fournit des bibliothèques qui lisent ces documents pour générer des clients – on pourrait théoriquement utiliser ces libs en pointant vers France-Nuage. Cela ouvre la voie à la réutilisation d’outils existants.

API REST d’Écriture (Administration)
------------------------------------

Pour créer et maintenir les enregistrements d’API dans ce service, on propose une série d’endpoints REST **internes** (non nécessairement exposés au public, ou bien protégés). Ces endpoints permettent de _créer, mettre à jour, supprimer_ les différentes entités (API, ressource, méthode, schéma…) qui composent une définition. L’approche suit un design REST classique, avec des URLs reflétant la hiérarchie API -> Resource -> Method, etc., et des corps JSON reprenant la structure attendue.

Les principales routes d’écriture sont :

*   **POST `/discovery/v1/apis`** – _Créer une nouvelle API_. Le corps de la requête contiendra les métadonnées de l’API. Au minimum, `name` et `version` doivent être fournis, ainsi que les champs descriptifs utiles (`title`, `description`, `documentationLink`). On peut aussi inclure des champs comme `revision` ou `labels`.
    
    **Exemple** :
    
``` json
{
  "name": "library",
  "version": "v1",
  "title": "Service de gestion de bibliothèque",
  "description": "API pour gérer les livres et auteurs",
  "documentationLink": "https://docs.france-nuage.fr/library/v1",
  "labels": ["limited_availability"]
}
```
    
    Réponse possible : un objet JSON représentant l’API créée (incluant son `id` interne éventuellement). Le service va insérer dans la table `api` et renvoyer la ressource créée (format proche d’un directoryItem).
    
*   **PUT/PATCH `/discovery/v1/apis/{apiName}/{version}`** – _Mettre à jour une API existante_. Permet de modifier le titre, description, etc., ou les labels. En pratique, comme l’identifiant d’API est `{apiName, version}`, on l’utilise dans l’URL. Le corps JSON peut être au même format que ci-dessus avec les champs à modifier. Un appel GET sur la même URL (hors `/rest`) pourrait retourner l’objet API (on pourrait implémenter GET /apis/{name}/{ver} renvoyant un mini-objet, mais ce n’est pas strictement nécessaire car la liste donne déjà toutes les infos).
    
*   **DELETE `/discovery/v1/apis/{apiName}/{version}`** – _Supprimer une API_ et toutes ses métadonnées associées. Cette opération supprimera en cascade les ressources, méthodes, etc., correspondants dans la base.
    

Pour les **ressources** et **sous-ressources** :

*   **POST `/discovery/v1/apis/{apiName}/{version}/resources`** – _Ajouter une ressource de top-level_ à l’API. Corps JSON attendu : `name` (nom de la ressource) et optionnellement `description`.
    
    `{ "name": "books", "description": "Collection des livres" }`
    
    Le service crée l’entrée Resource correspondante (avec parent\_resource\_id = null). La réponse peut inclure l’ID interne ou le repréciser.
    
*   **POST `/discovery/v1/apis/{apiName}/{version}/resources/{resourceName}/resources`** – _Ajouter une sous-ressource_ à une ressource existante. Ex: ajouter “instances” sous “projects”. Corps similaire (nom, description). L’URL identifie la ressource par son nom parent. (En interne, le service retrouvera `parent_resource_id` via api + resourceName). On assume que le couple (api, resourceName) est unique pour top-level, et (parent, resourceName) unique pour sub-level, ce qui est imposé par le modèle.
    
*   **PUT/PATCH `/discovery/v1/apis/{apiName}/{version}/resources/{resourceName}`** – _Mettre à jour une ressource_ (par ex changer sa description ou la marquer deprecated).
    
*   **DELETE `/discovery/v1/apis/{apiName}/{version}/resources/{resourceName}`** – _Supprimer une ressource_ (et récursivement ses sous-ressources et méthodes).
    

Pour les **méthodes** :

*   **POST `/discovery/v1/apis/{apiName}/{version}/methods`** – _Ajouter une méthode au niveau de l’API_ (hors ressource). Le corps JSON doit fournir les attributs de la méthode, à savoir :
    
    *   `name` (nom interne de la méthode, ex: "getIamPolicy"),
        
    *   `path` (chemin relatif),
        
    *   `httpMethod` (verbe HTTP),
        
    *   `description`,
        
    *   éventuellement `request` (objet avec `"$ref"` pointant vers un schéma existant ou définition inline d’un schéma),
        
    *   `response` (de même),
        
    *   `parameters` (objet listant les paramètres query/path avec leur définition).
        
    
    **Exemple** (méthode sans ressource) :
    
```json
{
  "name": "getStatus",
  "path": "status",
  "httpMethod": "GET",
  "description": "Get global status of the service",
  "response": {
    "id": "StatusResponse",
    "type": "object",
    "properties": {
      "status": {"type": "string"}, 
      "timestamp": {"type": "string", "format": "date-time"}
    }
  }
}
```
    
    Ici, on montre qu’il est possible d’inclure la définition d’un schéma (`StatusResponse`) inline dans la requête de création de méthode. Le service doit alors créer d’abord le schéma (insertion dans `message` et `field`), puis la méthode qui s’y réfère. Alternativement, on aurait pu exiger que le schéma soit créé via l’endpoint de schéma séparé avant, et ne fournir ici que `"$ref": "StatusResponse"`. Pour plus de convivialité, on peut accepter la définition inline complète et la traiter en conséquence.
    
*   **POST `/discovery/v1/apis/{apiName}/{version}/resources/{resourceName}/methods`** – _Ajouter une méthode dans une ressource donnée_. Corps JSON similaire, mais concernant un endpoint sous la ressource. Par exemple, pour ajouter un `list` sur `books` :
    
```json
{
  "name": "list",
  "path": "books",
  "httpMethod": "GET",
  "description": "Liste tous les livres",
  "parameters": {
    "pageSize": {
       "type": "integer", "format": "int32", "minimum": "1", "maximum": "100"
    },
    "pageToken": { "type": "string" }
  },
  "response": { "$ref": "ListBooksResponse" }
}
```
    
    Le service créera la Method associée à la ressource `"books"`. Notez que dans ce cas, `path` est `"books"` – puisque le chemin complet final sera `basePath + "books"` (et potentiellement plus si parent resources, etc.). S’il s’agissait d’un élément individuel, `path` contiendrait l’identifiant entre accolades, ex: `"books/{bookId}"` pour une méthode GET unique ou DELETE.
    
*   **PUT/PATCH `/discovery/v1/apis/{apiName}/{version}/resources/{resourceName}/methods/{methodName}`** – _Mettre à jour une méthode existante_. Permet de modifier la description, ajouter de nouveaux paramètres, ou marquer deprecated. Il faudra préciser comment traiter la mise à jour de schémas request/response si la structure change – idéalement en recréant la nouvelle version de schéma. Étant un ADR, on peut simplement noter que les modifications doivent maintenir la cohérence (p. ex. ne pas supprimer un schéma utilisé ailleurs sans mise à jour de références).
    
*   **DELETE `/discovery/v1/apis/{apiName}/{version}/resources/{resourceName}/methods/{methodName}`** – _Supprimer une méthode_. Ceci retirera l’entrée de la table Method et toutes ses Parameter associées. Il faudra éventuellement nettoyer les schémas s’ils ne sont plus référencés (on peut le faire manuellement ou ultérieurement via un outil de purge, ou décider de ne pas supprimer les schémas orphelins tout de suite pour ne pas casser d’autres choses).
    

Pour les **schémas** (messages) directement :

*   **POST `/discovery/v1/apis/{apiName}/{version}/schemas`** – _Ajouter un schéma global_. Dans le cas où l’on souhaite enregistrer un schéma réutilisable indépendamment d’une méthode. Le corps JSON attend la structure du schéma, par exemple :
    
    `{   "id": "Author",   "type": "object",   "description": "Un auteur de livre",   "properties": {      "name": { "type": "string", "required": true },      "birthdate": { "type": "string", "format": "date" }   } }`
    
    Le service crée l’entrée Message correspondante et les Field associés. Ce schéma pourra ensuite être référencé par des méthodes (`"$ref": "Author"`).
    
*   **PUT `/discovery/v1/apis/{apiName}/{version}/schemas/{schemaId}`** – _Mettre à jour un schéma_. On pourrait autoriser la mise à jour de la description ou l’ajout de nouveaux champs. La modification de la structure est délicate car elle peut impacter les méthodes existantes. Néanmoins, dans un contexte maîtrisé, on peut remplacer la définition en question. Notre design de base de données permet de versionner via le champ `revision` dans API, mais pas nativement sur un schéma individuel (on ferait des modifications en place ou on créerait un nouveau schéma et mettrait à jour les références).
    
*   **DELETE `/discovery/v1/apis/{apiName}/{version}/schemas/{schemaId}`** – _Supprimer un schéma_. Possible seulement si plus aucune méthode/paramètre/field ne l’utilise. Sinon l’opération sera refusée (on doit d’abord dissocier des méthodes ou les supprimer). On peut vérifier via les FKs (un `ON DELETE RESTRICT` sur message.id référencé par field.schema\_id, parameter.schema\_id, method.request/response).
    

Enfin, pour les **opérations Terraform** :

*   **POST** `/discovery/v1/apis/{apiName}/{version}/resources/{resourceName}/operations` – _Associer une méthode à un type d’opération_. Le corps JSON pourrait être du style :
    
    `{ "operation": "CREATE", "method": "create" }`
    
    c’est-à-dire on indique que la méthode nommée "create" dans cette ressource correspond à l’opération de création. Alternativement, on pourrait référencer par l’ID interne de méthode ou son nom complet. Le service chercherait la Method et créerait l’entrée `discovery.operation`. On ferait de même pour "GET" -> READ, "DELETE" -> DELETE, etc. (On pourrait aussi _inférer_ automatiquement ces correspondances par convention – par ex. méthode POST sans identifiant -> CREATE, GET avec {id} -> READ, etc. – mais une configuration explicite via cette API garantit la bonne correspondance même si l’API ne suit pas exactement les conventions).
    
*   **DELETE** `/discovery/v1/apis/{apiName}/{version}/resources/{resourceName}/operations/{operation}` – pour dissocier. (Ou un PUT pour changer l’association).
    

Dans l’ensemble, ces endpoints d’écriture forment un ensemble cohérent qui suit la structure de nos données. **Le format JSON utilisé en entrée reprend les mêmes champs que ceux présents en sortie dans le document de découverte**, ce qui facilite la compréhension. Par exemple, pour définir un paramètre dans une méthode, on envoie quasiment la même structure que celle qu’on retrouvera dans `"parameters"` du discovery document. Cela évite toute confusion de mapping.

La sécurité : ces endpoints d’administration seraient probablement restreints aux développeurs ou administrateurs du système (via auth et role adéquats), car ils modifient la configuration du catalogue d’API.

Transformation des Données vers .proto (gRPC)
---------------------------------------------

Une fois les métadonnées d’une API renseignées, un des objectifs est de **générer automatiquement un fichier .proto** correspondant, pour offrir une interface gRPC équivalente. Voici comment se fait la correspondance :

*   Chaque **API** enregistrée pourrait correspondre à un **fichier .proto distinct**, avec un _package_ dérivé du nom de l’API (par ex. `francenuage.library.v1`). On peut inclure un champ `option go_package` etc., si on veut générer du code dans un langage cible. Le service de découverte pourra fournir ce .proto via un endpoint (éventuellement `GET /discovery/v1/apis/{api}/{version}/proto` par exemple), ou via un outil de ligne de commande.
    
*   Pour chaque **schéma** (Message) de type objet dans l’API, on va créer un **message protobuf** du même nom. Les champs de l’objet deviennent des champs du message proto, avec un type converti :
    
    *   Types JSON vs proto : `string` -> `string`, `integer` (format int32 ou int64) -> `int32` ou `int64`, `number` (float/double) -> `double` (par défaut), `boolean` -> `bool`.
        
    *   Si un champ a un schéma `$ref` vers un autre Message (objet), on utilisera ce message comme type de champ (message imbriqué).
        
    *   Les champs `repeated` -> on marquera le champ proto comme `repeated`.
        
    *   Les valeurs `enum` sur un champ ou paramètre pourraient se traduire en un **type enum proto**. Par exemple, le champ `status` de Book avec {"AVAILABLE","BORROWED"} pourrait devenir un enum `Status { AVAILABLE=0; BORROWED=1; }` dans le .proto. Cependant, transformer chaque petit ensemble de strings en enum proto peut complexifier les références (surtout si ces valeurs ne sont pas utilisées partout ou amenées à changer). Une alternative est de garder ces champs en `string` dans le proto, mais ce serait moins propre côté typing. Pour maximiser le typage, on peut générer des enums proto pour chaque champ énuméré distinct (en nommant par ex `Book.StatusEnum`). Dans cet ADR, on peut supposer qu’on génèrera des enums globaux ou imbriqués par message quand pertinent.
        
    *   Les noms de champs proto doivent être en lower\_snake\_case typiquement. Nos champs JSON sont souvent lowerCamelCase. On peut convertir lors de la génération (ex: `nextPageToken` -> `next_page_token`). Ou choisir de garder CamelCase si on n’adhère pas strictement aux conventions proto (mais idéalement on suit les conventions). L’outil de génération fera cette conversion de style.
        
    *   Chaque champ se voit assigner un **numéro de champ unique** dans le message proto. On peut les numéroter séquentiellement dans l’ordre naturel ou explicite (ex: ordre d’insertion ou ordre alphabétique constant pour stabilité). Ce détail sera implémenté par le générateur, l’essentiel est d’assurer la stabilité des numéros entre versions successives pour compatibilité (si un schéma évolue, on ne réutilise pas un numéro abandonné, etc. – ceci relève de la gouvernance des versions d’API plus que de l’ADR initial).
        
*   Pour chaque **ressource** (Resource) ayant des opérations CRUD, on crée un **service gRPC** correspondant, avec des RPC pour chaque opération supportée. Par exemple, la ressource `books` donnera lieu à un service `BooksService` (ou juste `Books` selon convention Google). Les RPC incluront :
    
    *   `ListBooks(ListBooksRequest) returns (ListBooksResponse)`
        
    *   `GetBook(GetBookRequest) returns (Book)`
        
    *   `CreateBook(CreateBookRequest) returns (Book)`
        
    *   `DeleteBook(DeleteBookRequest) returns (Empty)` (en utilisant un message Empty standard ou généré)
        
    
    Si une opération n’existe pas (ex: pas d’update), on ne la met pas. On peut également générer un `UpdateBook` si l’API a une méthode PUT/PATCH.
    
    _Mapping des messages de requête/réponse_ :
    
    *   `ListBooksRequest` par exemple contiendra les paramètres de requête de la méthode list – ici `page_size`, `page_token` – et possiblement un champ pour le parent si la ressource est subordonnée (ex: si on listait les instances d’un project, on aurait un champ `project_id`). Dans notre exemple, books est top-level, donc pas de parent.
        
    *   `ListBooksResponse` correspond déjà à un schéma existant (celui défini dans `schemas`), on peut le réutiliser tel quel comme message proto (juste en convertissant ses champs).
        
    *   `GetBookRequest` contiendra la clé du livre, par ex `book_id` (string) – correspondant au paramètre path `{bookId}`.
        
    *   `CreateBookRequest` contiendra éventuellement le Book à créer. On peut avoir deux approches : (a) inclure directement un champ `book` de type `Book` dans la requête, (b) “flatten” les champs de Book dans la requête. Google dans ses API gRPC a souvent une approche où la requête contient un champ pour l’entité à créer, plus le parent id éventuellement. Ici par ex: `CreateBookRequest { Book book = 1; }`. On privilégie cette approche (plus propre et alignée Google Cloud).
        
    *   `DeleteBookRequest` contiendra le `book_id`.
        
    
    Pour clarifier, le générateur peut s’appuyer sur la table Operation pour savoir quel method est GET, CREATE, etc., et générer le RPC au bon nom. _Convention de nommage_ : on utilisera la catégorie d’opération (create, get, list, update, delete) + le nom singulier de la ressource. On pourrait déduire le singulier du nom (books -> book), ou stocker quelque part (ex: on peut ajouter dans Resource un champ `schema_id` pointant vers le schéma principal de cette ressource – Book dans l’exemple – ce qui donne le nom singulier). Dans notre cas, on voit que le schéma principal se nomme comme la ressource en singulier souvent, on peut donc retrouver Book depuis books. Supposons qu’on ait cette info, alors on nomme RPC CreateBook, etc.
    
    Si l’API a des méthodes additionnelles non CRUD (par ex: une action `sendNotification`), on peut les intégrer aussi : par ex. méthode `sendNotification` sur resource `books` -> RPC `SendBookNotification(SendBookNotificationRequest) returns (SendBookNotificationResponse)`. On suivra alors le `name` de la méthode pour nommer le RPC (en UpperCamelCase). Ce cas nécessite que le générateur traite les méthodes sans opération type particulière.
    
*   Les **services gRPC** générés seront groupés éventuellement par API ou par sous-namespace. On peut mettre tous les services d’une API dans un même proto (c’est le plus simple). Alternativement, on peut décider d’avoir un service global par API (monolithique) au lieu d’un par ressource, mais ce serait moins organisé. Le standard gRPC, notamment chez Google Cloud, est d’avoir un service par type principal (d’où notre choix).
    
*   On ajoutera aussi dans le .proto les **options HTTP** (google.api.http option) pour chaque RPC, afin de documenter le mapping vers REST. Ceci permettrait à l’API Gateway gRPC (ou aux outils comme grpc-httpjson-transcoding) de router les appels REST vers ces RPC. Ces options reprennent essentiellement les `path` et verbes de nos Method. Ex:
    
```proto
service Books {
  rpc ListBooks(ListBooksRequest) returns (ListBooksResponse) {
    option (google.api.http) = {
      get: "/library/v1/books"
    };
  }
  rpc GetBook(GetBookRequest) returns (Book) {
    option (google.api.http) = {
      get: "/library/v1/books/{book_id}"
    };
  }
  rpc CreateBook(CreateBookRequest) returns (Book) {
    option (google.api.http) = {
      post: "/library/v1/books",
      body: "book"
    };
  }
  rpc DeleteBook(DeleteBookRequest) returns (google.protobuf.Empty) {
    option (google.api.http) = {
      delete: "/library/v1/books/{book_id}"
    };
  }
}
```    

    Ces options ne sont pas obligatoires pour la génération .proto, mais soulignent la cohérence entre ce qu’on stocke et l’éventuelle utilisation d’un protocole unifié. Cela montre aussi comment **le design de nos métadonnées est aligné avec une possible API gRPC**.
    
*   Concernant les **imports proto** : on importera `google/protobuf/empty.proto` pour l’utiliser dans Delete par ex. On pourrait aussi générer un fichier commun pour tous les types courants (Empty, etc.).
    

En somme, la transformation suit ces règles mécaniques. Une étape de génération parcourra les entités stockées:

1.  Générer toutes les définitions de messages proto à partir de `discovery.message` (et `field`).
    
2.  Générer les enums proto nécessaires (d’après `parameter_enum` et `field_enum`).
    
3.  Générer les services et RPC d’après `resource` et `method`/`operation`.
    
4.  Insérer les options HTTP basées sur `method.http_method` et `method.path`.
    

Le résultat est un `.proto` complet pour l’API, qui peut être compilé pour obtenir des clients gRPC. L’**avantage** est d’assurer que **l’interface gRPC reste synchronisée** avec l’interface REST documentée. Si l’API évolue (ex: ajout d’un champ dans un schéma ou d’un nouvel endpoint), on met à jour la découverte, on régénère le proto, les deux sont cohérents.

Génération du Provider Terraform
--------------------------------

L’autre livrable important est la génération d’un **provider Terraform** permettant de gérer les ressources offertes par les APIs France-Nuage. Grâce à notre modèle, la génération peut être automatisée ainsi :

*   Chaque **ressource** (au sens API, table Resource) correspond généralement à une **ressource Terraform** (type de ressource gérable). Par exemple, la ressource API `books` devient un type Terraform `francenuage_library_book`. (Convention typique: `<provider>_<api>_<resourceSingulier>`). Si une API a plusieurs ressources, chaque aura son mapping.
    
*   À partir des métadonnées, on connaît tous les **champs de la ressource** (via le schéma principal associé, ici `Book`). Pour construire la définition Terraform, on distingue :
    
    *   Les champs **configurables** par l’utilisateur (généralement ceux nécessaires à la création, ou modifiables). Dans `Book`, par exemple, `title`, `author` seraient configurables.
        
    *   Les champs **en lecture seule/générés par le serveur** (ex: `id`, ou d’autres champs calculés). On doit marquer ces champs comme `Computed` dans Terraform.
        
    *   On peut déduire cela en comparant le schéma de requête de création et le schéma de réponse : si un champ est présent dans la réponse (`Book`) mais pas fourni dans la requête (`Book` fourni lors du create, potentiellement sans id), alors ce champ est _output only_. Dans notre cas, `id` peut être généré serveur, donc on le marquerait computed. Si on a des champs timestamps de création, pareil.
        
    *   On peut affiner en ajoutant une méta-donnée dans `field` pour signaler `readonly` ou `output_only`. Notre modèle de base ne l’a pas explicitement, mais on peut gérer par convention ou extension.
        
*   Le provider Terraform en Go (en utilisant le Terraform Plugin SDK) nécessite d’écrire du code pour **chaque ressource** avec les opérations `Create`, `Read` (ou `Read`/`List`), `Update`, `Delete`. Grâce à la table `operation`, on sait quel endpoint correspond à quelle action:
    
    *   **Create**: on a l’URL (method.path relative au baseUrl) et verbe (POST typiquement). On génère du code qui prendra les champs définis dans Terraform (depuis le state) et construira un appel HTTP. Par exemple, pour créer un `book`, on fera un POST sur `.../books` avec un JSON body contenant title/author fournis. L’id étant renvoyé par la réponse, on l’extraira pour enregistrer dans le state.
        
    *   **Read**: typiquement GET sur `/books/{id}`. On génère le code pour faire l’appel GET en fournissant l’ID connu (stocké dans l’état Terraform), puis mapper la réponse JSON aux attributs du Terraform state (ainsi Terraform peut détecter toute dérive).
        
    *   **Update**: si disponible (PUT/PATCH /books/{id}), on envoie les champs modifiés. On peut comparer l’état ou simplement envoyer tout ou partiel selon API. Le code sera généré pour appeler la méthode d’update correspondante. S’il n’y a pas de méthode update (certaines API REST n’autorisent pas la modification, seulement recreate), on ne génère pas la fonction Update et Terraform saura qu’il faut recréer la ressource en cas de changement.
        
    *   **Delete**: appel DELETE /books/{id}, puis marquer la ressource absente.
        
*   Pour les **paramètres d’entrée** de chaque appel, on utilise les définitions de `parameter` et `field`. Par ex, pour Delete, on sait qu’il faut passer bookId dans l’URL (notre param path). On générera le code pour construire l’URL en insérant l’ID du state. Pour List (si on implémente data source), on sait quels query params on peut accepter (pageSize etc.), on pourrait paramétrer la data source avec ces options.
    
*   On doit également générer la définition du **schéma Terraform** (pas confondre avec schéma JSON) pour chaque ressource. En Terraform (SDKv2 par ex), on définirait un map de `schema.Schema{}` en Go, listant chaque attribut:
    
    *   name: type (SchemaTypeString, Int, Bool, List/Object for complex), required/optional/computed, etc.  
        On a toutes ces infos :
        
        *   type via `field.type_code`,
            
        *   si required (likely use that unless overridden by default or so),
            
        *   if computed (deduce output-only).
            
        *   if repeated/array, on set MaxItems=... or TypeList/TypeSet accordingly.
            
        *   if object nested, Terraform can do nesting but often providers flatten or use separate resource for nested. On peut initialement représenter un champ objet comme TypeList of single element with its own sub-schema or TypeMap. Pour ne pas trop détailler, disons qu’on peut a minima représenter une structure JSON arbitraire via TypeMap (string->interface) to pass transparently. Mais mieux est de générer un sub-schema for object fields (Terraform supporte bien les structures imbriquées).
            
        *   enumerations: Terraform n’a pas de type enum natif, mais on peut valider via `ValidateFunc` ou listing possible values. On peut générer un ValidateFunc qui check la valeur dans la liste `field_enum` ou `parameter_enum`. Ou au moins documenter que possible values are X, Y.
            
*   Concernant les **dépendances ou id** : Terraform nécessite un champ `id` pour chaque resource (identifiant unique). Dans notre provider, on définira que le champ `id` (computed) correspond à l’identifiant renvoyé par l’API (par ex. un entier ou string unique). On extraira ce champ de la réponse du Create ou List/Get. Notre modèle par chance a identifié généralement un champ d’id dans le schema (ex: Book.id). On peut ajouter conventionnellement que tout ressource a un champ `id` ou quelque chose d’identifiant.
    
*   **Data Sources** : En plus des resources (qui sont CRUD), on peut générer des **data sources** Terraform pour les _listages_. Par exemple, pouvoir faire `data "francenuage_library_books" { ... }` qui utiliserait la méthode list pour sortir tous les books (ou filtrer). C’est un plus. On générerait un data source for each LIST operation that exists. Ce data source retournera une liste d’objects ou un attribut, etc. La génération est similaire, mais par convention Terraform data sources sont read-only (just implementing a Read that calls GET or list).
    
*   **Fichier provider global** : On assemblera le code provider en enregistrant chaque resource type et data source. On peut en outre générer des tests de base et la documentation MD à partir du modèle (beaucoup de providers font ça – ex: champs description => doc field description).
    

En résumé, la génération du provider repose sur **la correspondance directe entre l’API REST et les opérations Terraform**. Grâce aux associations `discovery.operation`, on sait quel endpoint correspond à `Create`, `Read`, etc. On a les schémas pour savoir quels champs exposer. **Le fait d’utiliser la même source de vérité (le catalogue)** élimine le risque d’incohérence. Par exemple, si on ajoute un nouveau champ dans l’API, il suffit de mettre à jour le schéma dans la découverte ; la prochaine génération du provider inclura ce champ (soit comme updatable soit read-only selon le cas).

Cette automatisation apporte un énorme gain : plutôt que coder manuellement chaque ressource Terraform, on aura un _provider France-Nuage_ toujours à jour couvrant toutes les APIs. Cela facilite l’adoption de nouvelles fonctionnalités par les utilisateurs rapidement.

Justification des Choix Techniques
----------------------------------

### PostgreSQL comme base de stockage

Le choix de PostgreSQL s’impose pour plusieurs raisons : d’abord sa **fiabilité et ses garanties ACID** conviennent pour stocker des métadonnées critiques (on veut éviter toute corruption du catalogue d’API). Ensuite, le modèle relationnel de Postgres s’adapte très bien à nos données structurées et interconnectées. On bénéficie du **SQL** pour faire des jointures complexes lors de la reconstruction des documents de découverte ou de vérifications d’intégrité (par ex: lister toutes les méthodes orphelines sans ressource, etc.). PostgreSQL offre également des types JSONB si on avait besoin de stocker des portions non structurées, mais ici on a défini un schéma clair.

En outre, PostgreSQL est **open-source et maîtrisé** dans le contexte France-Nuage, ce qui correspond à nos exigences de souveraineté. Il s’intègre bien avec les outils de migration et de déploiement existants. Enfin, il permet d’évoluer (on peut ajouter des colonnes, des index facilement en cas de besoins de performance).

Une alternative NoSQL aurait compliqué la garantie de cohérence (par exemple, assurer que chaque `$ref` pointe vers un schéma existant aurait nécessité de la logique applicative supplémentaire). Le SQL nous permet d’appliquer ces contraintes au niveau base, renforçant la fiabilité du système.

### Usage des schémas PostgreSQL pour modulariser

Découper nos tables entre schémas `discovery` et `ref` apporte une **clarté**. Les tables `ref` contiennent de petites listes de valeurs autorisées qui sont conceptuellement différentes des données métier stockées. En les isolant, on indique clairement leur rôle. Cela facilite aussi la gestion des sauvegardes ou exports : on peut par exemple peupler le schéma `ref` avec des données par défaut à l’initialisation du service (les verbes HTTP, etc.), séparément du reste.

Si à l’avenir on souhaite permettre à différentes organisations de définir leurs APIs dans le même service en isolation, on pourrait introduire un schéma par organisation, mais ce n’est pas un besoin actuel. Néanmoins, le découpage existant est un premier niveau d’organisation.

### Tables d’énumération vs ENUM natifs

Comme mentionné, nous avons explicitement évité les colonnes de type `ENUM` SQL. Celles-ci posent problème en termes d’**évolution du schéma** : ajouter une valeur à un ENUM nécessite une migration DDL verrouillant potentiellement la table, ce qui n’est pas désirable en production. De plus, toutes les valeurs possibles doivent être connues à l’avance. En utilisant à la place des **tables de référence avec clés étrangères**, on gagne en flexibilité :

*   On peut insérer de nouvelles valeurs dynamiquement (par exemple si on décide de supporter un nouveau verbe HTTP futur).
    
*   On peut associer des métadonnées aux valeurs (des descriptions, un ordre, un statut de dépréciation, etc.).
    
*   On conserve l’intégrité via les FK (on ne pourra pas mettre un verbe qui n’est pas listé dans `ref.http_method`, par ex).
    

C’est une pratique courante pour éviter les pièges des ENUM SQL[sitepoint.com](https://www.sitepoint.com/community/t/using-enum-vs-check-constraint-vs-lookup-tables/6704#:~:text=enums%20and%20check%20constraints%20require,like%20any%20other%20user%20data). Notre cas s’y prête particulièrement parce que certaines listes pourraient s’agrandir (imaginons demain qu’on supporte d’autres protocoles que REST – on pourrait ajouter `graphql` dans ref.protocol sans altérer la table API, si on l’avait modélisé, etc.).

### Choix d’exposer une API RESTful style Google Discovery

En se calquant sur l’API Discovery de Google, nous profitons d’un **format éprouvé et documenté**. Ce format JSON est conçu pour être complet et générable. D’autres solutions existaient (OpenAPI YAML/JSON exposé via une URL, ou GraphQL introspection) mais chacune avait ses inconvénients. Le format Discovery offre une cohérence avec l’existant Google. Par exemple, certains outils open-source permettent déjà de convertir un document Discovery en spécification OpenAPI[github.com](https://github.com/stackql/google-discovery-to-openapi#:~:text=Google%20Discovery%20to%20OpenAPI%203,for%20Google%20Cloud%20APIs) ou inversement, ce qui nous donne une interopérabilité.

De plus, en exposant ces données via une API REST, on reste dans une approche standard web, facile à appeler depuis n’importe quel script ou outil (HTTP GET renvoyant JSON). C’est _auto-documentant_ : un simple GET sur le catalogue donne toutes les API disponibles, ce qui peut alimenter des portails ou interfaces.

L’**aspect “compatible Google”** est potentiellement stratégique : les développeurs ayant utilisé les Google APIs seront en terrain connu. Également, cela nous force à structurer proprement nos métadatas.

Enfin, fournir une API de lecture permet la mise en place de mécanismes de _cache_ ou de _versionnement_ des documents. Par exemple, on peut mettre un cache HTTP sur les responses Discovery (puisqu’elles ne changent qu’à l’update de l’API), ce qui améliore les performances côté clients.

### Génération de .proto gRPC pour gRPC

La décision de générer des `.proto` vise à encourager l’utilisation de **gRPC** pour les communications internes ou externes. gRPC apporte de nombreux bénéfices : efficacité binaire, streaming bidirectionnel, etc. Dans un contexte microservices cloud, il peut être plus adapté que REST pour certaines interactions. En proposant automatiquement une interface gRPC, on donne le choix aux consommateurs de l’API d’utiliser REST ou gRPC selon leurs besoins.

Techniquement, comme nos APIs suivent un style RESTful assez classique (ressources, méthodes CRUD), il est naturel de les mapper à des RPC. En fait, OpenAPI et gRPC ne sont pas si éloignés conceptuellement[cloud.google.com](https://cloud.google.com/blog/products/api-management/understanding-grpc-openapi-and-rest-and-when-to-use-them#:~:text=an%20OpenAPI%20API%20is%20very,combine%20the%20parameters%20with%20the)[cloud.google.com](https://cloud.google.com/blog/products/api-management/understanding-grpc-openapi-and-rest-and-when-to-use-them#:~:text=match%20at%20L202%20mapping%2C%20while,details%20using%20a%20predefined%20mapping) – ce sont deux façons de décrire des RPC, l’une textuelle sur HTTP, l’autre binaire sur HTTP/2\. En générant les .proto, on permet potentiellement d’implémenter un _transcoding REST/gRPC_ sans écrire manuellement la correspondance.

Par ailleurs, utiliser les protos comme artefact permet de **générer des clients dans de nombreux langages** via `protoc`. Cela complète l’offre : un développeur pourrait récupérer le proto de l’API et générer un stub client gRPC en Java, Go, Python… qui suit exactement le contrat. C’est un gain de productivité et de fiabilité (moins d’erreurs d’implémentation du client HTTP à la main).

En interne, France-Nuage pourrait choisir d’implémenter les services directement en gRPC et d’utiliser un proxy (Envoy + grpc-json transcoder par ex) pour le REST. Dans ce cas, la découverte pourrait être alimentée par les protos directement (bien que nous partions de la découverte vers le proto dans cette ADR, l’inverse serait possible). Notre solution laisse ces choix ouverts tout en gardant une source unique de vérité.

### Justification de la génération Terraform

Terraform est largement utilisé pour piloter des infrastructures. En fournissant un provider Terraform pour France-Nuage, on **facilite l’intégration** des services dans l’Infrastructure as Code des utilisateurs. Le développer à la main serait très coûteux et sujet à divergence si les APIs évoluent souvent. L’automatiser garantit une **synchronisation parfaite** avec la définition de l’API : chaque ressource Terraform correspond exactement aux endpoints disponibles.

Ce choix renforce l’adoption des services France-Nuage, car les équipes DevOps pourront immédiatement consommer les nouvelles fonctionnalités via Terraform dès qu’elles sont ajoutées. Cela cadre avec l’objectif de France-Nuage de proposer une expérience développeur moderne et fluide.

Techniquement, la faisabilité est bonne : de nombreux providers (ex: AWS) sont générés partiellement à partir de descriptions d’API. Notre cas est plus simple car nous maîtrisons l’ensemble. En générant du code, on s’assure de minimiser les erreurs. Il faudra bien sûr tester le provider, mais comme on utilise le même source de données, si la définition API a été testée, le provider devrait fonctionner en conséquence.

Le fait d’avoir modélisé explicitement les opérations et schémas rend cette génération quasi mécanique. On a fait le **choix d’isoler les correspondances d’opérations dans une table** justement pour rendre explicite le lien entre REST et actions CRUD, plutôt que de le déduire implicitement. Cela évite les ambiguïtés (ex: s’il y avait plusieurs GET, on sait lequel est LIST car marqué comme tel). Cette légère duplication de l’info est justifiée par le bénéfice côté génération.

### Évolutivité et maintenabilité

L’architecture proposée est **modulaire**. Ajouter un nouveau type d’information ne remet pas en cause l’ensemble : par exemple, si on voulait supporter aussi une sortie en format OpenAPI, on pourrait ajouter un convertisseur depuis les mêmes tables. Le modèle relationnel peut paraître verbeux, mais il est **normalisé** et évite les redondances, ce qui simplifie les mises à jour (on modifie une valeur à un endroit).

En termes de performance, le chargement d’un document de découverte demande plusieurs jointures (API -> resources -> methods -> params, etc.). Avec des index appropriés (sur FK, etc.), Postgres peut facilement assembler même des documents complexes en quelques dizaines de millisecondes. De plus, le cache HTTP ou l’appel ponctuel (on ne liste pas les APIs à chaque requête utilisateur, c’est plutôt utilisé aux phases d’intégration) fait que ce n’est pas un goulot d’étranglement.

Nous avons également isolé certaines choses potentiellement volumineuses : par ex, la description textuelle des schémas (documentation) pourrait être longue, mais cela reste du texte. PostgreSQL gère bien le stockage texte, et on peut externaliser des docs plus lourdes via `documentationLink`.

La génération de `.proto` et du provider est elle aussi scalable car réalisée hors-ligne ou à la demande. Envisageons le cycle de vie : à chaque modification d’une API dans la découverte, on peut déclencher (manuellement ou via pipeline) la régénération du proto et du provider. Ces outputs versionnés peuvent être diffusés (publication d’une nouvelle version du provider Go, etc.). Ceci peut s’automatiser et ne sollicite pas outre mesure le service lui-même (qui ne fait que lire ses tables).

En conclusion, cette décision d’architecture **répond aux besoins** en fournissant un **point central** pour gérer les définitions d’API et en tirant parti de ces définitions pour plusieurs usages (REST discovery, gRPC, Terraform). Elle s’appuie sur des technologies robustes (PostgreSQL, REST/HTTP, JSON, gRPC) alignées avec l’écosystème open-source. Chaque choix a été motivé par la recherche de cohérence, de maintenabilité et d’automatisation, ce qui devrait pérenniser la solution dans le contexte de France-Nuage.