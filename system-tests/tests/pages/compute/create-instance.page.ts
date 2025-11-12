import type { Locator, Page } from "@playwright/test";
import BasePage from "../base.page";

export class CreateInstancePage extends BasePage {
  locators: {
    createInstanceButton: Locator;
    editSnippetSwitch: Locator;
    instanceTypeField: Locator;
    nameField: Locator;
    projectField: Locator;
    snippetField: Locator;
  };

  /**
   * @inheritdoc
   */
  public constructor(page: Page) {
    super(page, "/compute/instances/create");
    this.locators = {
      createInstanceButton: page.getByRole('button', { name: 'Créer la nouvelle instance' }),
      nameField: page.getByLabel('Nom de l\'instance'),
      projectField: page.getByLabel('Projet'),
      instanceTypeField: page.getByLabel('Type d\'instance'),
      snippetField: page.getByLabel('Snippet'),
      editSnippetSwitch: page.getByRole('checkbox', { name: 'Editable' }),
    };
  }
}
