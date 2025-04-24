import type { Meta, StoryObj } from "@storybook/react";
import { hypervisors } from "@/fixtures";
import { HypervisorTable } from "./hypervisor-table";

const meta = {
  title: "Components / Hypervisor Table",
  component: HypervisorTable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    hypervisors: hypervisors(5),
  },
} satisfies Meta<typeof HypervisorTable>;

export default meta;
type Story = StoryObj<typeof HypervisorTable>;

export const Primary: Story = {};
