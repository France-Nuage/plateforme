import type { Meta, StoryObj } from "@storybook/react";
import { instances } from "@/fixtures";
import { InstanceTable } from "./instance-table";

const meta = {
  title: "Components / Instance Table",
  component: InstanceTable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    instances: instances(5),
  },
} satisfies Meta<typeof InstanceTable>;

export default meta;
type Story = StoryObj<typeof InstanceTable>;

export const Primary: Story = {};
