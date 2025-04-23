import type { Meta, StoryObj } from "@storybook/react";

import { AppInput } from "./app-input";

const meta = {
  title: "Components / App Input",
  component: AppInput,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {},
} satisfies Meta<typeof AppInput>;

export default meta;
type Story = StoryObj<typeof AppInput>;

export const Primary: Story = {};
