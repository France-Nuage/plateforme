<template>
  <div>
    <c-button size="sm" @click="isOpen = true">
      Accorder l'accès
    </c-button>

    <c-modal v-model="isOpen" title="Ajouter un utilisateur" @add="">
      <c-modal-body>

        <div class="mb-8">
          <h2 class="dark:text-white text-lg">Ajouter des comptes principaux</h2>
          <p class="dark:text-gray-400 text-sm">Les comptes principaux sont des utilisateurs, des groupes, des domaines ou des comptes de service.</p>
          <div class="flex gap-4 items-center w-full mt-4">
            <c-text-field id="email" name="email" placeholder="Renseigner un email" v-model="email" type="email" class="w-full"/>
            <c-button variant="primary" size="sm" @click="addEmail">+</c-button>
          </div>
          <div class="mt-4">
            <div v-for="person in persons" :key="person" class="dark:text-gray-400 flex items-center px-4 justify-between border-y dark:border-gray-700 py-4">

              <div>{{ person }}</div>
              <div>
                <button
                  class="w-6 h-6 transition-all duration-75 rounded-full hover:bg-gray-700 cursor-pointer flex items-center justify-center"
                  @click="onDelete(person)"
                >-</button>
              </div>
            </div>
          </div>
        </div>

        <div class="mb-8">
          <h2 class="dark:text-white text-lg">Attribuer des rôles</h2>
          <p class="dark:text-gray-400 text-sm">Les rôles sont composés d'ensembles d'autorisations et déterminent ce que le compte principal peut faire avec cette ressource.</p>

          <div class="flex flex-wrap gap-3 mt-4">
            <c-badge v-for="role in roles" remove variant="information" @remove="removeRole(role.id)">{{ role.id }}</c-badge>
          </div>

          <c-select-role
            id="role"
            v-model="selectedRole"
            name="role"
            placeholder="Selectionner un rôle"
            :roles-selected="roles"
          />

        </div>

      </c-modal-body>
    </c-modal>
  </div>
</template>

<script setup lang="ts">
import CButton from "~/components/forms/CButton.vue";
import CModal from "~/components/modal/CModal.vue";
import CTextField from "~/components/forms/CTextField.vue";
import CModalBody from "~/components/modal/CModalBody.vue";
import CSelectRole from "~/components/forms/select/CSelectRole.vue";
import CBadge from "~/components/badge/CBadge.vue";

const isOpen = ref(false);
const email = ref('');
const selectedRole = ref(null)
const roles = ref([])
const persons = ref([])

const addEmail = () => {
  if (!email.value) return
  if (!persons.value.includes(email.value)) {
    persons.value.push(email.value)
    email.value = ''
  }
}

const onDelete = (value) => {
  persons.value = persons.value.filter(person => person !== value)
}

watch(selectedRole, () => {
  if (selectedRole.value && !roles.value.includes(selectedRole.value)) {
    roles.value.push(selectedRole.value)
  }
})

const removeRole = (id) => {
  roles.value = roles.value.filter((i) => i.id !== id)
}

const add = () => {
  console.log({
    persons,
    roles,
  })
}
</script>
