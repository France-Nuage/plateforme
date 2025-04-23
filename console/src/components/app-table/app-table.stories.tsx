import type { Meta, StoryObj } from "@storybook/react";
import { instances } from "@/fixtures";
import { InstanceInfo } from "@/protocol";
import { AppTable } from "./app-table";

const meta = {
  title: "Components / App Table",
  component: AppTable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    data: instances(10),
  },
} satisfies Meta<typeof AppTable<InstanceInfo>>;

export default meta;
type Story = StoryObj<typeof AppTable>;

export const Primary: Story = {};
