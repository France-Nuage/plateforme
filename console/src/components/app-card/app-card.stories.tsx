import type { Meta, StoryObj } from "@storybook/react";

import { AppCard } from "./app-card";

const meta = {
  title: "Components / App Card",
  component: AppCard,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    children: "Card",
    header: <div>coucou</div>
  },
} satisfies Meta<typeof AppCard>;

export default meta;
type Story = StoryObj<typeof AppCard>;

export const Primary: Story = {};
