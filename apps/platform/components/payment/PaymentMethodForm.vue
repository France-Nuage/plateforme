<template>
  <div>
    <div id="paymentForm"></div>
    <c-alert v-if="error" :title="error" variant="danger" />
    <!--    <button @click="handleSubmit">plop</button>-->
  </div>
</template>

<script setup lang="ts">
import { loadStripe } from "@stripe/stripe-js";
import appearance from "./appearance.json";
import CAlert from "~/components/alert/CAlert.vue";

const stripe = ref(null);
const paymentElements = ref(null);
const elements = ref(null);
const error = ref(null);

const { $api } = useNuxtApp();
const emit = defineEmits(["close"]);
const { organization } = storeToRefs(useNavigationStore());

const initialize = () => {
  $api()
    .payment.methods.post({ organization__id: organization.value.id })
    .then((data) => {
      elements.value = stripe.value.elements({
        clientSecret: data.clientSecret,
      });
      paymentElements.value = elements.value.create("payment");
      paymentElements.value.mount(document.getElementById("paymentForm"));
    });
};

onMounted(() => {
  loadStripe(
    "pk_test_51QOjXfGC39dHI2PZO3zoZjJXzxR1dJfZDaf5eXE0aUgizYIlDSKC2jS9liadlBe7sXoiXadO4FWF5fZ4OiHst27p00Ij69VMMV",
  ).then((response) => {
    stripe.value = response;
    // stripe._locale = $i18n.locale;
    initialize();
  });
});

const route = useRoute();

const handleSubmit = () => {
  if (!stripe.value || !paymentElements.value) {
    return;
  }

  stripe.value
    .confirmSetup({
      elements: elements.value,
      confirmParams: {
        return_url: "http://localhost:3001" + "/settings/payment-methods",
      },
    })
    .then((response) => {
      if (response.error) {
        error.value = response.error.message;
        return;
      }
      emit("close", true);
    })
    .catch((e) => {
      error.value = e.message;
    });
};
</script>
