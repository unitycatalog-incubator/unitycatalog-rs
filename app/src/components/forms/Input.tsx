import { Field, Input as Input_ } from "@fluentui/react-components";
import { InputChange } from "../../types";

type InputProps = {
  label: string;
  value?: string;
  onChange?: InputChange;
  style?: React.CSSProperties;
  placeholder?: string;
  type?:
    | "number"
    | "search"
    | "time"
    | "text"
    | "email"
    | "password"
    | "tel"
    | "url"
    | "date"
    | "datetime-local"
    | "month"
    | "week"
    | undefined;
};

const Input = ({
  label,
  value,
  onChange,
  style,
  type,
  placeholder,
}: InputProps) => {
  return (
    <Field label={label} style={style}>
      <Input_
        value={value}
        onChange={onChange}
        type={type}
        autoComplete="off"
        autoCapitalize="off"
        autoCorrect="off"
        placeholder={placeholder}
      />
    </Field>
  );
};

export default Input;
