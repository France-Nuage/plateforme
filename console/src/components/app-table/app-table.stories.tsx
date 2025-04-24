import type { Meta, StoryObj } from "@storybook/react";
import { instances } from "@/fixtures";
import { Instance } from "@/types";
import { AppTable } from "./app-table";

const meta = {
  title: "Components / App Table",
  component: AppTable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    columns: ["id", { key: "name", label: "Instance Name" }],
    rows: instances(5),
  },
} satisfies Meta<typeof AppTable<Instance>>;

export default meta;
type Story = StoryObj<typeof AppTable>;

export const Primary: Story = {};
