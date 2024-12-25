<template>
  <div class="flex flex-col gap-8">

    <c-card>
      <c-card-header title="Payment Methods" />
      <c-card-body>
        <div class="grid grid-cols-12 gap-10">

          <div class="col-span-3">
            <c-label light label="Payments for your subscription are made using the default card." for="email" />
          </div>
          <div class="col-span-9">
            <div class="w-full text-center text-gray-600">Aucune carte d'enregistrer</div>
          </div>
        </div>

      </c-card-body>
      <c-card-footer>
        <div class="flex w-full justify-end gap-4">
          <c-button variant="success" size="sm">Ajouter une carte</c-button>
        </div>
      </c-card-footer>
    </c-card>

    <c-card>
      <c-card-header title="Email Recipient" />
      <c-card-body>

        <div class="grid grid-cols-12 gap-10">

          <div class="col-span-3">
            <c-label light label="All billing correspondence will go to this email" for="email" />
          </div>
          <div class="col-span-9 flex flex-col gap-1">
            <c-text-field
              id="email"
              name="email"
              type="text"
              label="Adresse E-mail"
              placeholder="john@france-nuage.fr"
              v-model="taxNumber"
            />
          </div>

        </div>

      </c-card-body>
      <c-card-footer>
        <div class="flex w-full justify-end gap-4">
          <c-button variant="filled" size="sm">Annuler</c-button>
          <c-button variant="success" size="sm">Enregistrer</c-button>
        </div>
      </c-card-footer>
    </c-card>

    <c-card>
      <c-card-header title="Billing Address" />
      <c-card-body>

        <div class="grid grid-cols-12 gap-10">

          <div class="col-span-3">
            <c-label light label="This will be reflected in every upcoming invoice, past invoices are not affected" for="#" />
          </div>
          <div class="col-span-9 flex flex-col gap-1">
            <c-text-field
              id="name"
              name="name"
              type="text"
              placeholder="Address line 1"
              v-model="taxNumber"
            />
            <c-text-field
              id="slug"
              name="slug"
              type="text"
              placeholder="Address line 2"
              v-model="taxNumber"
            />
            <c-text-field
              id="city"
              name="city"
              type="text"
              placeholder="Country"
              v-model="taxNumber"
            />
            <div class="flex gap-4 w-full">
              <c-text-field
                id="country"
                name="country"
                type="text"
                class="w-full"
                placeholder="Country"
                v-model="taxNumber"
              />
              <c-text-field
                id="postal_code"
                name="postal_code"
                type="text"
                class="w-full"
                placeholder="Code postal"
                v-model="taxNumber"
              />
            </div>

          </div>

        </div>

      </c-card-body>
      <c-card-footer>
        <div class="flex w-full justify-end gap-4">
          <c-button variant="filled" size="sm">Annuler</c-button>
          <c-button variant="success" size="sm">Enregistrer</c-button>
        </div>
      </c-card-footer>
    </c-card>

    <c-card>
      <c-card-header title="Tax ID" />
      <c-card-body>

        <div class="grid grid-cols-12 gap-10">

          <div class="col-span-3">
            <c-label label="Add a tax ID to your invoices. Changes only apply to future invoices. Make sure the tax ID looks exactly like the placeholder text." light for="text_number" />
          </div>
          <div class="col-span-9 flex flex-col gap-1">
            <c-text-field
              id="text_number"
              name="text_number"
              type="text"
              label="Tax Number"
              placeholder="john@france-nuage.fr"
              v-model="taxNumber"
            />
          </div>

        </div>

      </c-card-body>
      <c-card-footer>
        <div class="flex w-full justify-end gap-4">
          <c-button variant="filled" size="sm">Annuler</c-button>
          <c-button variant="success" size="sm">Enregistrer</c-button>
        </div>
      </c-card-footer>
    </c-card>

  </div>
</template>

<script setup lang="ts">
import CTextField from "~/components/forms/CTextField.vue";
import CCardBody from "~/components/card/CCardBody.vue";
import CButton from "~/components/forms/CButton.vue";
import CCard from "~/components/card/CCard.vue";
import CCardFooter from "~/components/card/CCardFooter.vue";
import CLabel from "~/components/forms/CLabel.vue";
import CCardHeader from "~/components/card/CCardHeader.vue";

const taxNumber = ''
</script>
