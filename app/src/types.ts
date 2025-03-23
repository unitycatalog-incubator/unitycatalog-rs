import {
  TreeItemProps,
  InputProps,
  RadioGroupProps,
  CheckboxProps,
  TabListProps,
  SelectProps,
  DropdownProps,
} from "@fluentui/react-components";

export type TreeItemOnChange = NonNullable<TreeItemProps["onOpenChange"]>;
export type InputChange = NonNullable<InputProps["onChange"]>;
export type RadioChange = NonNullable<RadioGroupProps["onChange"]>;
export type CheckboxChange = NonNullable<CheckboxProps["onChange"]>;
export type TabSelect = NonNullable<TabListProps["onTabSelect"]>;
export type SelectChange = NonNullable<SelectProps["onChange"]>;
export type DropdownSelect = NonNullable<DropdownProps["onOptionSelect"]>;
