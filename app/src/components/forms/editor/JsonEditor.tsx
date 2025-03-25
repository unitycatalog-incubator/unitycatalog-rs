import Editor, { useMonaco, OnMount } from "@monaco-editor/react";
import { useEffect } from "react";
import schemas from "./schemas";

type JsonEditorProps = {
  onMount?: OnMount;
  typeName: string;
  readOnly?: boolean;
};

function JsonEditor({ onMount, typeName, readOnly }: JsonEditorProps) {
  const monaco = useMonaco();

  useEffect(() => {
    if (monaco) {
      monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
        schemas: Object.entries(schemas).map(([key, value]) => ({
          uri: value.$id,
          schema: value,
          fileMatch: [`${key}.json`],
        })),
      });
    }
  }, [monaco]);

  return (
    <Editor
      path={`${typeName}.json`}
      defaultLanguage="json"
      theme="vs-dark"
      options={{ automaticLayout: true, readOnly }}
      onMount={onMount}
    />
  );
}

export default JsonEditor;
