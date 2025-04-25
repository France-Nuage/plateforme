import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { HypervisorFormValue } from "@/types";
import { HypervisorInput } from "./hypervisor-input";
import { hypervisor } from "@/fixtures";

const meta = {
  title: "Components / Hypervisor Input",
  component: HypervisorInput,
  tags: ["autodocs"],
  args: {},
} satisfies Meta<typeof HypervisorInput>;

export default meta;
type Story = StoryObj<typeof HypervisorInput>;

export const Primary: Story = {
  render: (props) => {
    const [value, onChange] = useState<HypervisorFormValue>({
      ...hypervisor(),
      authorizationToken: "",
    });
    return <HypervisorInput {...props} onChange={onChange} value={value} />;
  },
};
