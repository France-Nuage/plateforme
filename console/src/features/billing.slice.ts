import { CheckoutResult, CreateCheckoutInput } from '@france-nuage/sdk';
import { createAsyncThunk } from '@reduxjs/toolkit';

import { ExtraArgument } from '@/store';

export const createCheckoutSession = createAsyncThunk<
  CheckoutResult,
  CreateCheckoutInput,
  { extra: ExtraArgument }
>('billing/createCheckoutSession', (data, { extra }) =>
  extra.services.billing.createCheckoutSession(data),
);
