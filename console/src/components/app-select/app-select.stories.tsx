import type { Meta, StoryObj } from "@storybook/react";
import { InstanceInfo } from "@/protocol";
import { instances } from "@/fixtures";

import { AppSelect } from "./app-select";
import { useState } from "react";

const meta = {
  title: "Components / App Select",
  component: AppSelect,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  args: {
    format: (instance) => instance.name,
    options: instances(5),
  },
} satisfies Meta<typeof AppSelect<InstanceInfo>>;

export default meta;
type Story = StoryObj<typeof AppSelect<InstanceInfo>>;

export const Primary: Story = {
  render: (props) => {
    const [value, onChange] = useState(props.options[0]);
    return <AppSelect {...props} onChange={onChange} value={value} />;
  },
};
