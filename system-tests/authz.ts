import { v1 } from '@authzed/authzed-node';
import { ZedClientInterface } from '@authzed/authzed-node/dist/src/v1';

export class Authz {
  private client: ZedClientInterface;

  constructor() {
    this.client = v1.NewClient(process.env.SPICEDB_GRPC_PRESHARED_KEY!, process.env.SPICEDB_URL!, v1.ClientSecurity.INSECURE_PLAINTEXT_CREDENTIALS);
  }

  foo(resourceType: string, resourceId: string, relation: string, subjectType: string, subjectId: string) {
    const relationship = v1.Relationship.create({
      resource: v1.ObjectReference.create({
        objectType: resourceType,
        objectId: resourceId,
      }),
      relation,
      subject: v1.SubjectReference.create({
        object: v1.ObjectReference.create({
          objectType: subjectType,
          objectId: subjectId,
        }),
      }),
    });

    const request = v1.WriteRelationshipsRequest.create({
      updates: [
        v1.RelationshipUpdate.create({
          operation: v1.RelationshipUpdate_Operation.TOUCH,
          relationship,
        }),
      ],
    });

    return this.client.writeRelationships(request, (x) => console.log(x));
  }
}
