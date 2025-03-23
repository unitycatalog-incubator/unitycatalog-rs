import React, { createContext, Dispatch, SetStateAction } from "react";

export { NotifyProvider, useNotify } from "./notify";
export {
  TreeProvider,
  useTreeContext,
  useTreeScope,
  useTypeName,
  useTreeFunctions,
} from "./tree";

export type ExplorerPropsInner = {
  display?: "create" | "view" | "edit";
  scope?: string[];
};
export type ExplorerProps = ExplorerPropsInner & {
  update: Dispatch<SetStateAction<ExplorerPropsInner>>;
};
export const ExplorerContext = createContext<ExplorerProps>({
  update: () => {},
});
export const useExplorer = () => React.useContext(ExplorerContext);
export const ExplorerProvider = ExplorerContext.Provider;
