import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { InstanceFormValue } from "@/types";
import { InstanceInput } from "./instance-input";
import { instance } from "@/fixtures";

const meta = {
  title: "Components / Instance Input",
  component: InstanceInput,
  tags: ["autodocs"],
  args: {},
} satisfies Meta<typeof InstanceInput>;

export default meta;
type Story = StoryObj<typeof InstanceInput>;

export const Primary: Story = {
  render: (props) => {
    const [value, onChange] = useState<InstanceFormValue>(instance());
    return <InstanceInput {...props} onChange={onChange} value={value} />;
  },
};
