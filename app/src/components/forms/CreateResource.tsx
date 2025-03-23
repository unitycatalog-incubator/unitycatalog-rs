import {
  makeStyles,
  Toolbar,
  ToolbarButton,
  tokens,
  Text,
  ToolbarGroup,
  ToolbarToggleButton,
  ToolbarProps,
} from "@fluentui/react-components";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  useCallback,
  useState,
  Dispatch,
  SetStateAction,
  useRef,
  useEffect,
  useMemo,
  ComponentType,
} from "react";
import { useNotify, useExplorer, useTreeContext } from "../../context";
import {
  ArrowLeftRegular,
  AddRegular,
  BracesRegular,
  ArrowUploadRegular,
} from "@fluentui/react-icons";
import type monaco from "monaco-editor";
import JsonEditor from "./editor/JsonEditor";
import { OnMount } from "@monaco-editor/react";

const useStyles = makeStyles({
  root: {
    display: "flex",
    height: "100%",
    width: "100%",
    flexDirection: "column",
    overflowY: "scroll",
  },

  toolbar: {
    justifyContent: "space-between",
    borderBottomColor: tokens.colorNeutralForeground4,
    borderBottomWidth: "1px",
    borderBottomStyle: "solid",
  },

  content: {
    flex: 1,
    padding: "25px 25px 10px 25px",
    display: "flex",
    flexDirection: "column",
    rowGap: "10px",
    overflowY: "auto",
    backgroundColor: tokens.colorNeutralBackground2,
  },

  editor: {
    flex: 1,
  },

  editorHidden: {
    display: "none",
  },
});

export type CreateFormState<T> = {
  values: T;
  setValues: Dispatch<SetStateAction<T>>;
};

type CreateResourceProps<Req, Res> = {
  createFn: (values: Req) => Promise<Res>;
  FormComponent: ComponentType<CreateFormState<Req>>;
  defaultValues?: Req;
  resourceType: string;
  typeName: string;
  operation?: "create" | "update";
};
type ToggleChange = ToolbarProps["onCheckedValueChange"];

function CreateResource<Req, Res>({
  createFn,
  defaultValues,
  resourceType,
  FormComponent,
  typeName,
  operation,
}: CreateResourceProps<Req, Res>) {
  const styles = useStyles();
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [values, setValues] = useState<Req>(defaultValues ?? ({} as Req));
  const [checkedValues, setCheckedValues] = useState<Record<string, string[]>>({
    display: [],
  });
  const onChange: ToggleChange = (_e, { name, checkedItems }) => {
    setCheckedValues((s) => {
      return s ? { ...s, [name]: checkedItems } : { [name]: checkedItems };
    });
  };
  const showJson = useMemo(
    () => checkedValues.display.includes("json"),
    [checkedValues],
  );

  const notify = useNotify();
  const queryClient = useQueryClient();
  const queryKey = useTreeContext();
  const { update } = useExplorer();

  const mutation = useMutation({
    mutationFn: createFn,
    onError: () =>
      notify("error", `Failed to ${operation ?? "create"} ${resourceType}`),
    onSuccess: () => {
      notify(
        "success",
        `${resourceType} ${operation ?? "create"}ed successfully`,
      );
      queryClient.invalidateQueries({ queryKey });
      update({});
      setValues({} as Req);
    },
  });

  const onMount: OnMount = useCallback(
    (editor) => {
      editorRef.current = editor;
    },
    [editorRef],
  );

  const onSubmit = useCallback(() => {
    mutation.mutate(values);
  }, [mutation, values]);

  const onCancel = useCallback(() => {
    update({});
    setValues({} as Req);
  }, [update]);

  useEffect(() => {
    if (showJson && editorRef.current) {
      editorRef.current.setValue(JSON.stringify(values, null, 4));
    } else if (editorRef.current) {
      setValues(JSON.parse(editorRef.current.getValue()));
    }
  }, [showJson]);

  return (
    <div className={styles.root}>
      <Toolbar
        className={styles.toolbar}
        size="medium"
        checkedValues={checkedValues}
        onCheckedValueChange={onChange}
      >
        <ToolbarButton
          appearance="subtle"
          icon={<ArrowLeftRegular />}
          onClick={onCancel}
        />
        <Text>{`${operation === "update" ? "Update" : "Create"} ${resourceType}`}</Text>
        <ToolbarGroup>
          <ToolbarToggleButton
            aria-label="Toggle JSON editor"
            icon={<BracesRegular />}
            name="display"
            value="json"
          />
          <ToolbarButton
            appearance="subtle"
            icon={
              operation === "update" ? <ArrowUploadRegular /> : <AddRegular />
            }
            onClick={onSubmit}
          >
            {operation === "update" ? "Update" : "Create"}
          </ToolbarButton>
        </ToolbarGroup>
      </Toolbar>
      {!showJson && (
        <div className={styles.content}>
          <FormComponent values={values} setValues={setValues} />
        </div>
      )}
      {
        <div className={showJson ? styles.editor : styles.editorHidden}>
          <JsonEditor onMount={onMount} typeName={typeName} />
        </div>
      }
    </div>
  );
}

export default CreateResource;
