import { useState } from "react";
import type { Meta, StoryObj } from "@storybook/react";
import { instances } from "@/fixtures";
import { Instance } from "@/types";
import { AppSelect } from "./app-select";

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
} satisfies Meta<typeof AppSelect<Instance>>;

export default meta;
type Story = StoryObj<typeof AppSelect<Instance>>;

export const Primary: Story = {
  render: (props) => {
    const [value, onChange] = useState(props.options[0]);
    return <AppSelect {...props} onChange={onChange} value={value} />;
  },
};
