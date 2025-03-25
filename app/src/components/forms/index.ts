import { Dispatch, SetStateAction } from "react";
import CreateResource from "./CreateResource";
import ViewResource from "./ViewResource";
import Input from "./Input";

export { CreateResource, Input, ViewResource };

export type CreateFormState<T> = {
  values: T;
  setValues: Dispatch<SetStateAction<T>>;
};
