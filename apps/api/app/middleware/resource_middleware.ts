import type { HttpContext } from '@adonisjs/core/http'
import type { NextFn } from '@adonisjs/core/types/http'
import Organization from '#models/resource/organization'
import Folder from '#models/resource/folder'
import Project from '#models/resource/project'

export default class ResourceMiddleware {
  async handle(ctx: HttpContext, next: NextFn) {
    const resources: Array<{ type: 'organization' | 'folder' | 'project'; id: string }> = []

    const headers = ctx.request.headers()

    const organizationId = headers['x-organization']
    const folderId = headers['x-folder']
    const projectId = headers['x-project']

    let organization: Organization | null = null
    let folder: Folder | null = null
    let project: Project | null = null

    /**
     * 1) Chargement des ressources en fonction des headers
     */

    // Si ProjectId est présent, on charge le project
    if (projectId) {
      project = await Project.find(projectId)
      if (!project) {
        // throw new HttpException('Project not found', 404)
      }
    }

    // Si FolderId est présent, on charge le folder
    if (folderId) {
      folder = await Folder.find(folderId)
      if (!folder) {
        // throw new HttpException('Folder not found', 404)
      }
    }

    // Si OrganizationId est présent, on charge l'organization
    if (organizationId) {
      organization = await Organization.find(organizationId)
      if (!organization) {
        // throw new HttpException('Organization not found', 404)
      }
    }

    /**
     * 2) Compléter les ressources manquantes
     *    - Si on a un project mais pas de folder, on va chercher son folder parent
     *    - Si on a un project ou un folder mais pas d'organization, on va la chercher à partir du folder
     */

    // Si on a un project et pas de folderId
    if (project && !folder) {
      folder = await Folder.find(project.folderId)
      if (!folder) {
        // throw new HttpException('Folder for this project not found', 404)
      }
    }

    // Si on a un folder et pas d'organization
    if (folder && !organization) {
      organization = await Organization.find(folder.organizationId)
      if (!organization) {
        // throw new HttpException('Organization for this folder not found', 404)
      }
    }

    /**
     * 3) Vérifier la cohérence de la hiérarchie
     *    - Si project existe, vérifier que folder.folderId = project.folderId
     *    - Si folder existe, vérifier qu’il appartient à l’organisation
     *    - Si project existe, vérifier indirectement qu’il appartient à la bonne organisation (via folder)
     */
    if (project && folder && project.folderId !== folder.id) {
      // throw new HttpException('Inconsistent project/folder relation', 400)
    }

    if (folder && organization && folder.organizationId !== organization.id) {
      // throw new HttpException('Inconsistent folder/organization relation', 400)
    }

    // Si organization et project sont donnés sans folder, on peut vérifier que le project appartient indirectement à l'organisation
    if (project && organization && !folder) {
      const projFolder = await Folder.find(project.folderId)
      if (!projFolder || projFolder.organizationId !== organization.id) {
        // throw new HttpException('Project does not belong to the given organization', 400)
      }
    }

    if (organization) {
      resources.push({ type: 'organization', id: organization.id })
    }

    // On inclut le folder seulement si on l'a récupéré (en fonction des scenarios)
    // Par exemple, si on a organizationId et projectId mais pas folderId, on a choisi de ne pas forcément remonter le folder
    // Si vous souhaitez toujours remonter le folder s'il existe, vous pouvez laisser ce code tel quel.
    if (folder && (folderId || projectId)) {
      resources.push({ type: 'folder', id: folder.id })
    }

    if (project && projectId) {
      resources.push({ type: 'project', id: project.id })
    }

    // if (this.ctx) {
    //   if (ctx.request.headers()['x-project']) {
    //     this.resources = { type: 'project', id: ctx.request.headers()['x-project'] }
    //   } else if (ctx.request.headers()['x-folder']) {
    //     this.resources = { type: 'folder', id: ctx.request.headers()['x-folder'] }
    //   } else if (ctx.request.headers()['x-organization']) {
    //     this.resources = { type: 'organization', id: ctx.request.headers()['x-organization'] }
    //   }
    // }

    ctx.resources = resources

    /**
     * Call next method in the pipeline and return its output
     */
    const output = await next()
    return output
  }
}

/**
 * Notify TypeScript about i18n property
 */
declare module '@adonisjs/core/http' {
  export interface HttpContext {
    resources: Array<{ type: 'organization' | 'folder' | 'project'; id: string }>
  }
}
