import type { Meta, StoryObj } from "@storybook/react";
import { fn } from "@storybook/test";

import { AppButton } from "./app-button";

const meta = {
  title: "Components / App Button",
  component: AppButton,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    children: "Button",
    onClick: fn(),
    size: "md",
  },
} satisfies Meta<typeof AppButton>;

export default meta;
type Story = StoryObj<typeof AppButton>;

export const Primary: Story = {};
