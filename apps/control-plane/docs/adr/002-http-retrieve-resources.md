# Framework Choice for API Management

## Status

Accepted

## Context

On nous apprend toujours dans les écoles Française qu'une API Rest doit avoir une façons de récupérer sa donnée.

### Cas numéro 1 :

Imaginons que nous souhaitons récupérer la liste des projets d'une organization :

```
GET 'http://localhost:3333/api/v1/organizations/<<organization__id>>'
GET 'http://localhost:3333/api/v1/organizations/<<organization__id>>/folders'
GET 'http://localhost:3333/api/v1/organizations/<<organization__id>>/folders/<<folder__id>>'
GET 'http://localhost:3333/api/v1/organizations/<<organization__id>>/folders/<<folder__id>>/project'
```

Dans notre exemple, nous devons effectué 4 requêtes depuis notre Front ce qui est assez lourd en terme de gestion.
Nous devons réfléchir à un système qui nous permet de récupérer les données que nous souhaitons en une seul requête
qui nous retourne un JSON avec un ensemble de resource imbriquer de cette manière :

```json
// http://localhost:3333/api/v1/organizations/<<organization__id>>
{
  "organization__id": "219bec10-c7d4-441e-9eeb-d945986c94b6",
  "name": "France Nuage",
  "folders": [
    {
      "folder__id": "219bec10-c7d4-441e-9eeb-d945986c94b6",
      "name": "France Nuage",
      "projects": [] // here we obtain our data
    }
  ]
}
```

## Decision

### Solutions

#### GraphQL

#### jsonAPI

### Solution choisie

Mise en place

## Consequences
