import type { Meta, StoryObj } from "@storybook/react";

import { HypervisorInput } from "./hypervisor-input";
import { useState } from "react";
import { Hypervisor } from "@/protocol";

const meta = {
  title: "Components / Hypervisor Input",
  component: HypervisorInput,
  tags: ["autodocs"],
  args: {
  },
} satisfies Meta<typeof HypervisorInput>;

export default meta;
type Story = StoryObj<typeof HypervisorInput>;

export const Primary: Story = {
  render: (props) => {
    const [value, onChange] = useState<Hypervisor>({ url: 'https://hypervisor.acme' });
    return <HypervisorInput {...props} onChange={onChange} value={value} />
  }
};
