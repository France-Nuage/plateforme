<template>
  <nuxt-layout>
    <div class="relative isolate bg-white px-6 py-24 sm:py-32 lg:px-8">
      <div class="mx-auto max-w-xl lg:max-w-4xl">
        <h2
          class="text-pretty text-4xl font-semibold tracking-tight text-gray-900 sm:text-5xl"
        >
          Parlons de votre projet
        </h2>
        <p class="mt-2 text-lg/8 text-gray-600">
          Nous aidons les entreprises à reprendre le contrôle sur leur destin en
          sortant du modèle de cloud traditionnel.
        </p>
        <div class="mt-16 flex flex-col gap-16 sm:gap-y-20 lg:flex-row">
          <form action="#" method="POST" class="lg:flex-auto">
            <div class="grid grid-cols-1 gap-x-8 gap-y-6 sm:grid-cols-2">
              <div>
                <label
                  for="first-name"
                  class="block text-sm/6 font-semibold text-gray-900"
                  >Prénom</label
                >
                <div class="mt-2.5">
                  <input
                    v-model="formData.firstname"
                    type="text"
                    name="first-name"
                    id="first-name"
                    autocomplete="given-name"
                    class="block w-full rounded-md border-0 px-3.5 py-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm/6"
                  />
                </div>
              </div>
              <div>
                <label
                  for="last-name"
                  class="block text-sm/6 font-semibold text-gray-900"
                  >Nom</label
                >
                <div class="mt-2.5">
                  <input
                    v-model="formData.lastname"
                    type="text"
                    name="last-name"
                    id="last-name"
                    autocomplete="family-name"
                    class="block w-full rounded-md border-0 px-3.5 py-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm/6"
                  />
                </div>
              </div>
              <div class="sm:col-span-2">
                <label
                  for="email"
                  class="block text-sm/6 font-semibold text-gray-900"
                  >Email</label
                >
                <div class="mt-2.5">
                  <input
                    v-model="formData.email"
                    id="email"
                    name="email"
                    type="email"
                    class="block w-full rounded-md border-0 px-3.5 py-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm/6"
                    @blur="validateEmail"
                  />
                  <p v-if="emailError" class="text-red-500 text-sm mt-2">
                    {{ emailError }}
                  </p>
                  <!-- Show error -->
                </div>
              </div>
              <div class="sm:col-span-2">
                <label
                  for="company"
                  class="block text-sm/6 font-semibold text-gray-900"
                  >Entreprise</label
                >
                <div class="mt-2.5">
                  <input
                    v-model="formData.company"
                    id="company"
                    name="company"
                    type="text"
                    class="block w-full rounded-md border-0 px-3.5 py-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm/6"
                  />
                </div>
              </div>
              <div class="sm:col-span-2">
                <label
                  for="message"
                  class="block text-sm/6 font-semibold text-gray-900"
                  >Votre plus grosse douleur avec le cloud ?</label
                >
                <div class="mt-2.5">
                  <textarea
                    v-model="formData.message"
                    id="message"
                    name="message"
                    rows="4"
                    placeholder="Décrivez votre problème ou besoin lié au cloud et comment notre solution pourrait vous aider."
                    class="block w-full rounded-md border-0 px-3.5 py-2 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm/6"
                  ></textarea>
                </div>
              </div>
            </div>

            <div v-if="successSubmit" class="rounded-md bg-green-50 p-4">
              <div class="flex">
                <div class="shrink-0">
                  <CheckCircleIcon
                    class="size-5 text-green-400"
                    aria-hidden="true"
                  />
                </div>
                <div class="ml-3">
                  <h3 class="text-sm font-medium text-green-800">
                    Message envoyé
                  </h3>
                  <div class="mt-2 text-sm text-green-700">
                    <p>
                      Votre message a bien été envoyé, nous revenons vers vous
                      au plus vite.
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <div class="mt-10">
              <Button block @click="onSubmit"> Venez on parle ! </Button>
            </div>
            <p class="mt-4 text-sm/6 text-gray-500">
              En soumettant ce formulaire, j'accepte la
              <a href="/privacyPolicy" class="font-semibold text-primary"
                >politique de confidentialité</a
              >.
            </p>
          </form>
          <div class="lg:mt-6 lg:w-80 lg:flex-none">
            <div class="flex items-center space-x-4 col-span-3 xl:col-span-1">
              <Logo />
              <span class="font-semibold text-gray-900">France Nuage</span>
            </div>
            <figure class="mt-10">
              <blockquote class="text-lg/8 font-semibold text-gray-900">
                <p>
                  “Chez France-Nuage, nous croyons qu'il est temps de reprendre
                  le contrôle de votre destin. Notre mission est de vous aider à
                  vous affranchir des limites des solutions cloud
                  traditionnelles en vous offrant des alternatives sur mesure,
                  transparentes, et adaptées à vos besoins.”
                </p>
              </blockquote>
              <figcaption class="mt-10 flex gap-x-6">
                <img
                  src="/images/profil_kev.jpeg"
                  alt=""
                  class="size-12 flex-none rounded-full bg-gray-50"
                />
                <div>
                  <div class="text-base font-semibold text-gray-900">
                    Kévin GILLET
                  </div>
                  <div class="text-sm/6 text-gray-600">
                    Co-fondateur de France-Nuage
                  </div>
                </div>
              </figcaption>
            </figure>
          </div>
        </div>
      </div>
    </div>
  </nuxt-layout>
</template>

<script setup lang="ts">
import Logo from "~/components/Logo.vue";
import { CheckCircleIcon } from "@heroicons/vue/20/solid";
import { ref } from "vue";

const config = useRuntimeConfig();
const successSubmit = ref(false);

const formData = ref({
  email: "",
  lastname: "",
  firstname: "",
  company: "",
  message: "",
});

const emailError = ref("");

const validateEmail = () => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/; // Basic email validation regex
  if (!emailRegex.test(formData.value.email)) {
    emailError.value = "Veuillez entrer une adresse email valide.";
    return false;
  }
  emailError.value = ""; // Clear the error if email is valid
  return true;
};

const onSubmit = async (event: Event) => {
  event.preventDefault(); // Prevent the default form submission
  console.log("Starting form submission...");

  // Validate email before proceeding
  if (!validateEmail()) {
    console.error("Invalid email format:", formData.value.email);
    return; // Exit if email is invalid
  }

  // Log form data to verify correctness
  console.log("Form Data:", {
    email: formData.value.email,
    firstname: formData.value.firstname,
    lastname: formData.value.lastname,
    company: formData.value.company,
    message: formData.value.message,
  });

  try {
    const payload = {
      email: formData.value.email,
      attributes: {
        PRENOM: formData.value.firstname,
        NOM: formData.value.lastname,
        COMPANY: formData.value.company,
        MESSAGE: formData.value.message,
      },
      listIds: [2], // Ensure this matches your Brevo list IDs
      updateEnabled: true, // Update if the contact exists
    };

    console.log("Payload to be sent to Brevo:", payload);

    // Make the API call
    const response = await fetch("https://api.brevo.com/v3/contacts", {
      method: "POST",
      headers: {
        "api-key": config.public.brevoApiKey, // Log the API key for debugging
        "Content-Type": "application/json",
      },
      body: JSON.stringify(payload),
    });

    console.log("Response status:", response.status); // Log status code
    console.log("Response headers:", Array.from(response.headers.entries())); // Log all headers

    // Check for valid response before parsing
    const contentType = response.headers.get("content-type");
    console.log("Content-Type of response:", contentType);

    if (!response.ok) {
      // If the response status is not OK, log the error response
      const errorData = await response.text();
      console.error("Error response from Brevo:", errorData);
      return; // Exit the function
    } else {
      console.log("Contact saved successfully");
      successSubmit.value = true;

      // Clear the form after successful submission
      formData.value = {
        email: "",
        firstname: "",
        lastname: "",
        company: "",
        message: "",
      };
      console.log("Form fields cleared!");
    }

    // Parse JSON if content-type is application/json
    if (contentType && contentType.includes("application/json")) {
      const data = await response.json();
      console.log("Parsed JSON response from Brevo:", data);
    } else {
      console.warn("Response did not return JSON. Possible empty body.");
    }
  } catch (error) {
    // Log unexpected errors
    console.error("Unexpected error occurred:", error);
  }
};
</script>
